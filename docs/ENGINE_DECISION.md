# ADR-001 — NovaBrowser owns the product; third-party engines are components

Status: **Accepted and corrected, 2026-11-22**.

## Product decision

NovaBrowser is the **one** browser application. Its UI, data, preferences,
tab/session behavior and identity are authoritative. Do not replace it with
a Servo demonstration shell or reimplement working NovaBrowser features
without a concrete need.

The earlier decision to promote a `tauri-runtime-servo` demonstration as
the official app was **wrong**. Revert that promotion. The canonical shell
is `crates/nova-desktop/`; `run-nova.cmd` and `cargo nova` must launch it.

## Component selection

Servo is the primary **candidate for native website rendering**, selected
for its embeddable WebView API and the intent to avoid screenshot-based
rendering. It can supply website HTML/CSS, JavaScript, layout, painting,
WebView control and native input integration. This is **not** an instruction
to adopt servoshell, its demo UI, or a completely separate product.

Re-use well-fitting features from existing Obscura/NovaBrowser modules,
including navigation, local storage, session handling, and UI work.
Do **not** run both Obscura's and Servo's complete page rendering/JavaScript
pipelines for the same website. Auxiliary components should earn their
place via compatibility, correctness and memory benchmarks.

The historical Obscura page renderer remains in the current executable
**temporarily** while the Servo adapter is implemented; no unsupported
claim of completed engine migration should be made.

## Required architecture

```text
NovaBrowser (existing UI: tabs, omnibox, Home, Settings, i18n)
  | browser-owned commands/events; persisted browser profile
Nova browser core / engine_api.rs
  | trusted event-loop + view lifecycle + native input + rendering bounds
Servo WebView embedder [to be integrated into this window]
  | HTML/CSS/JS/layout/rendering of external websites
```

- Each website has an engine WebView handle owned by its tab.
- Keep app chrome persistent, separate from untrusted website documents.
- Do not give remote websites Tauri IPC capability.
- Retain browsing history/bookmarks/Settings/session data in NovaBrowser.
- Integrate real pointer, keyboard and IME input; avoid synthetic JavaScript
  events as the final input path.
- Background tabs should be able to hide/suspend/discard without deleting
  persisted browser state.
- Adapt to the same native window; never iframe a remote webpage to fake tabs.

## Evidence and blockers

Servo 0.8.0 API documents `ServoBuilder`, `WebViewBuilder`, WebView paint,
`RenderingContext` and the `EventLoopWaker` contract. The currently
pinned `tauri-runtime-servo` 0.1.0 harness embeds Servo 0.5.0 and does
**not** support multiple Servo webviews in one window. Therefore the
harness stays in `tools/servo-embed-harness/`, not under the product app.
Investigate a direct Servo embedder or an adapter that supports independent
site WebViews and persistent chrome before selecting the final implementation.

No measured Windows RAM comparison has been performed; no claim that Servo
uses less RAM than WebView2/Obscura is justified yet.

## Completion criteria

1. One NovaBrowser binary launches the existing browser chrome.
2. Servo paints external content in that window without a screenshot
   bridge or a replacement Home/Settings UI.
3. Multiple pages support native input, history, switching, resizing, focus,
   isolation and controlled background-resource lifecycles.
4. Existing bookmark/history/preferences/session data survive migration.
5. Windows-local tests and process-tree memory benchmarks are recorded.
6. Only after validation retire unused legacy full-engine code and review
   licensing/attribution.

References:
- https://doc.servo.org/servo/
- https://doc.servo.org/servo/struct.WebView.html
- https://github.com/servo/servo/blob/main/components/servo/examples/winit_minimal.rs
- https://github.com/copse-dev/tauri-runtime-servo

No GitHub Actions, automated builds or hosted workflows.
