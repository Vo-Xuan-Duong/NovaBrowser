# ADR-001: NovaBrowser owns the product and embeds suitable engine components

Status: Accepted and corrected (2026-11-22).

## Product

NovaBrowser is **one** personal browser: the existing tab strip, omnibox,
internal Home/Settings, bilingual UI and persisted local data.
Neither servoshell nor the stand-alone Servo demonstration is the product.
Keep and extend `crates/nova-desktop/` rather than discarding its
already-developed interface.

## Engine selection

Servo's embeddable WebView API is the preferred candidate for rendering
website HTML/CSS/JavaScript. Reuse existing NovaBrowser and Obscura
application-level modules for URL normalization, history, bookmarks and
session persistence. Do not run two full website renderers simultaneously
for the same page. The Obscura screenshot pipeline remains the **default**
until a Windows-verified native replacement is available.

A **feature-gated native Servo rendering integration** now exists. The
browser-owned adapter in `crates/nova-desktop/src/servo_embed.rs` talks via
private IPC to a render-only helper in `engines/servo-native/`. The helper
has its own Cargo workspace because Servo and Obscura/V8 need incompatible
ICU bindings. Servo's event loop and page compositor are intended to draw
inside a native child HWND beneath NovaBrowser's trusted chrome. It is opt-in with
`--features servo-embed` and is **not yet Windows-compiled/accepted**.
The experimental adapter supports independent WebView handles but still
requires GPU/IME compatibility and window composition tests.

The previous decision that changed the official app to
`apps/nova-browser/` was incorrect and has been reverted. The program in
`tools/servo-embed-harness/` is a separate research tool, never the
canonical launcher.

## Acceptance criteria

1. One native NovaBrowser window retains all current UI features.
2. Servo external websites render below toolbar without PNG screenshot
   transfer or privileged Tauri IPC exposed to remote pages.
3. Tab switching, browser history, site navigation, focus, input and native
   composition work on real Windows hardware.
4. Bookmarks, Settings, languages and session persistence remain compatible.
5. Native child window safely hides for Home, Settings and browser library.
6. GPU scaling, IME, popups/downloads, security boundaries and site
   compatibility pass manual Windows regression tests.
7. Compare complete process-tree RAM under identical workloads; do not
   claim Servo is lighter without measurements.
8. Only then switch the main build from Obscura to Servo and remove unused
   legacy engine code while preserving attribution.

See [native validation](SERVO_NATIVE_EMBED.md) for the implemented
integration and known limitations. No GitHub Actions.

References: https://doc.servo.org/servo/ and
https://github.com/servo/servo/blob/main/components/servo/examples/winit_minimal.rs
