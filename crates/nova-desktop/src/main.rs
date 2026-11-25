// Engine-agnostic contract for the Servo integration; not yet wired to the renderer.
#[allow(dead_code)]
mod engine_api;
mod font_fallback;
mod input;
mod navigation;
mod storage;
mod viewport;
#[cfg(all(feature = "servo-embed", target_os = "windows"))]
mod servo_embed;
#[cfg(all(feature = "servo-embed", not(target_os = "windows")))]
compile_error!("The native Servo child-window integration currently requires Windows.");

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use obscura_browser::lifecycle::WaitUntil;
use obscura_browser::{BrowserContext, Page};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use navigation::{blocked_google_search_query, build_search_url, normalize_address_with_search, NavigationHistory};
use storage::{Bookmark, BrowserDataStore, BrowserSession, BrowserSettings, SessionTab, Visit, MAX_TABS};
use viewport::Viewport;

enum Command {
    FrontendReady,
    Navigate(String),
    SearchWithProvider(String, String),
    DismissSearchBlock,
    NewTab,
    ReopenTab,
    CloseTab(String),
    ActivateTab(String),
    Back,
    Forward,
    Reload,
    Resize(f32, f32),
    Click(f32, f32),
    Scroll(f32, f32, f32, f32),
    Type(String),
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    ServoLocation(String, String),
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    ServoTitle(String, String),
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    ServoHistory(String, bool, bool),
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    ServoError(String, String),
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    ServoExited(String),
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    ServoFocusAddress,
    Shutdown,
}

struct EngineState {
    commands: mpsc::Sender<Command>,
    browser_data: Arc<BrowserDataStore>,
}

struct BrowserTab {
    id: String,
    title: String,
    address: String,
    // The native Servo mode owns webpage JS; the legacy V8 page is absent.
    page: Option<Page>,
    history: NavigationHistory,
    last_frame: Option<u64>,
    restore_url: Option<String>,
    blocked_search: Option<String>,
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    servo_can_back: bool,
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    servo_can_forward: bool,
    // Preserve the live Servo page when user visits Nova's internal Home.
    // Back restores the same WebView (scroll/forms), not a new network load.
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    servo_home_back: Option<(String, String)>,
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    servo_home_forward: bool,
    // Track visits independently of tab.address: the address is already
    // set during navigation, before Servo reports the completed URL.
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    servo_last_recorded_url: Option<String>,
}

impl BrowserTab {
    fn new(id: String, context: Option<Arc<BrowserContext>>, viewport: Viewport) -> Self {
        let page = context.map(|context| {
            let mut page = Page::new(id.clone(), context);
            page.set_viewport(viewport.css_size());
            page
        });
        Self {
            id,
            title: "New tab".to_string(),
            address: String::new(),
            page,
            history: NavigationHistory::default(),
            last_frame: None,
            restore_url: None,
            blocked_search: None,
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            servo_can_back: false,
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            servo_can_forward: false,
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            servo_home_back: None,
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            servo_home_forward: false,
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            servo_last_recorded_url: None,
        }
    }

    fn restored(id: String, context: Option<Arc<BrowserContext>>, viewport: Viewport, saved: SessionTab) -> Self {
        let mut tab = Self::new(id, context, viewport);
        tab.title = if saved.title.is_empty() { "Untitled page".into() } else { saved.title };
        tab.address = saved.url.clone();
        if !saved.url.is_empty() {
            tab.restore_url = Some(saved.url);
        }
        tab
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusUpdate {
    address: Option<String>,
    message: String,
    loading: bool,
    show_status: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryUpdate {
    can_back: bool,
    can_forward: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TabInfo {
    id: String,
    title: String,
    address: String,
    active: bool,
    closable: bool,
}

fn report(app: &AppHandle, address: Option<String>, message: String, loading: bool) {
    let show_status = message.starts_with("Navigation error:")
        || message.starts_with("Engine startup failed:")
        || message.starts_with("Invalid")
        || message.starts_with("Only ")
        || message.starts_with("Too many tabs");
    let _ = app.emit(
        "browser-status",
        StatusUpdate {
            address,
            message,
            loading,
            show_status,
        },
    );
}


fn persist_session(browser_data: &BrowserDataStore, tabs: &[BrowserTab], active_id: &str) {
    let session = BrowserSession {
        active_index: tabs.iter().position(|tab| tab.id == active_id).unwrap_or(0),
        tabs: tabs.iter().map(|tab| SessionTab {
            title: tab.title.clone(),
            url: tab.address.clone(),
        }).collect(),
    };
    if let Err(error) = browser_data.save_session(session) {
        eprintln!("NovaBrowser could not save the tab session: {error}");
    }
}

fn publish_tab_list(app: &AppHandle, tabs: &[BrowserTab], active_id: &str) {
    let closable = tabs.len() > 1;
    let items: Vec<TabInfo> = tabs
        .iter()
        .map(|tab| TabInfo {
            id: tab.id.clone(),
            title: tab.title.clone(),
            address: tab.address.clone(),
            active: tab.id == active_id,
            closable,
        })
        .collect();
    let _ = app.emit("browser-tabs", items);
}

fn publish_search_blocked(app: &AppHandle, tab: &BrowserTab) {
    // Only the search query is sent to the trusted UI. A null payload clears
    // the notice; no challenge tokens, IPs or cookies are exposed.
    let _ = app.emit("browser-search-blocked", tab.blocked_search.clone());
}

fn publish_buttons(app: &AppHandle, tab: &BrowserTab) {
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    {
        let _ = app.emit("browser-history", HistoryUpdate {
            can_back: if tab.address == "nova://home" {
                tab.servo_home_back.is_some()
            } else {
                tab.servo_can_back
            },
            can_forward: if tab.address == "nova://home" {
                false
            } else {
                tab.servo_home_forward || tab.servo_can_forward
            },
        });
        return;
    }
    #[cfg(not(all(feature = "servo-embed", target_os = "windows")))]
    let _ = app.emit(
        "browser-history",
        HistoryUpdate {
            can_back: tab.history.previous().is_some(),
            can_forward: tab.history.next().is_some(),
        },
    );
}

fn document_title(page: &mut Page) -> String {
    let title = page.evaluate("document.title");
    title.as_str().unwrap_or("").trim().to_string()
}

fn publish_tab_title(app: &AppHandle, title: &str) {
    let title = if title.is_empty() { "Untitled page" } else { title };
    let _ = app.emit("browser-title", title.to_string());
}

fn publish_frame(app: &AppHandle, png: Vec<u8>) {
    let _ = app.emit("browser-frame", BASE64.encode(png));
}

/// Frame comparisons avoid redundant updates on visually static pages.
fn capture(page: &Page, viewport: Viewport, app: &AppHandle, hash: &mut Option<u64>) {
    if cfg!(all(feature = "servo-embed", target_os = "windows")) {
        let _ = (page, viewport, app, hash);
        return; // Never transport PNG frames when the native Servo surface is active.
    }
    let Some(png) = page.screenshot(viewport.css_size()) else {
        let _ = app.emit("browser-frame", String::new());
        return;
    };
    let mut hasher = DefaultHasher::new();
    png.hash(&mut hasher);
    let fingerprint = hasher.finish();
    if *hash == Some(fingerprint) {
        return;
    }
    *hash = Some(fingerprint);
    publish_frame(app, png);
}

fn navigate(
    tab: &mut BrowserTab,
    runtime: &tokio::runtime::Runtime,
    viewport: Viewport,
    app: &AppHandle,
    browser_data: &BrowserDataStore,
    target: &str,
    history_index: Option<usize>,
) {
    tab.blocked_search = None;
    publish_search_blocked(app, tab);
    // Internal Home is part of the trusted Tauri chrome, never a network page
    // or JavaScript payload passed to the Obscura engine.
    if target == "nova://home" {
        #[cfg(all(feature = "servo-embed", target_os = "windows"))]
        {
            if tab.address.starts_with("http://") || tab.address.starts_with("https://") {
                tab.servo_home_back = Some((tab.address.clone(), tab.title.clone()));
            }
            tab.servo_home_forward = false;
        }
        if let Some(index) = history_index {
            tab.history.select(index);
        } else {
            tab.history.record(target.to_string());
        }
        tab.address = target.to_string();
        tab.title = "New tab".to_string();
        tab.last_frame = None;
        tab.restore_url = None;
        let _ = app.emit("browser-frame", String::new());
        report(app, Some(target.to_string()), "New tab".into(), false);
        publish_tab_title(app, &tab.title);
        publish_buttons(app, tab);
        return;
    }
    // All external navigation crosses the same engine-independent scheme
    // gate, whether the current renderer is Obscura or a future Servo view.
    // Internal NovaBrowser routes stay in the trusted browser shell.
    if let Err(error) = engine_api::external_page_url(target) {
        report(app, None, format!("Navigation error: {error}"), false);
        return;
    }
    // In native mode this tab owns a real Servo WebView. Do not render its
    // document a second time with Obscura/V8 or generate PNG frame events.
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    if let Some(bridge) = app.try_state::<servo_embed::ServoBridge>() {
        let url = match engine_api::external_page_url(target) {
            Ok(url) => url,
            Err(error) => {
                report(app, None, format!("Navigation error: {error}"), false);
                return;
            }
        };
        if let Err(error) = bridge.send(servo_embed::Control::Navigate {
            tab: tab.id.clone(), url,
        }) {
            report(app, None, format!("Navigation error: {error}"), false);
            return;
        }
        if let Some(index) = history_index {
            tab.history.select(index);
        } else {
            tab.history.record(target.to_string());
        }
        tab.servo_home_back = None;
        tab.servo_home_forward = false;
        tab.servo_last_recorded_url = None;
        tab.address = target.to_owned();
        tab.title = "Loading...".to_string();
        tab.restore_url = None;
        tab.last_frame = None;
        report(app, Some(target.to_owned()), format!("Loading {target}"), true);
        publish_tab_title(app, &tab.title);
        publish_buttons(app, tab);
        return;
    }
    report(app, None, format!("Loading {target}"), true);
    let result: Result<(), String> = runtime.block_on(async {
        tab.page.as_mut().expect("legacy page only")
            .navigate_with_wait(target, WaitUntil::Load)
            .await
            .map_err(|error| error.to_string())?;
        tab.page.as_mut().expect("legacy page only").settle(180).await;
        tab.page.as_mut().expect("legacy page only").prepare_screenshot_resources(300).await;
        Ok(())
    });

    match result {
        Ok(()) => {
            let url = tab.page.as_mut().expect("legacy page only").url_string();
            let title = document_title(tab.page.as_mut().expect("legacy page only"));
            let blocked_query = blocked_google_search_query(target, &url);
            if let Some(index) = history_index {
                tab.history.select(index);
            } else {
                tab.history.record(url.clone());
            }
            tab.address = url.clone();
            tab.title = if blocked_query.is_some() {
                "Google search unavailable".to_string()
            } else if title.is_empty() {
                "Untitled page".into()
            } else {
                title.clone()
            };
            tab.blocked_search = blocked_query;
            tab.last_frame = None;
            if tab.blocked_search.is_none() && browser_data.settings().save_history {
                let _ = browser_data.record_visit(title.clone(), url.clone());
            }
            let message = if tab.blocked_search.is_some() {
                "Google requested traffic verification".to_string()
            } else if title.is_empty() {
                "Page loaded".to_string()
            } else {
                format!("Page loaded: {title}")
            };
            report(app, Some(url), message, false);
            publish_tab_title(app, &tab.title);
            publish_buttons(app, tab);
            publish_search_blocked(app, tab);
            if tab.blocked_search.is_some() {
                let _ = app.emit("browser-frame", String::new());
            } else {
                capture(tab.page.as_ref().expect("legacy page only"), viewport, app, &mut tab.last_frame);
            }
        }
        Err(error) => report(app, None, format!("Navigation error: {error}"), false),
    }
}

/// Observe same-document navigation where the engine has updated its URL.
fn sync_history(tab: &mut BrowserTab, app: &AppHandle, browser_data: &BrowserDataStore) -> bool {
    let url = tab.page.as_mut().expect("legacy page only").url_string();
    if (url.starts_with("https://") || url.starts_with("http://"))
        && tab.history.current() != Some(url.as_str())
    {
        // Site-initiated searches may redirect without going through the
        // address-bar Navigate command. The trusted Google continue= URL
        // provides the original search query in that case.
        let blocked = blocked_google_search_query(&tab.address, &url);
        tab.history.record(url.clone());
        let title = document_title(tab.page.as_mut().expect("legacy page only"));
        tab.address = url.clone();
        tab.title = if blocked.is_some() {
            "Google search unavailable".to_string()
        } else if title.is_empty() {
            "Untitled page".into()
        } else {
            title.clone()
        };
        tab.blocked_search = blocked;
        if tab.blocked_search.is_none() && browser_data.settings().save_history {
            let _ = browser_data.record_visit(title.clone(), url.clone());
        }
        let message = if tab.blocked_search.is_some() {
            "Google requested traffic verification".to_string()
        } else if title.is_empty() {
            "Page loaded".to_string()
        } else {
            format!("Page loaded: {title}")
        };
        report(app, Some(url), message, false);
        publish_tab_title(app, &tab.title);
        publish_buttons(app, tab);
        publish_search_blocked(app, tab);
        if tab.blocked_search.is_some() {
            let _ = app.emit("browser-frame", String::new());
        }
        return true;
    }
    false
}

fn publish_active_tab(app: &AppHandle, tabs: &mut [BrowserTab], active_id: &str, viewport: Viewport) {
    publish_tab_list(app, tabs, active_id);
    let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) else {
        return;
    };
    report(
        app,
        if tab.address.is_empty() { None } else { Some(tab.address.clone()) },
        if tab.address.is_empty() { "New tab".into() } else { tab.title.clone() },
        false,
    );
    publish_tab_title(app, &tab.title);
    publish_buttons(app, tab);
    publish_search_blocked(app, tab);
    tab.last_frame = None;
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    if let Some(bridge) = app.try_state::<servo_embed::ServoBridge>() {
        let _ = bridge.send(servo_embed::Control::Select { tab: tab.id.clone() });
        let _ = app.emit("browser-frame", String::new());
        return;
    }
    if tab.blocked_search.is_some() {
        let _ = app.emit("browser-frame", String::new());
    } else if tab.address.starts_with("http://") || tab.address.starts_with("https://") {
        capture(tab.page.as_ref().expect("legacy page only"), viewport, app, &mut tab.last_frame);
    } else {
        let _ = app.emit("browser-frame", String::new());
    }
}

fn engine_thread(
    app: AppHandle,
    browser_data: Arc<BrowserDataStore>,
    commands: mpsc::Receiver<Command>,
) {
    // Font configuration must happen before the renderer initializes its
    // process-wide font database. Loading a curated set avoids scanning all
    // installed Windows fonts (which can be very large).
    if !cfg!(all(feature = "servo-embed", target_os = "windows")) {
        font_fallback::configure_desktop_fonts();
    }

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            report(&app, None, format!("Engine startup failed: {error}"), false);
            return;
        }
    };

    // Synchronous V8/DOM callbacks (click, scroll, input, evaluation and page
    // teardown) can start Tokio-backed fetches outside runtime.block_on().
    // Enter the runtime for the entire engine-thread lifetime so async ops
    // always find a reactor. block_on below still drives their progress.
    let _engine_runtime_guard = runtime.enter();

    // All page isolates stay on this thread. Tabs share cookies and the HTTP
    // client through BrowserContext while keeping their DOM and JS realm apart.
    // Skip BrowserContext and all unused V8-backed Page allocations when
    // external websites are rendered exclusively by the Servo sidecar.
    let context = if cfg!(all(feature = "servo-embed", target_os = "windows")) {
        None
    } else {
        Some(Arc::new(BrowserContext::new("nova-default".to_owned())))
    };
    let mut viewport = Viewport::DEFAULT;
    // Recover tab metadata without opening every saved website at once.
    // Inactive tabs are restored lazily when the user selects them.
    let stored = if browser_data.settings().startup == "restore" {
        browser_data.session()
    } else {
        BrowserSession::default()
    };
    let selected_index = stored.active_index;
    let had_saved_tabs = !stored.tabs.is_empty();
    let mut tabs: Vec<BrowserTab> = stored.tabs.into_iter().enumerate()
        .map(|(index, saved)| BrowserTab::restored(
            format!("nova-tab-{}", index + 1), context.clone(), viewport, saved,
        ))
        .collect();
    if tabs.is_empty() {
        tabs.push(BrowserTab::new("nova-tab-1".into(), context.clone(), viewport));
    }
    let mut active_id = tabs[selected_index.min(tabs.len() - 1)].id.clone();
    let mut next_tab_id = tabs.len() as u64 + 1;
    let mut recently_closed: Vec<SessionTab> = Vec::new();
    let mut frontend_ready = false;

    loop {
        match commands.recv_timeout(Duration::from_millis(500)) {
            Ok(Command::FrontendReady) => {
                if frontend_ready { continue; }
                frontend_ready = true;
                publish_active_tab(&app, &mut tabs, &active_id, viewport);
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if let Some(url) = tab.restore_url.take() {
                        navigate(tab, &runtime, viewport, &app, &browser_data, &url, None);
                    } else if tab.address.is_empty() && !had_saved_tabs {
                        let settings = browser_data.settings();
                        let target = if settings.startup == "homepage" {
                            settings.homepage.as_str()
                        } else {
                            "nova://home"
                        };
                        navigate(tab, &runtime, viewport, &app, &browser_data, target, None);
                    }
                }
                publish_tab_list(&app, &tabs, &active_id);
                persist_session(&browser_data, &tabs, &active_id);
            }
            Ok(Command::Navigate(raw)) => {
                let search_engine = browser_data.settings().search_engine;
                match normalize_address_with_search(&raw, &search_engine) {
                    Ok(url) => {
                        if let Some(index) = tabs.iter().position(|tab| tab.id == active_id) {
                            publish_tab_list(&app, &tabs, &active_id);
                            tabs[index].restore_url = None;
                            navigate(&mut tabs[index], &runtime, viewport, &app, &browser_data, &url, None);
                            publish_tab_list(&app, &tabs, &active_id);
                            persist_session(&browser_data, &tabs, &active_id);
                        }
                    }
                    Err(error) => report(&app, None, error.to_string(), false),
                }
            }
            Ok(Command::SearchWithProvider(query, provider)) => {
                // A provider switch is user-initiated and never updates
                // the persisted default search engine.
                match build_search_url(&query, &provider) {
                    Ok(url) => {
                        if let Some(index) = tabs.iter().position(|tab| tab.id == active_id) {
                            tabs[index].restore_url = None;
                            navigate(&mut tabs[index], &runtime, viewport, &app, &browser_data, &url, None);
                            publish_tab_list(&app, &tabs, &active_id);
                            persist_session(&browser_data, &tabs, &active_id);
                        }
                    }
                    Err(error) => report(&app, None, error.to_string(), false),
                }
            }
            Ok(Command::DismissSearchBlock) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if tab.blocked_search.take().is_some() {
                        publish_search_blocked(&app, tab);
                        tab.last_frame = None;
                        capture(tab.page.as_ref().expect("legacy page only"), viewport, &app, &mut tab.last_frame);
                    }
                }
            }
            Ok(Command::NewTab) => {
                if tabs.len() >= MAX_TABS {
                    report(&app, None, format!("Too many tabs (maximum {MAX_TABS})"), false);
                    continue;
                }
                let id = format!("nova-tab-{next_tab_id}");
                next_tab_id += 1;
                tabs.push(BrowserTab::new(id.clone(), context.clone(), viewport));
                active_id = id;
                publish_active_tab(&app, &mut tabs, &active_id, viewport);
                let settings = browser_data.settings();
                let new_tab_target = match settings.new_tab.as_str() {
                    "nova" => Some("nova://home"),
                    "homepage" => Some(settings.homepage.as_str()),
                    _ => None,
                };
                if let Some(target) = new_tab_target {
                    if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                        navigate(tab, &runtime, viewport, &app, &browser_data, target, None);
                    }
                }
                publish_tab_list(&app, &tabs, &active_id);
                persist_session(&browser_data, &tabs, &active_id);
            }
            Ok(Command::CloseTab(id)) => {
                if tabs.len() > 1 {
                    if let Some(closing_index) = tabs.iter().position(|tab| tab.id == id) {
                        let old_active_index = tabs.iter().position(|tab| tab.id == active_id).unwrap_or(0);
                        let closing_active = id == active_id;
                        let closed_tab = tabs.remove(closing_index);
                        recently_closed.push(SessionTab {
                            title: closed_tab.title.clone(),
                            url: closed_tab.address.clone(),
                        });
                        if recently_closed.len() > 16 {
                            recently_closed.remove(0);
                        }
                        #[cfg(all(feature = "servo-embed", target_os = "windows"))]
                        if let Some(bridge) = app.try_state::<servo_embed::ServoBridge>() {
                            let _ = bridge.send(servo_embed::Control::Close {
                                tab: closed_tab.id.clone(),
                            });
                        }
                        runtime.block_on(async { drop(closed_tab) });
                        if closing_active {
                            let next_index = old_active_index.saturating_sub(1).min(tabs.len() - 1);
                            active_id = tabs[next_index].id.clone();
                            publish_active_tab(&app, &mut tabs, &active_id, viewport);
                        } else {
                            publish_tab_list(&app, &tabs, &active_id);
                        }
                        persist_session(&browser_data, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::ReopenTab) => {
                if tabs.len() < MAX_TABS {
                    if let Some(saved) = recently_closed.pop() {
                        let id = format!("nova-tab-{next_tab_id}");
                        next_tab_id += 1;
                        let target = saved.url.clone();
                        tabs.push(BrowserTab::restored(id.clone(), context.clone(), viewport, saved));
                        active_id = id;
                        publish_active_tab(&app, &mut tabs, &active_id, viewport);
                        if !target.is_empty() {
                            if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                                tab.restore_url = None;
                                navigate(tab, &runtime, viewport, &app, &browser_data, &target, None);
                            }
                        }
                        publish_tab_list(&app, &tabs, &active_id);
                        persist_session(&browser_data, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::ActivateTab(id)) => {
                if tabs.iter().any(|tab| tab.id == id) && active_id != id {
                    active_id = id;
                    publish_active_tab(&app, &mut tabs, &active_id, viewport);
                    if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                        if let Some(url) = tab.restore_url.take() {
                            navigate(tab, &runtime, viewport, &app, &browser_data, &url, None);
                            publish_tab_list(&app, &tabs, &active_id);
                        }
                    }
                    persist_session(&browser_data, &tabs, &active_id);
                }
            }
            Ok(Command::Back) => {
                #[cfg(all(feature = "servo-embed", target_os = "windows"))]
                {
                    let restored = if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                        if tab.address == "nova://home" {
                            if let Some((url, title)) = tab.servo_home_back.take() {
                                tab.address = url;
                                tab.title = title;
                                tab.servo_home_forward = true;
                                true
                            } else { false }
                        } else { false }
                    } else { false };
                    if restored {
                        publish_active_tab(&app, &mut tabs, &active_id, viewport);
                        persist_session(&browser_data, &tabs, &active_id);
                        continue;
                    }
                }
                #[cfg(all(feature = "servo-embed", target_os = "windows"))]
                if let Some(bridge) = app.try_state::<servo_embed::ServoBridge>() {
                    let _ = bridge.send(servo_embed::Control::Back { tab: active_id.clone() });
                    continue;
                }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if let Some((index, url)) = tab.history.previous() {
                        navigate(tab, &runtime, viewport, &app, &browser_data, &url, Some(index));
                        publish_tab_list(&app, &tabs, &active_id);
                        persist_session(&browser_data, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Forward) => {
                #[cfg(all(feature = "servo-embed", target_os = "windows"))]
                {
                    let to_home = tabs.iter().any(|tab|
                        tab.id == active_id && tab.address != "nova://home"
                            && tab.servo_home_forward);
                    if to_home {
                        if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                            navigate(tab, &runtime, viewport, &app, &browser_data, "nova://home", None);
                        }
                        publish_tab_list(&app, &tabs, &active_id);
                        persist_session(&browser_data, &tabs, &active_id);
                        continue;
                    }
                }
                #[cfg(all(feature = "servo-embed", target_os = "windows"))]
                if let Some(bridge) = app.try_state::<servo_embed::ServoBridge>() {
                    let _ = bridge.send(servo_embed::Control::Forward { tab: active_id.clone() });
                    continue;
                }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if let Some((index, url)) = tab.history.next() {
                        navigate(tab, &runtime, viewport, &app, &browser_data, &url, Some(index));
                        publish_tab_list(&app, &tabs, &active_id);
                        persist_session(&browser_data, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Reload) => {
                #[cfg(all(feature = "servo-embed", target_os = "windows"))]
                if let Some(bridge) = app.try_state::<servo_embed::ServoBridge>() {
                    if tabs.iter().any(|tab| tab.id == active_id &&
                        (tab.address.starts_with("http://") || tab.address.starts_with("https://"))) {
                        let _ = bridge.send(servo_embed::Control::Reload { tab: active_id.clone() });
                        continue;
                    }
                }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if let Some(url) = tab.history.current().map(str::to_owned) {
                        navigate(tab, &runtime, viewport, &app, &browser_data, &url, None);
                        publish_tab_list(&app, &tabs, &active_id);
                        persist_session(&browser_data, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Resize(width, height)) => {
                if cfg!(all(feature = "servo-embed", target_os = "windows")) { continue; }
                if let Some(next) = Viewport::from_window(width, height) {
                    if next.width != viewport.width || next.height != viewport.height {
                        viewport = next;
                        for tab in &mut tabs {
                            tab.page.as_mut().expect("legacy page only").set_viewport(next.css_size());
                            tab.last_frame = None;
                        }
                        if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                            if tab.blocked_search.is_none()
                                && (tab.address.starts_with("http://") || tab.address.starts_with("https://"))
                            {
                                capture(tab.page.as_ref().expect("legacy page only"), next, &app, &mut tab.last_frame);
                            }
                        }
                    }
                }
            }
            Ok(Command::Click(x, y)) => {
                if cfg!(all(feature = "servo-embed", target_os = "windows")) { continue; }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if tab.blocked_search.is_none()
                        && (tab.address.starts_with("http://") || tab.address.starts_with("https://"))
                    {
                        input::click(tab.page.as_mut().expect("legacy page only"), viewport, x, y);
                        runtime.block_on(tab.page.as_mut().expect("legacy page only").settle(60));
                        let history_changed = sync_history(tab, &app, &browser_data);
                        if tab.blocked_search.is_none() {
                            capture(tab.page.as_ref().expect("legacy page only"), viewport, &app, &mut tab.last_frame);
                        }
                        publish_tab_list(&app, &tabs, &active_id);
                        if history_changed {
                            persist_session(&browser_data, &tabs, &active_id);
                        }
                    }
                }
            }
            Ok(Command::Scroll(x, y, dx, dy)) => {
                if cfg!(all(feature = "servo-embed", target_os = "windows")) { continue; }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if tab.blocked_search.is_none()
                        && (tab.address.starts_with("http://") || tab.address.starts_with("https://"))
                    {
                        input::scroll(tab.page.as_mut().expect("legacy page only"), viewport, x, y, dx, dy);
                        runtime.block_on(tab.page.as_mut().expect("legacy page only").settle(40));
                        capture(tab.page.as_ref().expect("legacy page only"), viewport, &app, &mut tab.last_frame);
                    }
                }
            }
            Ok(Command::Type(key)) => {
                if cfg!(all(feature = "servo-embed", target_os = "windows")) { continue; }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if tab.blocked_search.is_none()
                        && (tab.address.starts_with("http://") || tab.address.starts_with("https://"))
                    {
                        input::key(tab.page.as_mut().expect("legacy page only"), &key);
                        runtime.block_on(tab.page.as_mut().expect("legacy page only").settle(40));
                        let history_changed = sync_history(tab, &app, &browser_data);
                        if tab.blocked_search.is_none() {
                            capture(tab.page.as_ref().expect("legacy page only"), viewport, &app, &mut tab.last_frame);
                        }
                        publish_tab_list(&app, &tabs, &active_id);
                        if history_changed {
                            persist_session(&browser_data, &tabs, &active_id);
                        }
                    }
                }
            }
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            Ok(Command::ServoLocation(tab_id, url)) => {
                if engine_api::external_page_url(&url).is_err() { continue; }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    // The hidden WebView may redirect or finish loading after
                    // its tab switched to nova://home. Preserve the trusted
                    // internal route and update only its return target.
                    if tab.address == "nova://home" {
                        if let Some((previous_url, _)) = tab.servo_home_back.as_mut() {
                            *previous_url = url;
                        }
                        continue;
                    }
                    if tab.address != url {
                        tab.history.record(url.clone());
                        tab.address = url.clone();
                    }
                    // A typed URL was already put in tab.address before
                    // Servo loaded it; don't use address inequality as the
                    // signal for recording visits.
                    if tab.servo_last_recorded_url.as_deref() != Some(url.as_str()) {
                        tab.servo_last_recorded_url = Some(url.clone());
                        if browser_data.settings().save_history {
                            let _ = browser_data.record_visit(tab.title.clone(), url.clone());
                        }
                    }
                    if tab_id == active_id {
                        report(&app, Some(url), "Page loaded".into(), false);
                        publish_buttons(&app, tab);
                    }
                    publish_tab_list(&app, &tabs, &active_id);
                    persist_session(&browser_data, &tabs, &active_id);
                }
            }
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            Ok(Command::ServoTitle(tab_id, title)) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    let title = if title.trim().is_empty() {
                        "Untitled page".to_string()
                    } else { title };
                    if tab.address == "nova://home" {
                        // A previously loaded site can finish title updates
                        // while its WebView is hidden behind Nova Home.
                        if let Some((_, previous_title)) = tab.servo_home_back.as_mut() {
                            *previous_title = title;
                        }
                        continue;
                    }
                    if tab.title == title { continue; }
                    tab.title = title;
                    if browser_data.settings().save_history {
                        let _ = browser_data.update_visit_title(&tab.address, &tab.title);
                    }
                    if tab_id == active_id { publish_tab_title(&app, &tab.title); }
                    publish_tab_list(&app, &tabs, &active_id);
                    persist_session(&browser_data, &tabs, &active_id);
                }
            }
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            Ok(Command::ServoHistory(tab_id, back, forward)) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.servo_can_back = back;
                    tab.servo_can_forward = forward;
                    if tab_id == active_id { publish_buttons(&app, tab); }
                }
            }
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            Ok(Command::ServoError(tab_id, error)) => {
                if tab_id == active_id && tabs.iter().any(|tab|
                    tab.id == tab_id && tab.address != "nova://home")
                {
                    report(&app, None, format!("Navigation error: Servo: {error}"), false);
                }
            }
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            Ok(Command::ServoFocusAddress) => {
                // Ctrl+L from the native child HWND needs to return focus to
                // NovaBrowser's trusted WebView2 address bar.
                if let Some(chrome) = app.get_webview("main") {
                    let _ = chrome.set_focus();
                }
                let _ = app.emit("browser-focus-address", ());
            }
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            Ok(Command::ServoExited(reason)) => {
                report(&app, None, format!("Navigation error: Servo renderer exited: {reason}"), false);
            }
            Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                persist_session(&browser_data, &tabs, &active_id);
                break;
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if cfg!(all(feature = "servo-embed", target_os = "windows")) { continue; }
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if tab.blocked_search.is_none()
                        && (tab.address.starts_with("http://") || tab.address.starts_with("https://"))
                    {
                        runtime.block_on(async {
                            tab.page.as_mut().expect("legacy page only").settle(50).await;
                            tab.page.as_mut().expect("legacy page only").prepare_screenshot_resources(75).await;
                        });
                        let history_changed = sync_history(tab, &app, &browser_data);
                        if tab.blocked_search.is_none() {
                            capture(tab.page.as_ref().expect("legacy page only"), viewport, &app, &mut tab.last_frame);
                        }
                        publish_tab_list(&app, &tabs, &active_id);
                        if history_changed {
                            persist_session(&browser_data, &tabs, &active_id);
                        }
                    }
                }
            }
        }
    }

    // Page teardown aborts document-owned background loads. Keep that work
    // inside Tokio's entered context so dropping a tab never loses its reactor.
    runtime.block_on(async move { drop(tabs) });
}

fn send_command(state: &State<'_, EngineState>, command: Command) -> Result<(), String> {
    state
        .commands
        .send(command)
        .map_err(|error| format!("Browser engine stopped: {error}"))
}

#[tauri::command]
fn browser_frontend_ready(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::FrontendReady)
}

#[tauri::command]
fn browser_navigate(state: State<'_, EngineState>, address: String) -> Result<(), String> {
    send_command(&state, Command::Navigate(address))
}

#[tauri::command]
fn browser_search_with_provider(
    state: State<'_, EngineState>,
    query: String,
    provider: String,
) -> Result<(), String> {
    // Reject unsupported providers before enqueuing work.
    build_search_url(&query, &provider)?;
    send_command(&state, Command::SearchWithProvider(query, provider))
}

#[tauri::command]
fn browser_dismiss_search_notice(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::DismissSearchBlock)
}

#[tauri::command]
fn browser_new_tab(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::NewTab)
}

#[tauri::command]
fn browser_reopen_tab(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::ReopenTab)
}

#[tauri::command]
fn browser_close_tab(state: State<'_, EngineState>, tab_id: String) -> Result<(), String> {
    send_command(&state, Command::CloseTab(tab_id))
}

#[tauri::command]
fn browser_activate_tab(state: State<'_, EngineState>, tab_id: String) -> Result<(), String> {
    send_command(&state, Command::ActivateTab(tab_id))
}

#[tauri::command]
fn browser_back(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::Back)
}

#[tauri::command]
fn browser_forward(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::Forward)
}

#[tauri::command]
fn browser_reload(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::Reload)
}

#[tauri::command]
fn browser_resize(state: State<'_, EngineState>, width: f32, height: f32) -> Result<(), String> {
    send_command(&state, Command::Resize(width, height))
}

#[tauri::command]
fn browser_click(state: State<'_, EngineState>, x: f32, y: f32) -> Result<(), String> {
    send_command(&state, Command::Click(x, y))
}

#[tauri::command]
fn browser_scroll(
    state: State<'_, EngineState>,
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
) -> Result<(), String> {
    send_command(&state, Command::Scroll(x, y, dx, dy))
}

#[tauri::command]
fn browser_type(state: State<'_, EngineState>, key: String) -> Result<(), String> {
    send_command(&state, Command::Type(key))
}

#[tauri::command]
fn browser_toggle_bookmark(
    state: State<'_, EngineState>,
    title: String,
    url: String,
) -> Result<bool, String> {
    state.browser_data.toggle_bookmark(title, url)
}

#[tauri::command]
fn browser_is_bookmarked(state: State<'_, EngineState>, url: String) -> bool {
    state.browser_data.is_bookmarked(&url)
}

#[tauri::command]
fn browser_get_bookmarks(state: State<'_, EngineState>) -> Vec<Bookmark> {
    state.browser_data.bookmarks()
}

#[tauri::command]
fn browser_remove_bookmark(state: State<'_, EngineState>, url: String) -> Result<(), String> {
    state.browser_data.remove_bookmark(&url)
}

#[tauri::command]
fn browser_clear_bookmarks(state: State<'_, EngineState>) -> Result<(), String> {
    state.browser_data.clear_bookmarks()
}

#[tauri::command]
fn browser_get_history(state: State<'_, EngineState>) -> Vec<Visit> {
    state.browser_data.history()
}

#[tauri::command]
fn browser_clear_history(state: State<'_, EngineState>) -> Result<(), String> {
    state.browser_data.clear_history()
}

#[tauri::command]
fn browser_get_settings(state: State<'_, EngineState>) -> BrowserSettings {
    state.browser_data.settings()
}

#[tauri::command]
fn browser_save_settings(
    state: State<'_, EngineState>,
    theme: String,
    homepage: String,
    startup: String,
    new_tab: String,
    search_engine: String,
    save_history: bool,
    show_home_button: bool,
    show_bookmarks_bar: bool,
    language: String,
) -> Result<BrowserSettings, String> {
    if !matches!(theme.as_str(), "light" | "dark" | "system") {
        return Err("Choose a valid appearance".to_string());
    }
    if !matches!(startup.as_str(), "restore" | "homepage") {
        return Err("Choose a valid startup behavior".to_string());
    }
    if !matches!(new_tab.as_str(), "nova" | "homepage" | "blank") {
        return Err("Choose a valid new tab behavior".to_string());
    }
    if !matches!(search_engine.as_str(), "google" | "bing" | "duckduckgo") {
        return Err("Choose a valid search engine".to_string());
    }
    if !matches!(language.as_str(), "system" | "en" | "vi") {
        return Err("Choose a supported interface language".to_string());
    }
    if homepage.len() > 4096 {
        return Err("The homepage must be 4,096 bytes or fewer".to_string());
    }
    let homepage = normalize_address_with_search(&homepage, &search_engine)
        .map_err(|error| error.to_string())?;
    let settings = BrowserSettings {
        theme, homepage, startup, new_tab, search_engine, save_history,
        show_home_button, show_bookmarks_bar, language,
    };
    state.browser_data.save_settings(settings.clone())?;
    Ok(settings)
}

#[tauri::command]
fn browser_get_engine_mode() -> &'static str {
    if cfg!(all(feature = "servo-embed", target_os = "windows")) {
        "servo"
    } else {
        "obscura"
    }
}

/// Trusted chrome reports its vertical content boundary in CSS pixels.
/// The native host uses the actual parent client size to avoid shrinking the
/// website when the WebView2 chrome itself is clipped to the toolbar height.
#[tauri::command]
fn browser_surface_layout(app: AppHandle, top: f64, visible: bool) -> Result<(), String> {
    #[cfg(all(feature = "servo-embed", target_os = "windows"))]
    {
        if !top.is_finite() || top < 0.0 { return Err("Invalid chrome boundary".into()); }
        let window = app.get_webview_window("main").ok_or("Browser window unavailable")?;
        let full = window.inner_size().map_err(|error| error.to_string())?;
        let scale = window.scale_factor().map_err(|error| error.to_string())?;
        let top = (top * scale).round().clamp(0.0, f64::from(full.height)) as u32;
        let show_page = visible && top < full.height;
        let chrome_height = if show_page { top.max(1) } else { full.height.max(1) };
        let chrome = app.get_webview("main").ok_or("Browser chrome webview missing")?;
        chrome.set_bounds(tauri::Rect {
            position: tauri::PhysicalPosition::new(0, 0).into(),
            size: tauri::PhysicalSize::new(full.width.max(1), chrome_height).into(),
        }).map_err(|error| error.to_string())?;
        app.state::<servo_embed::ServoBridge>()
            .send(servo_embed::Control::Layout {
                top, width: full.width, height: full.height.saturating_sub(top),
                visible: show_page,
            })?;
    }
    #[cfg(not(all(feature = "servo-embed", target_os = "windows")))]
    let _ = (app, top, visible);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .setup(|app| {
            let (sender, receiver) = mpsc::channel();
            let data_path = app.path().app_local_data_dir()?.join("browser-data.json");
            let browser_data = Arc::new(BrowserDataStore::load(data_path));
            #[cfg(all(feature = "servo-embed", target_os = "windows"))]
            {
                let bridge = servo_embed::start(app.handle(), sender.clone())
                    .map_err(std::io::Error::other)?;
                app.manage(bridge);
            }
            app.manage(EngineState {
                commands: sender,
                browser_data: browser_data.clone(),
            });

            let engine_app = app.handle().clone();
            std::thread::Builder::new()
                .name("nova-engine".into())
                // The old V8 renderer may require deep stacks. In native
                // Servo mode this thread only handles browser metadata/IPC.
                .stack_size(if cfg!(all(feature = "servo-embed", target_os = "windows")) {
                    8 * 1024 * 1024
                } else {
                    256 * 1024 * 1024
                })
                .spawn(move || engine_thread(engine_app, browser_data, receiver))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            browser_frontend_ready,
            browser_navigate,
            browser_search_with_provider,
            browser_dismiss_search_notice,
            browser_new_tab,
            browser_reopen_tab,
            browser_close_tab,
            browser_activate_tab,
            browser_back,
            browser_forward,
            browser_reload,
            browser_resize,
            browser_click,
            browser_scroll,
            browser_type,
            browser_toggle_bookmark,
            browser_is_bookmarked,
            browser_get_bookmarks,
            browser_remove_bookmark,
            browser_clear_bookmarks,
            browser_get_history,
            browser_clear_history,
            browser_get_settings,
            browser_save_settings,
            browser_get_engine_mode,
            browser_surface_layout
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                #[cfg(all(feature = "servo-embed", target_os = "windows"))]
                if let Some(bridge) = window.app_handle().try_state::<servo_embed::ServoBridge>() {
                    let _ = bridge.send(servo_embed::Control::Shutdown);
                }
                if let Some(state) = window.app_handle().try_state::<EngineState>() {
                    let _ = state.commands.send(Command::Shutdown);
                }
            }
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
