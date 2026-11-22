# NovaBrowser

**NovaBrowser is a personal desktop browser developed as one product.**
Its own toolbar, tabs, Home, Settings, language preferences, bookmarks,
history and stored sessions are the application. Existing browser engines
are implementation sources, **not products that replace this shell**.

> **Current status: engineering preview, not a secure production browser.**
> The existing NovaBrowser window is functional as a shell but still uses
> experimental Obscura screenshot-based site rendering. Servo has been
> evaluated as a source for native web rendering, but its WebView compositor
> is **not yet embedded into the existing shell**. Do not assume the
> Servo test harness is the NovaBrowser application.

## Run the actual NovaBrowser on Windows

Install Rust, the Visual Studio C++ toolchain, Windows SDK and dependencies
needed for the current V8/Obscura build. From the repository root:

```cmd
git pull origin main
run-nova.cmd
```

The two equivalent ways to launch **the same NovaBrowser shell** are:

```cmd
cargo nova
cargo run -p nova-desktop --release
```

Local checks (no GitHub Actions):

```cmd
cargo check -p nova-desktop
cargo test -p nova-desktop
cargo run -p nova-desktop --release
```

The current main application is `crates/nova-desktop/` and its
`ui/` directory. It already contains the browser chrome, tab management,
bilingual English/Vietnamese UI, Settings, search providers, bookmarks,
history and saved sessions. **Keep these components, not a minimal Servo
sample UI, as the product.**

## Engine integration strategy

**Reuse instead of rewrite:**

- Servo's native embedding APIs (`ServoBuilder`, `WebViewBuilder`,
  `RenderingContext`, `WebView::notify_input_event`) are the candidate
  for website HTML/CSS, JavaScript and compositing inside NovaBrowser.
- Keep useful application-level components from the existing Obscura code,
  especially URL/search normalization, session persistence and the
  already-developed browser UI. Do not nest a second full headless browser
  under the Servo compositor.
- `crates/nova-desktop/src/engine_api.rs` establishes an engine-agnostic
  tab/input/navigation contract. The shell is still running the original
  renderer while the adapter is implemented and tested.
- The independent Servo program at
  [`tools/servo-embed-harness/`](tools/servo-embed-harness/) is an
  **integration test harness only**. It is not an alternate NovaBrowser
  release or the default launch command.

Key unresolved engineering work: native multi-webview composition with
persistent trusted browser chrome, input/IME, per-tab lifecycle and idle
memory management, actual Windows testing, and robust site security.
The experimental `tauri-runtime-servo` adapter cannot currently embed
multiple Servo webviews within one native window. A direct Servo embedder or
suitable maintained adapter must solve that boundary before cutover.

See [the architectural decision](docs/ENGINE_DECISION.md) and
[Servo integration notes](docs/Servo-engine-evaluation.md).

## Resource goals

Low RAM usage is a project goal, **not a measured Servo advantage**.
Benchmark private committed bytes and working set of all process descendants
under identical workloads. The test harness includes
[`measure-memory.ps1`](tools/servo-embed-harness/measure-memory.ps1).

## Attribution

The repository contains significant Obscura-derived source code.
Keep original notices and review Servo's MPL-2.0 requirements when
distributing an embedded engine. See [third-party notices](THIRD_PARTY_NOTICES.md).
