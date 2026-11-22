mod input;
mod navigation;
mod viewport;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use obscura_browser::lifecycle::WaitUntil;
use obscura_browser::{BrowserContext, Page};
use slint::{ComponentHandle, Image, Weak};

use navigation::{normalize_address, NavigationHistory};
use viewport::Viewport;

slint::include_modules!();

enum Command {
    Navigate(String),
    Back,
    Forward,
    Reload,
    Resize(f32, f32),
    Click(f32, f32),
    Scroll(f32, f32, f32, f32),
    Type(String),
    Shutdown,
}

fn report(ui: &Weak<NovaWindow>, address: Option<String>, status: String, loading: bool) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(window) = ui.upgrade() {
            if let Some(address) = address {
                window.set_address(address.into());
            }
            window.set_status(status.into());
            window.set_loading(loading);
        }
    });
}

fn publish_buttons(ui: &Weak<NovaWindow>, history: &NavigationHistory) {
    let can_back = history.previous().is_some();
    let can_forward = history.next().is_some();
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(window) = ui.upgrade() {
            window.set_can_back(can_back);
            window.set_can_forward(can_forward);
        }
    });
}

fn publish_frame(ui: &Weak<NovaWindow>, png: Vec<u8>) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(window) = ui.upgrade() {
            if let Ok(image) = Image::load_from_data(&png, Some("png")) {
                window.set_page_image(image);
            }
        }
    });
}

/// Frame comparisons avoid redundant updates on visually static pages.
fn capture(page: &Page, viewport: Viewport, ui: &Weak<NovaWindow>, hash: &mut Option<u64>) {
    let Some(png) = page.screenshot(viewport.css_size()) else {
        return;
    };
    let mut hasher = DefaultHasher::new();
    png.hash(&mut hasher);
    let fingerprint = hasher.finish();
    if *hash == Some(fingerprint) {
        return;
    }
    *hash = Some(fingerprint);
    publish_frame(ui, png);
}

fn loaded_title(page: &mut Page) -> String {
    let title = page.evaluate("document.title");
    let title = title.as_str().unwrap_or("").trim();
    if title.is_empty() {
        "Page loaded".to_string()
    } else {
        format!("Page loaded: {title}")
    }
}

fn navigate(
    page: &mut Page,
    runtime: &tokio::runtime::Runtime,
    viewport: Viewport,
    ui: &Weak<NovaWindow>,
    history: &mut NavigationHistory,
    last_frame: &mut Option<u64>,
    target: &str,
    history_index: Option<usize>,
) {
    report(ui, None, format!("Loading {target}"), true);

    let result: Result<(), String> = runtime.block_on(async {
        page.navigate_with_wait(target, WaitUntil::Load)
            .await
            .map_err(|e| e.to_string())?;
        page.settle(180).await;
        page.prepare_screenshot_resources(300).await;
        Ok(())
    });

    match result {
        Ok(()) => {
            let url = page.url_string();
            if let Some(index) = history_index {
                history.select(index);
            } else {
                history.record(url.clone());
            }
            *last_frame = None;
            report(ui, Some(url), loaded_title(page), false);
            publish_buttons(ui, history);
            capture(page, viewport, ui, last_frame);
        }
        Err(e) => {
            report(ui, None, format!("Navigation error: {e}"), false);
        }
    }
}

/// Observe same-document navigation where the engine has updated its URL.
fn sync_history(page: &mut Page, ui: &Weak<NovaWindow>, history: &mut NavigationHistory) {
    let url = page.url_string();
    if (url.starts_with("https://") || url.starts_with("http://"))
        && history.current() != Some(url.as_str())
    {
        history.record(url.clone());
        report(ui, Some(url), loaded_title(page), false);
        publish_buttons(ui, history);
    }
}

fn engine_thread(ui: Weak<NovaWindow>, commands: mpsc::Receiver<Command>) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            report(&ui, None, format!("Engine startup failed: {e}"), false);
            return;
        }
    };

    // V8 and Page stay on this thread. Network / private address protections
    // remain at their engine defaults; desktop UI does not grant exceptions.
    let context = Arc::new(BrowserContext::new("nova-default".to_owned()));
    let mut page = Page::new("nova-first-tab".to_owned(), context);
    let mut viewport = Viewport::DEFAULT;
    page.set_viewport(viewport.css_size());

    let mut history = NavigationHistory::default();
    let mut last_frame = None;

    loop {
        match commands.recv_timeout(Duration::from_millis(500)) {
            Ok(Command::Navigate(raw)) => match normalize_address(&raw) {
                Ok(url) => navigate(
                    &mut page, &runtime, viewport, &ui, &mut history,
                    &mut last_frame, &url, None,
                ),
                Err(error) => report(&ui, None, error.to_string(), false),
            },
            Ok(Command::Back) => {
                if let Some((index, url)) = history.previous() {
                    navigate(
                        &mut page, &runtime, viewport, &ui, &mut history,
                        &mut last_frame, &url, Some(index),
                    );
                }
            }
            Ok(Command::Forward) => {
                if let Some((index, url)) = history.next() {
                    navigate(
                        &mut page, &runtime, viewport, &ui, &mut history,
                        &mut last_frame, &url, Some(index),
                    );
                }
            }
            Ok(Command::Reload) => {
                if let Some(url) = history.current().map(str::to_owned) {
                    navigate(
                        &mut page, &runtime, viewport, &ui, &mut history,
                        &mut last_frame, &url, None,
                    );
                }
            }
            Ok(Command::Resize(width, height)) => {
                if let Some(next) = Viewport::from_window(width, height) {
                    if next != viewport {
                        viewport = next;
                        page.set_viewport(viewport.css_size());
                        last_frame = None;
                        if history.current().is_some() {
                            capture(&page, viewport, &ui, &mut last_frame);
                        }
                    }
                }
            }
            Ok(Command::Click(x, y)) => {
                if history.current().is_some() {
                    input::click(&mut page, viewport, x, y);
                    runtime.block_on(page.settle(60));
                    sync_history(&mut page, &ui, &mut history);
                    capture(&page, viewport, &ui, &mut last_frame);
                }
            }
            Ok(Command::Scroll(x, y, dx, dy)) => {
                if history.current().is_some() {
                    input::scroll(&mut page, viewport, x, y, dx, dy);
                    runtime.block_on(page.settle(40));
                    capture(&page, viewport, &ui, &mut last_frame);
                }
            }
            Ok(Command::Type(key)) => {
                if history.current().is_some() {
                    input::key(&mut page, &key);
                    runtime.block_on(page.settle(40));
                    sync_history(&mut page, &ui, &mut history);
                    capture(&page, viewport, &ui, &mut last_frame);
                }
            }
            Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if history.current().is_some() {
                    runtime.block_on(async {
                        page.settle(50).await;
                        page.prepare_screenshot_resources(75).await;
                    });
                    sync_history(&mut page, &ui, &mut history);
                    capture(&page, viewport, &ui, &mut last_frame);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = NovaWindow::new()?;
    let (tx, rx) = mpsc::channel();

    let engine = std::thread::Builder::new()
        .name("nova-engine".into())
        .stack_size(256 * 1024 * 1024)
        .spawn({
            let ui = window.as_weak();
            move || engine_thread(ui, rx)
        })?;

    let sender = tx.clone();
    window.on_navigate(move |url| {
        let _ = sender.send(Command::Navigate(url.to_string()));
    });
    let sender = tx.clone();
    window.on_back(move || { let _ = sender.send(Command::Back); });
    let sender = tx.clone();
    window.on_forward(move || { let _ = sender.send(Command::Forward); });
    let sender = tx.clone();
    window.on_reload(move || { let _ = sender.send(Command::Reload); });
    let sender = tx.clone();
    window.on_viewport_resized(move |width, height| {
        let _ = sender.send(Command::Resize(width, height));
    });
    let sender = tx.clone();
    window.on_page_click(move |x, y| {
        let _ = sender.send(Command::Click(x, y));
    });
    let sender = tx.clone();
    window.on_page_scroll(move |x, y, dx, dy| {
        let _ = sender.send(Command::Scroll(x, y, dx, dy));
    });
    let sender = tx.clone();
    window.on_page_key(move |key| {
        let _ = sender.send(Command::Type(key.to_string()));
    });

    let _ = tx.send(Command::Resize(
        window.get_viewport_width(),
        window.get_viewport_height(),
    ));
    let _ = tx.send(Command::Navigate("https://example.com".to_string()));

    let outcome = window.run();
    let _ = tx.send(Command::Shutdown);

    // In this prototype a blocked navigation should not freeze application
    // shutdown. A finished engine thread can be reaped; a busy one is detached
    // and terminated when the process exits.
    if engine.is_finished() {
        let _ = engine.join();
    }
    outcome?;
    Ok(())
}
