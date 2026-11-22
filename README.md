# NovaBrowser

**NovaBrowser** is an independent Rust browser project being developed as a native
desktop application. The repository includes an experimental Tauri browser shell
with an HTML/CSS interface and a retained, actively evolving engine codebase
originally developed as
[Obscura](https://github.com/h4ckf0r0day/obscura).

> **Status: engineering preview — not a production-ready browser.**
> The desktop window currently displays CPU-rendered PNG frames and uses an
> experimental DOM-based input adapter. Many websites will not work correctly.

## Get started on Windows

Install [Rust](https://rustup.rs/), Visual Studio C++ Build Tools and CMake,
along with the dependencies required to build V8. Tauri uses the WebView2
Runtime for its interface on Windows. Then, in the repository root:

```powershell
cargo check -p nova-desktop
cargo nextest run --release -p nova-desktop
cargo run -p nova-desktop --release
```

Cargo will resolve additional Tauri dependencies for the desktop crate and
update **Cargo.lock**. Review and commit that update after building locally.
The original engine requires significant disk space to compile V8.

## What is currently implemented

- Native Tauri desktop shell with a Chrome-inspired title bar, address bar and
  Back / Forward / Reload.
- Multiple isolated page tabs with shared browser cookies, capped at eight tabs.
- Persistent tab session restore: reopen previous tabs and selected tab, loading
  inactive tabs only when selected.
- Local bookmarks and browsing history with quick access from the toolbar.
- Settings for light/dark appearance and a saved homepage for new tabs,
  with live theme preview, cancel/revert, homepage validation and persistence.
- Integration with the existing Rust DOM, networking and V8 JavaScript runtime.
- Basic page screenshots, mouse clicks, scrolling and text entry.
- Viewport updates on window resize, with bounded frame dimensions.
- Provisional in-app navigation history and basic URL/search normalization.
- The original headless CLI, CDP and related engine crates remain available.

**Not implemented:** standards-complete input/IME, GPU page compositing,
downloads, permission prompts, extension support, full standards
compatibility, robust security sandboxing or process isolation.

## Repository layout

| Path | Purpose |
| --- | --- |
| `crates/nova-desktop/` | Native desktop browser preview |
| `crates/obscura-browser/` | Browser pages, navigation and lifecycle |
| `crates/obscura-dom/` | HTML/DOM primitives |
| `crates/obscura-js/` | V8 and browser JavaScript APIs |
| `crates/obscura-render/` | CSS layout and paint |
| `crates/obscura-net/` | Network stack and cookies |
| `crates/obscura-cdp/` | DevTools Protocol support |
| `docs/NovaBrowser-desktop-MVP.md` | Desktop architecture and limitations |

The original `obscura-*` crate names are **deliberately retained** for now:
renaming everything at once would make engine-level maintenance and migration
harder without improving the user-facing browser.

## Build & testing policy

**Build and test locally.** The repository's inherited GitHub Actions
workflows are being removed; no GitHub-hosted builds are part of the current
development process.

## Open-source acknowledgments

NovaBrowser is an independent repository, **not an independently invented
browser engine**. Substantial code comes from the Obscura project and its
contributors. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and
[LICENSE](LICENSE) for attribution and licensing.

On Windows, Tauri uses Microsoft Edge WebView2 to draw NovaBrowser's interface.
Website content is still rendered by the Rust browser engine.
