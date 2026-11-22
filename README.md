# NovaBrowser

**NovaBrowser is your personal Rust desktop browser.** Its existing browser
chrome, tabs, omnibox, Home, Settings, bookmarks, history, sessions, and
English/Vietnamese UI remain the product. Third-party engines are components,
not replacement browser applications.

> Engineering preview only. The default website renderer is still Obscura's
> CPU screenshot path. An **optional native Servo integration** now exists
> inside the same NovaBrowser window, but is **not yet Windows build-verified**.

## Default NovaBrowser (existing browser UI)

On Windows, install Rust, Visual Studio C++ Build Tools, Windows SDK, and the
native requirements of the current V8-based engine. From repository root:

```cmd
git pull origin main
run-nova.cmd
```

Equivalent commands:

```cmd
cargo nova
cargo run -p nova-desktop --release
```

This launches the full NovaBrowser interface from
`crates/nova-desktop/ui/`, including its existing features and user profile.

## Optional: Servo rendering inside NovaBrowser

Servo's SpiderMonkey and the legacy Obscura V8 dependencies require
**incompatible ICU native libraries**. Cargo resolves optional dependencies
for the whole root workspace lockfile, so adding Servo directly to
`nova-desktop` caused the `icu_calendar` version-selection error.

The fix uses **one NovaBrowser application** and one internal render helper:

- `crates/nova-desktop/` is still the **actual browser** with the existing
  toolbar, tabs, Home, Settings, bookmarks, session data and localization.
- `engines/servo-native/` builds a small **render-only process** with
  Servo 0.8, SpiderMonkey and its **own Cargo workspace and lockfile**.
- `crates/nova-desktop/src/servo_embed.rs` sends navigation, tab lifecycle
  and layout commands over private standard-input JSON lines. Native Servo
  page events come back through a private standard-output protocol.
- The renderer attaches its native child HWND below NovaBrowser's existing
  browser chrome. It does not provide another browser UI or an HTTP server.

**Recommended Windows local command:**

```cmd
git pull origin main
run-nova-servo.cmd
```

This first builds the isolated Servo helper, then runs NovaBrowser with
`servo-embed` enabled. If preferred, run the two commands manually:

```cmd
cargo build --manifest-path engines/servo-native/Cargo.toml --release
cargo run -p nova-desktop --release --features servo-embed
```

**Diagnostics and local checks:**

```cmd
node engines/servo-native/tests/isolation.cjs
cargo check -p nova-desktop --features servo-embed
cargo check --manifest-path engines/servo-native/Cargo.toml
```

The initial Servo build may require a modern Rust toolchain, C++ build tools,
Windows SDK and additional Servo system libraries. For custom target paths,
set `NOVA_SERVO_SIDECAR_BIN` to the built helper executable.

**Status:** dependency isolation has been implemented and the Node contract
test passes, but **neither Rust workspace nor the cross-process native child
window has been compiled/validated here on Windows**. Servo GPU compositing,
IME, multi-tab interactions, security and RAM still need testing. The
feature-gated NovaBrowser build currently retains some legacy Obscura
initialization; it is not yet memory optimized. Do not use the prototype for
sensitive accounts.

See [Windows-native integration checklist](docs/SERVO_NATIVE_EMBED.md)
and [engine decision](docs/ENGINE_DECISION.md).

## Reuse without creating another browser

The `crates/obscura-*/` source remains for selected engine utilities and
the current default page pipeline while native integration is verified.
The standalone `tools/servo-embed-harness/` is a test harness only, **not**
a NovaBrowser release. The goal is a single browser with a single active page
renderer, preserving existing UI, bookmarks, settings and profile data.

No GitHub Actions; build and test on your own Windows machine.

## RAM and licensing

A low process-tree memory footprint is a goal, not an established Servo
advantage. Compare the full application process tree on matching Windows
hardware, pages and workloads using the RAM sampler at
`tools/servo-embed-harness/measure-memory.ps1`.
Keep the original Obscura notices and review Servo MPL-2.0 requirements
before distributing binaries. See [third-party notices](THIRD_PARTY_NOTICES.md).
