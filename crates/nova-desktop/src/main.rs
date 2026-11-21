mod navigation;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use obscura_browser::lifecycle::WaitUntil;
use obscura_browser::{BrowserContext, Page};
use slint::{ComponentHandle, Image, Weak};

use navigation::{normalize_address, NavigationHistory};

slint::include_modules!();

/// Fixed CSS-pixel viewport for the first screenshot-based prototype.
/// A resize-aware native compositor is a separate milestone.
const VIEWPORT: (f32, f32) = (1200.0, 740.0);

enum Command {
    Navigate(String),
    Back,
    Forward,
    Reload,
    Click(f32, f32),
    Scroll(f32, f32),
    Key(String),
    Shutdown,
}

fn report(ui: &Weak<NovaWindow>, address: Option<String>, message: String, loading: bool) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(window) = ui.upgrade() {
            if let Some(address) = address {
                window.set_address(address.into());
            }
            window.set_status(message.into());
            window.set_loading(loading);
        }
    });
}

fn publish_frame(ui: &Weak<NovaWindow>, bytes: Vec<u8>) {
    let ui = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(window) = ui.upgrade() {
            if let Ok(image) = Image::load_from_data(&bytes, Some("png")) {
                window.set_page_image(image);
            }
        }
    });
}

fn capture(page: &Page, ui: &Weak<NovaWindow>, last_frame: &mut Option<u64>) {
    let Some(png) = page.screenshot(VIEWPORT) else {
        return;
    };
    let mut hasher = DefaultHasher::new();
    png.hash(&mut hasher);
    let hash = hasher.finish();
    if *last_frame == Some(hash) {
        return;
    }
    *last_frame = Some(hash);
    publish_frame(ui, png);
}

fn load(page: &mut Page, runtime: &tokio::runtime::Runtime, target: &str) -> Result<(), String> {
    runtime.block_on(async {
        page.navigate_with_wait(target, WaitUntil::Load)
            .await
            .map_err(|err| err.to_string())?;
        page.settle(180).await;
        page.prepare_screenshot_resources(350).await;
        Ok(())
    })
}

fn title(page: &mut Page) -> String {
    let value = page.evaluate("document.title");
    let title = value.as_str().unwrap_or("").trim();
    if title.is_empty() {
        "Page loaded".to_string()
    } else {
        format!("Page loaded: {title}")
    }
}

fn open(
    page: &mut Page,
    runtime: &tokio::runtime::Runtime,
    ui: &Weak<NovaWindow>,
    history: &mut NavigationHistory,
    last_frame: &mut Option<u64>,
    target: &str,
    history_index: Option<usize>,
) {
    report(ui, None, format!("Loading {target}"), true);
    match load(page, runtime, target) {
        Ok(()) => {
            let url = page.url_string();
            if let Some(index) = history_index {
                history.select(index);
            } else {
                history.record(url.clone());
            }
            *last_frame = None;
            report(ui, Some(url), title(page), false);
            capture(page, ui, last_frame);
        }
        Err(err) => {
            report(ui, None, format!("Navigation failed: {err}"), false);
        }
    }
}

fn sync_url(page: &mut Page, ui: &Weak<NovaWindow>, history: &mut NavigationHistory) {
    let url = page.url_string();
    if url.starts_with("http") && history.current() != Some(url.as_str()) {
        history.record(url.clone());
        report(ui, Some(url), title(page), false);
    }
}

fn click(page: &mut Page, x: f32, y: f32) {
    if !x.is_finite() || !y.is_finite() {
        return;
    }
    let x = x.clamp(0.0, 1.0) * VIEWPORT.0;
    let y = y.clamp(0.0, 1.0) * VIEWPORT.1;
    let script = format!(
        "(function() {{ const el = document.elementFromPoint({x:.2}, {y:.2}); \
         if (!el) return false; \
         if (typeof el.focus === 'function') el.focus(); \
         if (typeof el.click === 'function') el.click(); \
         return true; }})()"
    );
    let _ = page.evaluate(&script);
}

/// This is provisional DOM-level text injection, not trusted keyboard/IME input.
fn type_into_focused_element(page: &mut Page, input: &str) {
    let payload = serde_json::to_string(input).unwrap_or_else(|_| "\"\"".into());
    let script = format!(
        r#"(function() {{
            const el = document.activeElement;
            if (!el || (el.tagName !== "INPUT" && el.tagName !== "TEXTAREA")) return false;
            const old = String(el.value || "");
            const start = typeof el.selectionStart === "number" ? el.selectionStart : old.length;
            const end = typeof el.selectionEnd === "number" ? el.selectionEnd : start;
            const input = {payload};
            const erase = input === "__NOVA_BACKSPACE__";
            if (!erase && (input.length !== 1 || input.charCodeAt(0) < 32)) return false;
            const begin = erase && start === end ? Math.max(0, start - 1) : start;
            el.value = old.slice(0, begin) + (erase ? "" : input) + old.slice(end);
            const pos = begin + (erase ? 0 : input.length);
            if (typeof el.setSelectionRange === "function") el.setSelectionRange(pos, pos);
            el.dispatchEvent(new Event("input", {{ bubbles: true }}));
            return true;
        }})()"#
    );
    let _ = page.evaluate(&script);
}

fn engine_loop(ui: Weak<NovaWindow>, rx: mpsc::Receiver<Command>) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            report(&ui, None, format!("Cannot start engine: {err}"), false);
            return;
        }
    };

    // The page and its V8 isolate are created and used on this single thread.
    // Keep the SSRF restriction enabled; local-network browsing needs a
    // narrowly scoped explicit-navigation exception in a future milestone.
    let context = Arc::new(BrowserContext::new("nova-default".to_string()));
    let mut page = Page::new("nova-tab-1".to_string(), context);
    page.set_viewport(VIEWPORT);
    let mut history = NavigationHistory::default();
    let mut last_frame = None;

    loop {
        match rx.recv_timeout(Duration::from_millis(750)) {
            Ok(Command::Navigate(raw)) => match normalize_address(&raw) {
                Ok(target) => open(
                    &mut page, &runtime, &ui, &mut history, &mut last_frame, &target, None,
                ),
                Err(err) => report(&ui, None, err, false),
            },
            Ok(Command::Back) => {
                if let Some((index, url)) = history.previous() {
                    open(&mut page, &runtime, &ui, &mut history, &mut last_frame, &url, Some(index));
                }
            }
            Ok(Command::Forward) => {
                if let Some((index, url)) = history.next() {
                    open(&mut page, &runtime, &ui, &mut history, &mut last_frame, &url, Some(index));
                }
            }
            Ok(Command::Reload) => {
                if let Some(url) = history.current().map(str::to_owned) {
                    open(&mut page, &runtime, &ui, &mut history, &mut last_frame, &url, None);
                }
            }
            Ok(Command::Click(x, y)) => {
                if history.current().is_some() {
                    click(&mut page, x, y);
                    runtime.block_on(page.settle(80));
                    sync_url(&mut page, &ui, &mut history);
                    capture(&page, &ui, &mut last_frame);
                }
            }
            Ok(Command::Scroll(dx, dy)) => {
                if history.current().is_some() && dx.is_finite() && dy.is_finite() {
                    let _ = page.evaluate(&format!("window.scrollBy({}, {})", -dx, -dy));
                    runtime.block_on(page.settle(45));
                    capture(&page, &ui, &mut last_frame);
                }
            }
            Ok(Command::Key(value)) => {
                if history.current().is_some() {
                    type_into_focused_element(&mut page, &value);
                    runtime.block_on(page.settle(45));
                    capture(&page, &ui, &mut last_frame);
                }
            }
            Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if history.current().is_some() {
                    runtime.block_on(async {
                        page.settle(75).await;
                        page.prepare_screenshot_resources(75).await;
                    });
                    sync_url(&mut page, &ui, &mut history);
                    capture(&page, &ui, &mut last_frame);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = NovaWindow::new()?;
    let (tx, rx) = mpsc::channel();
    let weak = window.as_weak();

    let worker = std::thread::Builder::new()
        .name("nova-browser-engine".into())
        .stack_size(512 * 1024 * 1024)
        .spawn(move || engine_loop(weak, rx))?;

    let sender = tx.clone();
    window.on_navigate(move |url| {
        let _ = sender.send(Command::Navigate(url.to_string()));
    });
    let sender = tx.clone();
    window.on_back(move || {
        let _ = sender.send(Command::Back);
    });
    let sender = tx.clone();
    window.on_forward(move || {
        let _ = sender.send(Command::Forward);
    });
    let sender = tx.clone();
    window.on_reload(move || {
        let _ = sender.send(Command::Reload);
    });
    let sender = tx.clone();
    window.on_page_click(move |x, y| {
        let _ = sender.send(Command::Click(x, y));
    });
    let sender = tx.clone();
    window.on_page_scroll(move |x, y| {
        let _ = sender.send(Command::Scroll(x, y));
    });
    let sender = tx.clone();
    window.on_page_key(move |value| {
        let _ = sender.send(Command::Key(value.to_string()));
    });

    let _ = tx.send(Command::Navigate("https://example.com".into()));
    let result = window.run();
    let _ = tx.send(Command::Shutdown);
    let _ = worker.join();
    result?;
    Ok(())
}
