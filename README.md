# NovaBrowser

**NovaBrowser** is an independent Rust browser project being developed as a native
desktop application. The repository includes an experimental browser shell and
a retained, actively evolving engine codebase originally developed as
[Obscura](https://github.com/h4ckf0r0day/obscura).

> **Status: engineering preview — not a production-ready browser.**
> The desktop window currently displays CPU-rendered PNG frames and uses an
> experimental DOM-based input adapter. Many websites will not work correctly.

## Get started on Windows

Install [Rust](https://rustup.rs/), Visual Studio C++ Build Tools,
CMake and the dependencies required to build V8. Then, in the repository root:

```powershell
cargo check -p nova-desktop
cargo test -p nova-desktop
cargo run -p nova-desktop --release
```

Cargo will resolve additional dependencies for the new desktop crate and
update **Cargo.lock**. Review and commit that update after building locally.
The original engine requires significant disk space to compile V8.

## What is currently implemented

- Native Slint desktop shell with address bar and Back / Forward / Reload.
- Integration with the existing Rust DOM, networking and V8 JavaScript runtime.
- Basic page screenshots, mouse clicks, scrolling and text entry.
- Viewport updates on window resize, with bounded frame dimensions.
- Provisional in-app navigation history and basic URL/search normalization.
- The original headless CLI, CDP and related engine crates remain available.

**Not implemented:** tab management, standards-complete input/IME, GPU page
compositing, downloads, permission prompts, extension support, full standards
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

Before redistributing a closed-source desktop build, review the applicable
**Slint UI licensing** terms separately.
