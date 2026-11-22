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

An experimental Windows native child-window compositor was added to the
**same NovaBrowser executable**. It uses Servo's WebView API, not a Servo
sample application. The HTML/CSS frontend still owns the persistent toolbar,
Home, Settings and local library. Servo renders external websites beneath
the chrome, with per-tab WebView handles and native input events.

To build locally (Windows x64, Rust stable, Servo native prerequisites):

```cmd
cargo check -p nova-desktop --features servo-embed
cargo test -p nova-desktop --features servo-embed
cargo run -p nova-desktop --release --features servo-embed
```

**Unverified:** these commands have not been run successfully on Windows in
this development session. The opt-in implementation may require changes to
native integration APIs and GPU child-window compositing. Keyboard/IME,
download/permission flows, browser process isolation, and resource behavior
are incomplete. The optional mode still constructs legacy Obscura tab
objects and has **no demonstrated RAM saving**. Do not use it for sensitive
browsing.

Relevant source:
- `crates/nova-desktop/src/servo_embed.rs`: native Servo 0.8 WebView,
  event loop, per-tab handles, input and direct paint
- `crates/nova-desktop/src/main.rs`: navigation and tab events, native
  surface commands, storage integration
- `crates/nova-desktop/src/engine_api.rs`: browser-owned engine contract
- `crates/nova-desktop/ui/app.js`: chrome clipping and surface visibility

See [Windows validation](docs/SERVO_NATIVE_EMBED.md) and
[architecture decision](docs/ENGINE_DECISION.md).

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
