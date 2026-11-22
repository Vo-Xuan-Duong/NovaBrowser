# NovaBrowser desktop: integration MVP

## Intent

Build an independent native desktop browser shell on top of the Rust engine
retained from Obscura. Retain the existing engine crates initially and add
new NovaBrowser-specific components alongside them.

## Native window / page pipeline

```text
NovaWindow (Slint + Winit + FemtoVG)
    -> Command channel (std::sync::mpsc)
        -> Engine thread (Tokio current_thread + V8)
            -> obscura-browser::Page (navigation / JS / DOM)
                -> screenshot(viewport) -> PNG bytes
    <- invoke_from_event_loop(PNG) -> slint::Image
```

A single engine thread owns Page and its V8 isolate. The Slint UI thread only
sends commands and displays finished frames. Frame hashing avoids reloading
the same PNG image repeatedly.

The window tracks its native content size; the engine's viewport and image
size follow that size (bounded to a maximum 8-million-pixel frame). Input
coordinates use the same viewport dimensions as the screenshot.

## Functionality and security

- Address bar: HTTP(S), hostname shortcuts, search terms.
- Browser navigation: Back, Forward, Reload, page URL and title.
- Provisional pointer events, scrolling and simple editable text inputs.
- Obscura's default SSRF restrictions remain enabled: `localhost` and private
  IP URLs can be parsed but may be blocked by the engine.
- `file:`, `javascript:` and other unsupported schemes are rejected from the
  browser's address bar.

This is not a secure multi-process browser. Do not treat this preview as a
production security boundary or a replacement for Chromium/Firefox.

## Known limitations

- Rendering uses PNG screenshots rather than an efficient GPU compositor.
- Only one page/tab; site sandboxing and process isolation are not present.
- DOM-level synthetic input is incomplete, especially for IME, text editing,
  selection, drag/drop, complex frameworks and some form controls.
- Back/Forward are managed by the shell, not full engine session history.
- Animated/interactive content receives a periodic best-effort refresh.
- Site compatibility is limited by the current browser engine.
- Cargo.lock must be resolved locally for the newly introduced Slint crates.
- No GitHub Actions are used. No build of this desktop crate has been verified
  in the editing environment.

## Local validation

```powershell
cargo check -p nova-desktop
cargo test -p nova-desktop
cargo run -p nova-desktop --release
```

Slint 1.18.1 still selects an internal software-renderer dependency that
requires `swash ^0.2.7`, even when Slint's default features are disabled.
The Obscura paint renderer and vendored cosmic-text must therefore resolve a
shared compatible `swash` version. NovaBrowser pins the renderer to
`swash =0.2.10` to satisfy both dependency requirements. The window uses
Winit + FemtoVG for presentation and Slint image decoding for PNG frames.
This change has not yet been verified with a local Cargo build; check glyph
rendering and variable fonts before considering it production-ready.

## Next milestones

1. Real event dispatch and IME (including focus, key modifiers, wheel, drag).
2. Multi-tab orchestration, live history and session restore.
3. Retained scene layers / GPU compositor instead of repeated PNG conversion.
4. Downloads, bookmarks, permissions, storage and profiles.
5. Security isolation, crash handling, improved HTML/CSS/JS compatibility.


## Desktop shell styling

The native Slint window now uses a Fluent-inspired two-row browser chrome:
a single active-tab header (with live document title), compact vector-path
Back/Forward/Reload controls, a prominent address field and Go action, and
a low-profile loading/status strip. Disabled navigation actions are visually
distinguished and are not clickable. The rendered web viewport and mouse
coordinate mapping are unchanged.

Keyboard shortcuts while the web viewport has focus: Ctrl+L selects the
address bar, Ctrl+R and F5 reload the page. These shortcuts are currently
handled by the desktop window, not by a full browser input event pipeline.

The tab strip is intentionally limited to a single active tab: multi-tab
management must not be implied until the backend supports it.
