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

Slint default features are disabled to avoid pulling the optional software
renderer that conflicts with the engine's pinned `swash` version. The window
uses Winit + FemtoVG and enables image decoders for PNG frames. This must still
be confirmed by Cargo on a local development machine.

## Next milestones

1. Real event dispatch and IME (including focus, key modifiers, wheel, drag).
2. Multi-tab orchestration, live history and session restore.
3. Retained scene layers / GPU compositor instead of repeated PNG conversion.
4. Downloads, bookmarks, permissions, storage and profiles.
5. Security isolation, crash handling, improved HTML/CSS/JS compatibility.
