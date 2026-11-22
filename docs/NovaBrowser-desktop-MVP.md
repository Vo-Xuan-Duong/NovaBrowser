# NovaBrowser desktop MVP

## Intent

Build an independent native desktop browser shell on top of the Rust engine
retained from Obscura. Retain the existing engine crates and add NovaBrowser
components alongside them.

## Native window / page pipeline

```text
Tauri window (HTML/CSS chrome, WebView2 on Windows)
    -> Tauri commands and events
        -> Engine thread (Tokio current_thread + V8)
            -> obscura-browser::Page (navigation / JS / DOM)
                -> screenshot(viewport) -> PNG frame event
```

The Tauri webview draws only NovaBrowser's application interface. Website
content continues to come from the Obscura engine as rendered PNG frames.
A single engine thread owns `Page` and its V8 isolate. The frontend sends
navigation and input commands, and receives status, title, history, and frame
events.

## Functionality and security

- Chrome-inspired custom title bar, window controls, tab strip, toolbar and omnibox.
- Address bar: HTTP(S), hostname shortcuts and search terms.
- Browser navigation: Back, Forward, Reload, page URL and title.
- Local bookmarks and browsing history, stored in the app's local data folder.
- Local settings for light/dark appearance and the homepage used at startup and
  for new tabs.
- Clear browsing history without removing saved bookmarks.
- Provisional pointer events, scrolling and simple editable text inputs.
- Obscura's default SSRF restrictions remain enabled; localhost and private IP
  URLs may be blocked by the engine.
- `file:`, `javascript:` and other unsupported schemes are rejected from the
  browser's address bar.

This is not a secure multi-process browser. Do not treat this preview as a
production security boundary or a replacement for Chromium/Firefox.

## Known limitations

- Rendering uses PNG screenshots rather than an efficient GPU compositor.
- Up to eight tabs are supported. Each tab has a separate DOM/JS page state;
  tabs share cookies through one browser context.
- Site sandboxing and process isolation are not present.
- DOM-level synthetic input is incomplete, especially for IME, text editing,
  selection, drag/drop, complex frameworks and some form controls.
- Back/Forward are managed by the shell, not full engine session history.
- Animated/interactive content receives a periodic best-effort refresh.
- Site compatibility is limited by the current browser engine.
- Tauri's webview is used for browser chrome only; it does not render websites.

## Local validation

```powershell
cargo check -p nova-desktop
cargo test -p nova-desktop
cargo run -p nova-desktop --release
```

On Windows, Tauri uses Microsoft Edge WebView2 for the interface. WebView2 is
included with supported Windows versions or can be installed separately. The
Obscura engine still requires the dependencies needed to build V8.
