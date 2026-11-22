mod input;
mod navigation;
mod storage;
mod viewport;

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

use navigation::{normalize_address, NavigationHistory};
use storage::{Bookmark, BrowserDataStore, BrowserSettings, Visit};
use viewport::Viewport;

const MAX_TABS: usize = 8;

enum Command {
    Navigate(String),
    NewTab,
    CloseTab(String),
    ActivateTab(String),
    Back,
    Forward,
    Reload,
    Resize(f32, f32),
    Click(f32, f32),
    Scroll(f32, f32, f32, f32),
    Type(String),
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
    page: Page,
    history: NavigationHistory,
    last_frame: Option<u64>,
}

impl BrowserTab {
    fn new(id: String, context: Arc<BrowserContext>, viewport: Viewport) -> Self {
        let mut page = Page::new(id.clone(), context);
        page.set_viewport(viewport.css_size());
        Self {
            id,
            title: "New tab".to_string(),
            address: String::new(),
            page,
            history: NavigationHistory::default(),
            last_frame: None,
        }
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

fn publish_buttons(app: &AppHandle, tab: &BrowserTab) {
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
    report(app, None, format!("Loading {target}"), true);
    let result: Result<(), String> = runtime.block_on(async {
        tab.page
            .navigate_with_wait(target, WaitUntil::Load)
            .await
            .map_err(|error| error.to_string())?;
        tab.page.settle(180).await;
        tab.page.prepare_screenshot_resources(300).await;
        Ok(())
    });

    match result {
        Ok(()) => {
            let url = tab.page.url_string();
            let title = document_title(&mut tab.page);
            if let Some(index) = history_index {
                tab.history.select(index);
            } else {
                tab.history.record(url.clone());
            }
            tab.address = url.clone();
            tab.title = if title.is_empty() { "Untitled page".into() } else { title.clone() };
            tab.last_frame = None;
            let _ = browser_data.record_visit(title.clone(), url.clone());
            let message = if title.is_empty() {
                "Page loaded".to_string()
            } else {
                format!("Page loaded: {title}")
            };
            report(app, Some(url), message, false);
            publish_tab_title(app, &title);
            publish_buttons(app, tab);
            capture(&tab.page, viewport, app, &mut tab.last_frame);
        }
        Err(error) => report(app, None, format!("Navigation error: {error}"), false),
    }
}

/// Observe same-document navigation where the engine has updated its URL.
fn sync_history(tab: &mut BrowserTab, app: &AppHandle, browser_data: &BrowserDataStore) {
    let url = tab.page.url_string();
    if (url.starts_with("https://") || url.starts_with("http://"))
        && tab.history.current() != Some(url.as_str())
    {
        tab.history.record(url.clone());
        let title = document_title(&mut tab.page);
        tab.address = url.clone();
        tab.title = if title.is_empty() { "Untitled page".into() } else { title.clone() };
        let _ = browser_data.record_visit(title.clone(), url.clone());
        let message = if title.is_empty() {
            "Page loaded".to_string()
        } else {
            format!("Page loaded: {title}")
        };
        report(app, Some(url), message, false);
        publish_tab_title(app, &title);
        publish_buttons(app, tab);
    }
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
    tab.last_frame = None;
    capture(&tab.page, viewport, app, &mut tab.last_frame);
}

fn engine_thread(
    app: AppHandle,
    browser_data: Arc<BrowserDataStore>,
    commands: mpsc::Receiver<Command>,
) {
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

    // All page isolates stay on this thread. Tabs share cookies and the HTTP
    // client through BrowserContext while keeping their DOM and JS realm apart.
    let context = Arc::new(BrowserContext::new("nova-default".to_owned()));
    let mut viewport = Viewport::DEFAULT;
    let first_tab = BrowserTab::new("nova-tab-1".into(), context.clone(), viewport);
    let mut tabs = vec![first_tab];
    let mut active_id = "nova-tab-1".to_string();
    let mut next_tab_id = 2_u64;
    publish_tab_list(&app, &tabs, &active_id);

    loop {
        match commands.recv_timeout(Duration::from_millis(500)) {
            Ok(Command::Navigate(raw)) => match normalize_address(&raw) {
                Ok(url) => {
                    if let Some(index) = tabs.iter().position(|tab| tab.id == active_id) {
                        publish_tab_list(&app, &tabs, &active_id);
                        navigate(&mut tabs[index], &runtime, viewport, &app, &browser_data, &url, None);
                        publish_tab_list(&app, &tabs, &active_id);
                    }
                }
                Err(error) => report(&app, None, error.to_string(), false),
            },
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
                let homepage = browser_data.settings().homepage;
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    navigate(
                        tab,
                        &runtime,
                        viewport,
                        &app,
                        &browser_data,
                        &homepage,
                        None,
                    );
                }
                publish_tab_list(&app, &tabs, &active_id);
            }
            Ok(Command::CloseTab(id)) => {
                if tabs.len() > 1 {
                    if let Some(closing_index) = tabs.iter().position(|tab| tab.id == id) {
                        let old_active_index = tabs.iter().position(|tab| tab.id == active_id).unwrap_or(0);
                        let closing_active = id == active_id;
                        let closed_tab = tabs.remove(closing_index);
                        runtime.block_on(async { drop(closed_tab) });
                        if closing_active {
                            let next_index = old_active_index.saturating_sub(1).min(tabs.len() - 1);
                            active_id = tabs[next_index].id.clone();
                            publish_active_tab(&app, &mut tabs, &active_id, viewport);
                        } else {
                            publish_tab_list(&app, &tabs, &active_id);
                        }
                    }
                }
            }
            Ok(Command::ActivateTab(id)) => {
                if tabs.iter().any(|tab| tab.id == id) && active_id != id {
                    active_id = id;
                    publish_active_tab(&app, &mut tabs, &active_id, viewport);
                }
            }
            Ok(Command::Back) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if let Some((index, url)) = tab.history.previous() {
                        navigate(tab, &runtime, viewport, &app, &browser_data, &url, Some(index));
                        publish_tab_list(&app, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Forward) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if let Some((index, url)) = tab.history.next() {
                        navigate(tab, &runtime, viewport, &app, &browser_data, &url, Some(index));
                        publish_tab_list(&app, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Reload) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if let Some(url) = tab.history.current().map(str::to_owned) {
                        navigate(tab, &runtime, viewport, &app, &browser_data, &url, None);
                        publish_tab_list(&app, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Resize(width, height)) => {
                if let Some(next) = Viewport::from_window(width, height) {
                    if next.width != viewport.width || next.height != viewport.height {
                        viewport = next;
                        for tab in &mut tabs {
                            tab.page.set_viewport(next.css_size());
                            tab.last_frame = None;
                        }
                        if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                            capture(&tab.page, next, &app, &mut tab.last_frame);
                        }
                    }
                }
            }
            Ok(Command::Click(x, y)) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if !tab.address.is_empty() {
                        input::click(&mut tab.page, viewport, x, y);
                        runtime.block_on(tab.page.settle(60));
                        sync_history(tab, &app, &browser_data);
                        capture(&tab.page, viewport, &app, &mut tab.last_frame);
                        publish_tab_list(&app, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Scroll(x, y, dx, dy)) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if !tab.address.is_empty() {
                        input::scroll(&mut tab.page, viewport, x, y, dx, dy);
                        runtime.block_on(tab.page.settle(40));
                        capture(&tab.page, viewport, &app, &mut tab.last_frame);
                    }
                }
            }
            Ok(Command::Type(key)) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if !tab.address.is_empty() {
                        input::key(&mut tab.page, &key);
                        runtime.block_on(tab.page.settle(40));
                        sync_history(tab, &app, &browser_data);
                        capture(&tab.page, viewport, &app, &mut tab.last_frame);
                        publish_tab_list(&app, &tabs, &active_id);
                    }
                }
            }
            Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(tab) = tabs.iter_mut().find(|tab| tab.id == active_id) {
                    if !tab.address.is_empty() {
                        runtime.block_on(async {
                            tab.page.settle(50).await;
                            tab.page.prepare_screenshot_resources(75).await;
                        });
                        sync_history(tab, &app, &browser_data);
                        capture(&tab.page, viewport, &app, &mut tab.last_frame);
                        publish_tab_list(&app, &tabs, &active_id);
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
fn browser_navigate(state: State<'_, EngineState>, address: String) -> Result<(), String> {
    send_command(&state, Command::Navigate(address))
}

#[tauri::command]
fn browser_new_tab(state: State<'_, EngineState>) -> Result<(), String> {
    send_command(&state, Command::NewTab)
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
) -> Result<BrowserSettings, String> {
    if theme != "light" && theme != "dark" {
        return Err("Choose either the light or dark appearance".to_string());
    }
    let homepage = normalize_address(&homepage).map_err(|error| error.to_string())?;
    let settings = BrowserSettings { theme, homepage };
    state.browser_data.save_settings(settings.clone())?;
    Ok(settings)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .setup(|app| {
            let (sender, receiver) = mpsc::channel();
            let data_path = app.path().app_local_data_dir()?.join("browser-data.json");
            let browser_data = Arc::new(BrowserDataStore::load(data_path));
            app.manage(EngineState {
                commands: sender,
                browser_data: browser_data.clone(),
            });

            let engine_app = app.handle().clone();
            std::thread::Builder::new()
                .name("nova-engine".into())
                .stack_size(256 * 1024 * 1024)
                .spawn(move || engine_thread(engine_app, browser_data, receiver))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            browser_navigate,
            browser_new_tab,
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
            browser_get_history,
            browser_clear_history,
            browser_get_settings,
            browser_save_settings
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                if let Some(state) = window.app_handle().try_state::<EngineState>() {
                    let _ = state.commands.send(Command::Shutdown);
                }
            }
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
