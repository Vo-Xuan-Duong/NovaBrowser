# NovaBrowser desktop preview

This crate is the first native desktop window backed by the Rust/Obscura
engine, not a production-ready browser. It is deliberately isolated from
upstream crates to keep future engine merges practical.

## Run

Install stable Rust and the native development tools for V8; on Windows x64
install Visual Studio C++ Build Tools. From the repository root:

    cargo run -p nova-desktop --release

On Linux, install Slint's system windowing dependencies as well.

The existing Cargo.lock comes from upstream. Running Cargo after adding Slint
will update it; commit the resolved lockfile after validating the build.

## Capabilities

- Native Slint window with address bar, Back, Forward and Reload.
- The existing Obscura HTTP/DOM/V8/render engine on a dedicated thread.
- Display engine-generated PNG frames inside the window.
- Basic DOM-level click, scroll and text input forwarding.
- Preliminary shell-side navigation history and periodic refresh.
- Address bar supports HTTP(S) URLs, hostnames and search terms.

The engine's default SSRF/private-network restrictions are retained. Localhost
navigation will require a narrowly scoped explicit user-navigation policy.

## Important limitations

This is a **single-tab integration prototype**. It transfers encoded PNG
frames instead of using a GPU compositor, so the surface is not resize-aware,
animations are slow, and video is not supported. Input uses JavaScript DOM
evaluation instead of native trusted mouse/key/IME events. Rich editors, many
web apps and same-document history will not work like Chromium.

There is no site sandbox, tab process isolation, extension support, download
manager, permissions UI, bookmarks or profile management yet.

This patch was authored without Rust or the full build toolchain available
to the editing environment. Run the following before merging:

    cargo check -p nova-desktop
    cargo nextest run -p nova-desktop --no-fail-fast

## Next milestones

1. Expose the engine's retained paint surfaces/dirty regions to a compositor;
   replace the PNG screenshot polling path.
2. Connect native pointer/keyboard/IME dispatch, resize and viewport updates.
3. Integrate the engine's history, tabs, navigation and page lifecycle.
4. Add permission prompts, sandboxed processes and automated UI smoke tests.
5. Build a dedicated Windows release pipeline.

Review Slint's licensing terms before any proprietary distribution.
