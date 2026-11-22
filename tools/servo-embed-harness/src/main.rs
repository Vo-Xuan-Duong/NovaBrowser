#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Non-product test harness for evaluating Servo embedding in NovaBrowser.
/// The native menu remains accessible when the single Servo webview navigates
/// away from our trusted local Home page. No untrusted page may invoke Rust.
#[path = "../../../crates/nova-desktop/src/navigation.rs"]
mod navigation;

use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{Manager, State, Url};

type ServoRuntime = tauri_runtime_servo::Servo<tauri::EventLoopMessage>;

struct BrowserState {
    home_url: Url,
}

fn target_url(address: &str, search_engine: &str, home: &Url) -> Result<Url, String> {
    // Reuse the actual NovaBrowser address/search normalization and tests.
    if !matches!(search_engine, "google" | "bing" | "duckduckgo") {
        return Err("Unsupported search engine".into());
    }
    let normalized = navigation::normalize_address_with_search(address, search_engine)
        .map_err(str::to_string)?;
    if normalized == "nova://home" {
        return Ok(home.clone());
    }
    let url = Url::parse(&normalized).map_err(|_| "Invalid URL".to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Only HTTP and HTTPS pages may be opened".into());
    }
    Ok(url)
}

#[tauri::command]
fn servo_navigate(
    webview: tauri::WebviewWindow<ServoRuntime>,
    state: State<'_, BrowserState>,
    address: String,
    search_engine: String,
) -> Result<(), String> {
    // Even if a remote site somehow has access to the invoke bridge, reject
    // command calls unless the current document is the bundled Home page.
    let current = webview.url().map_err(|error| error.to_string())?;
    if current != state.home_url {
        return Err("Navigation commands are only accepted from the local Home page".into());
    }
    let url = target_url(&address, &search_engine, &state.home_url)?;
    webview.navigate(url).map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::<ServoRuntime>::new()
        // Servo does not expose custom-protocol request bodies, so the normal
        // Tauri invoke transport does not work without this replacement.
        .invoke_system(tauri_runtime_servo::INVOKE_SYSTEM_SCRIPT)
        .setup(|app| {
            let webview = app.get_webview_window("main")
                .ok_or_else(|| std::io::Error::other("Servo webview not created"))?;
            let home_url = webview.url()?;
            app.manage(BrowserState { home_url });

            // Servo's Tauri runtime currently supports one webview per native
            // window. Native menu items are still usable on remote pages;
            // there is no unsafe attempt to embed Google in an iframe.
            let home = MenuItemBuilder::with_id("nova-home", "Nova Home / Address")
                .accelerator("Ctrl+L").build(app)?;
            let back = MenuItemBuilder::with_id("nova-back", "Back")
                .accelerator("Alt+Left").build(app)?;
            let forward = MenuItemBuilder::with_id("nova-forward", "Forward")
                .accelerator("Alt+Right").build(app)?;
            let reload = MenuItemBuilder::with_id("nova-reload", "Reload")
                .accelerator("Ctrl+R").build(app)?;
            let nav = SubmenuBuilder::new(app, "Navigate")
                .items(&[&home, &back, &forward, &reload])
                .build()?;
            let menu = MenuBuilder::new(app).item(&nav).build()?;
            app.set_menu(menu)?;
            app.on_menu_event(|app, event| {
                let Some(webview) = app.get_webview_window("main") else { return };
                let result = if event.id() == "nova-home" {
                    app.try_state::<BrowserState>()
                        .map(|state| webview.navigate(state.home_url.clone()))
                        .unwrap_or(Ok(()))
                } else if event.id() == "nova-back" {
                    webview.eval("window.history.back()")
                } else if event.id() == "nova-forward" {
                    webview.eval("window.history.forward()")
                } else if event.id() == "nova-reload" {
                    webview.reload()
                } else {
                    return;
                };
                if let Err(error) = result {
                    eprintln!("NovaBrowser navigation error: {error}");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![servo_navigate])
        .run(tauri::generate_context!())
        .expect("NovaBrowser failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unicode_and_provider_choice() {
        let home = Url::parse("http://tauri.localhost/index.html").unwrap();
        let query = target_url("C++ tiếng Việt", "google", &home).unwrap();
        assert_eq!(query.host_str(), Some("www.google.com"));
        assert_eq!(
            query.query_pairs().find(|(key, _)| key == "q").unwrap().1,
            "C++ tiếng Việt"
        );
        let bing = target_url("rust language", "bing", &home).unwrap();
        assert_eq!(bing.host_str(), Some("www.bing.com"));
        assert_eq!(target_url("nova://home", "google", &home).unwrap(), home);
    }

    #[test]
    fn rejects_privileged_urls_and_unknown_providers() {
        let home = Url::parse("http://tauri.localhost/index.html").unwrap();
        for address in ["file:///C:/Windows", "javascript:alert(1)", "data:text/html,x", "ftp://example.org"] {
            assert!(target_url(address, "google", &home).is_err(), "{address}");
        }
        assert!(target_url("test", "unknown", &home).is_err());
    }
}
