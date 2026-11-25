# NovaBrowser — native Servo renderer integration (Windows)

## Why the browser and rendering engine have independent Cargo graphs

NovaBrowser is **one** application. Its current native shell and browser UI
are still `crates/nova-desktop/`, including Home, Settings, tabs, session
restore, bookmarks and bilingual UI. Only the site rendering engine is
replaceable.

A direct Servo dependency in the main workspace triggered:

```text
V8/temporal_capi -> icu_calendar ^2.2.1
Servo/mozjs_sys/icu_capi 2.1.2 -> icu_calendar ~2.1.1
failed to select a version for icu_calendar
```

Cargo resolves a workspace lockfile with all members and optional features,
so `cargo update`, `--no-default-features` or a plain feature-gate does not
solve this native-library conflict. Keep the engine in its own workspace.

## Files and process boundary

| Path | Purpose |
| --- | --- |
| `crates/nova-desktop/` | The actual NovaBrowser executable, UI and local browser profile |
| `crates/nova-desktop/src/servo_embed.rs` | Rust IPC adapter; contains no Servo crate dependency |
| `engines/servo-native/Cargo.toml` | Independent Servo 0.8 / SpiderMonkey Cargo workspace |
| `engines/servo-native/src/main.rs` | Native Windows child HWND, GPU rendering, WebViews and input |
| `run-nova-servo.cmd` | Builds renderer first, then runs NovaBrowser with the feature |
| `check-nova-servo.cmd` | Runs local source/IPC tests and both Rust workspace checks |
| `engines/servo-native/tests/isolation.cjs` | Static lockfile separation and IPC regression test |

The helper accepts tagged JSON commands via its inherited **stdin**:
`navigate`, `select`, `close`, `back`, `forward`, `reload`,
`layout` and `shutdown`. It emits prefixed JSON lines via **stdout** for
`ready`, `fatal`, `location`, `title`, `history`, `error` and browser
keyboard shortcuts. External pages never receive a Tauri IPC capability.

The child HWND is parented to the NovaBrowser native window. Tauri/WebView2
still draws the browser's trusted chrome; Servo paints external pages below
it. The helper is not a second browser or separate GUI product, although it
is technically another operating-system process.

## Run locally on Windows x64

```cmd
git pull origin main
rustup update stable
check-nova-servo.cmd
run-nova-servo.cmd
```

Equivalent manual release commands:

```cmd
cargo build --manifest-path engines/servo-native/Cargo.toml --release
cargo run -p nova-desktop --release --features servo-embed
```

`cargo run -p nova-desktop --features servo-embed` by itself expects the
Servo helper to have already been built. If it cannot find the executable,
the app reports the expected path. If using a nonstandard Cargo target
directory, set `NOVA_SERVO_SIDECAR_BIN` accordingly.

Normal Obscura page-rendering build remains available using:

```cmd
run-nova.cmd
```

The sidecar may add Servo-specific native dependencies. Use a recent
Windows C++ toolchain and SDK. Any errors after the resolver stage may be
distinct API, native library or GPU integration issues; **none have been
validated by compiling this version on Windows in this session**.

## Acceptance checklist

1. Compiler: Cargo resolves both workspaces independently without an ICU
   conflict; check and build the root and the Servo helper separately.
2. GUI: Nova Home and Settings remain the original NovaBrowser screens.
3. Website: a Servo site draws in the same NovaBrowser window below toolbar
   and tabs without a PNG/frame screenshot bridge.
4. Geometry: resize/maximize/Windows DPI scaling and bookmarks-bar toggling
   preserve the child window's bounds and prevent it covering toolbar.
5. Multiple tabs: switching, close/reopen, URL/title/history, Back/Forward,
   search and session persistence all behave correctly.
6. Native input: mouse, wheel, keyboard shortcuts, focus, Unicode and IME
   work across the child HWND and parent trusted chrome.
7. Security: remote origins cannot access Tauri commands; privileged
   schemes, popups, downloads and site permissions have safe handlers.
8. RAM: sample the **entire process tree**, including the helper, under the
   same browsing workloads as the default rendering engine.

Recent browser integration changes:
- Native Servo mode no longer constructs unused Obscura/V8 `Page`
  isolates or `BrowserContext` objects for every tab. It also avoids the
  legacy font setup and reserves a smaller control-thread stack.
- Browser history now records successful native page locations even when
  the target URL matches the address bar, then updates the real document
  title without creating duplicate visits.
- Leaving a website for `nova://home` retains its live Servo WebView.
  Back restores the page without reloading; hidden page redirects and
  title changes cannot replace the trusted internal Home route.
- The browser detects native renderer pipe closure and reports a stopped
  process. Settings > About displays the selected engine and its current
  process state. Native shortcuts use physical keys.
- The native renderer limits privileged top-level navigation to
  `about:blank`; other non-HTTP(S) targets are rejected.

Current limitations: cross-process Win32 parent-child composition has not
been verified; IME, GPU rendering and site compatibility remain unproven.
The feature-gated build still compiles some legacy Obscura dependencies,
so it is **not yet a Servo-only executable**. No production-readiness or
measured memory savings are claimed.

Never run GitHub Actions; compile and test locally.

## Technical references

- https://doc.rust-lang.org/cargo/reference/resolver.html
- https://doc.rust-lang.org/cargo/reference/workspaces.html
- https://doc.servo.org/servo/
- https://github.com/servo/servo/blob/main/components/servo/examples/winit_minimal.rs
