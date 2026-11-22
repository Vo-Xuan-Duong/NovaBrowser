# Embedding Servo inside NovaBrowser (Windows engineering plan)

This document concerns the **website engine of the existing NovaBrowser
application**, not a replacement Servo application. The product is the
browser chrome in `crates/nova-desktop/ui/` and the native Rust shell in
`crates/nova-desktop/src/`. It includes the tab strip, address bar, Home,
Settings, bookmarks, history, and English/Vietnamese localization.

## Current status (2026-11-22)

- The actual product is launched by `run-nova.cmd` or `cargo nova`.
- It still renders website content using Obscura's experimental CPU PNG
  frames and a synthetic DOM input adapter.
- A Windows-only native Servo child-window adapter is now available behind
  `servo-embed` in the existing NovaBrowser desktop. It is not yet compiled
  or tested on Windows and is not the default renderer.
- `crates/nova-desktop/src/engine_api.rs` now defines browser-owned tab
  identifiers, bounds, input events, page notifications and the renderer
  interface to support incremental replacement.
- `tools/servo-embed-harness/` is a **separate integration test tool**,
  never a release target; its one-webview demo cannot preserve the browser
  toolbar when browsing an external website.
- No Windows runtime build or measured RAM comparison has been completed in
  this phase.

## Why use Servo components?

Servo's embeddable `servo` crate provides a real HTML/CSS engine, DOM/JS
runtime, compositing and `WebView` API. It is a candidate to replace only
the website-content backend of NovaBrowser. Obscura-derived components
remain useful for app-owned URL normalization, persistent session data and
related utilities where they add value. Avoid running two complete JS/DOM
renderers on a single website.

The long-term goal is efficient interactive rendering; **Servo's total RAM
advantage has not been demonstrated**.

## Practical API integration path

The current Servo 0.8.0 embedding documentation describes:

1. `ServoBuilder` creating an instance of `Servo`, with a custom
   `EventLoopWaker` connected to the native UI event loop.
2. A `RenderingContext` for display/presentation.
3. `WebViewBuilder` creating an independent page handle for each tab.
4. `WebView::load`, `go_back`, `go_forward`, `reload`, `resize`,
   `notify_input_event` and `paint` implementing page operations.
5. `Servo::spin_event_loop` driving input, networking and page tasks.
6. `WebViewDelegate::notify_new_frame_ready` scheduling paints.

A Servo `WebView` is not `Send` or `Sync`; own page handles and the
associated rendering contexts on the appropriate event-loop thread. Do not
pass them across Tauri command-worker threads directly. Events delivered
to NovaBrowser's shell should contain browser-owned `TabId` values, URL,
title and history state, not leaked internal Servo handles.

`engine_api.rs` establishes the typed boundary. It is not an adapter and
does not yet implement any of these six Servo operations.

## Compositing blocker

The published `tauri-runtime-servo` 0.1.0 backend supports a single Servo
webview per native window. Replacing the NovaBrowser Tauri runtime with
that backend would make external pages replace the local chrome and break
the existing tab/Settings interface. **Do not do that.**

Evaluate either direct embedding of the `servo` crate using an application
owned rendering context and event loop, or extending an embeddable adapter
to support multiple independent composited WebViews and a persistent,
privileged local chrome surface. Prioritize retaining the existing
HTML/CSS UI and its interaction model over reimplementing it with a new
toolkit. Do not embed arbitrary sites as iframes.

The current native NovaBrowser UI uses WebView2 solely for browser chrome on
Windows. Removing its overhead is a further optimization, not a reason to
discard the working interface before Servo composition has been proven.

## Migration in safe increments

1. Build a single Servo `WebView` **within the existing NovaBrowser
   window**, inside a viewport below the real toolbar. Verify resizing,
   painting, focus and native click/scroll/keyboard input.
2. Route shell navigation, URL/title/history events through
   `engine_api.rs`. Keep `nova://home` and `nova://settings` local
   and privileged. Enforce HTTP(S)-only external site requests at the
   boundary.
3. Add WebView-per-tab ownership, selection/visibility, lazy creation and
   release of closed/inactive pages, without moving saved browser state
   into the engine.
4. Reconnect the pre-existing settings, search provider, language,
   bookmarks, history and session restore. Do not change the existing
   profile format without a tested migration.
5. Validate origin/IPC separation, permissions, downloads, IME, popups,
   clipboard, user navigation and per-site isolation.
6. Benchmark CPU and memory under identical Windows workloads; optimize
   cold launch, per-tab memory, background tabs and compositor reuse.
7. Once parity is demonstrated, retire obsolete full-renderer code without
   losing original license and attribution notices.

## Local tests

Current NovaBrowser:

```cmd
cargo check -p nova-desktop
cargo test -p nova-desktop
cargo run -p nova-desktop --release
```

Servo harness only:

```cmd
cargo check --manifest-path tools/servo-embed-harness/Cargo.toml
cargo test --manifest-path tools/servo-embed-harness/Cargo.toml
node tools/servo-embed-harness/tests/frontend-smoke.cjs
cargo run --manifest-path tools/servo-embed-harness/Cargo.toml --release
```

Memory measurement for the actual NovaBrowser, with the app open:

```powershell
powershell -ExecutionPolicy Bypass -File tools/servo-embed-harness/measure-memory.ps1 -ProcessName nova-desktop -Samples 15 -IntervalSeconds 2 -CsvPath nova-memory.csv
```

Repeat with `-ProcessName nova-servo-embed-harness` only for the engine
harness and identical site workloads. Observe whole-process-tree
PrivateMB, WorkingSetMB and process counts. Compare more than one workload
and report variation rather than a single unqualified result.

No GitHub Actions or remote build workflows are required or added.

## Sources

- https://doc.servo.org/servo/
- https://doc.servo.org/servo/struct.WebView.html
- https://github.com/servo/servo/blob/main/components/servo/examples/winit_minimal.rs
- https://github.com/copse-dev/tauri-runtime-servo
- https://docs.rs/crate/servo/0.8.0

## Native integration work (2026-11-22)

The existing NovaBrowser shell now contains an opt-in direct Servo WebView
adapter at `crates/nova-desktop/src/servo_embed.rs`, with tab/command routing
and native-window layout from the original UI. This is a feature-gated
integration, not the replacement Servo harness. See
[Windows manual validation](SERVO_NATIVE_EMBED.md). No Windows build or
RAM benchmark has been completed for this new code.
