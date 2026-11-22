# NovaBrowser native Servo embedding: Windows manual validation

The **existing NovaBrowser** browser chrome, tab manager, Home, Settings,
bookmarks, history, session persistence and English/Vietnamese interface are
kept. `servo-embed` integrates native Servo into the **same window**.

## Implemented behind the Windows-only feature

- `crates/nova-desktop/src/servo_embed.rs`: Windows child HWND created
  beneath the Tauri window, owning a dedicated winit event loop, Servo 0.8
  engine, GPU rendering context and independent WebView handle per tab.
- `crates/nova-desktop/src/main.rs`: routes navigation, Back/Forward,
  Reload, Select, Close and URL/title/history through browser-owned commands.
- `crates/nova-desktop/ui/app.js`: computes the actual local content top,
  clips trusted WebView2 browser chrome above native Servo webpage pixels,
  hides the child surface for Home/Settings/bookmark/history panels and
  disables synthetic JavaScript mouse/keyboard events for Servo.
- Navigation requests are restricted to HTTP(S), with limited internal
  `about:` documents permitted by Servo. Local trusted browser routes
  `nova://home` and `nova://settings` remain in NovaBrowser.

**This code has not been built or run on Windows yet.** Feature APIs,
native dependencies, GPU child-window compositing and keyboard/IME remain
verification risks. Default NovaBrowser behavior is not changed.

## Local compile and run

```cmd
git pull origin main
rustup update stable
cargo check -p nova-desktop --features servo-embed
cargo test -p nova-desktop --features servo-embed
cargo run -p nova-desktop --release --features servo-embed
```

Compile on Windows 10/11 x64 with Visual Studio C++ Build Tools and Windows
SDK. Review any Cargo.lock changes. No GitHub Actions.

For the existing default browser build, simply use `run-nova.cmd`.

## Manual acceptance checklist

1. Start at Home. Confirm full NovaBrowser toolbar, bookmarks, tabs,
   Settings and internal UI are visible without a child HWND covering them.
2. Visit `https://example.org/`. Website must appear below the real
   toolbar in **the same window**, with clickable links and scroll.
3. Resize, maximize, restore and change display scaling. The native page
   must never cover Chrome, tabs, window controls, or Settings.
4. Open/close Home, Settings, history and bookmarks while on an external
   tab. Local panels must reclaim the full WebView2 UI area.
5. Open 3 tabs and switch, Back/Forward, Reload and close one. Verify
   URL, title, history and active-content state remain consistent.
6. Try Google, Wikipedia, Unicode text, focus, forms, native keyboard,
   IME, links and blocked privileged `file:`/custom-scheme navigation.
7. Restart and verify bookmarks, history, language, preferences and saved
   sessions remain compatible with the existing NovaBrowser profile.
8. Compare actual complete process-tree memory (PrivateMB/WorkingSetMB)
   against the normal build at identical pages and tab counts.

Known unresolved tasks: reliable native IME/accelerators while child HWND
has focus, downloads/permissions/popups, GPU layer ordering on all Windows
configurations, site isolation/security auditing and resource optimization.
The optional build still constructs legacy Obscura page state; it does not
yet demonstrate low RAM consumption or an entirely Servo-only process.

## Reference APIs

- https://doc.servo.org/servo/
- https://doc.servo.org/servo/trait.WebViewDelegate.html
- https://docs.rs/winit/latest/winit/window/struct.WindowAttributes.html
- https://v2.tauri.app/reference/rust/tauri/struct.Webview.html
