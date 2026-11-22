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
cargo test -p nova-desktop
cargo run -p nova-desktop --release
```

Cargo will resolve additional Tauri dependencies for the desktop crate and
update **Cargo.lock**. Review and commit that update after building locally.
The original engine requires significant disk space to compile V8.

## Experimental Servo engine preview

A **separate, opt-in Servo-backed browser executable** is available at
[`experiments/servo-preview/`](experiments/servo-preview/README.md). It renders
real external sites using Servo (without Obscura and without WebView2),
reuses NovaBrowser's existing URL/search normalization, and provides native
Back/Forward/Reload/Home commands. The current NovaBrowser desktop is not
replaced yet because Servo's experimental Tauri runtime cannot host several
webviews in one native window.

```cmd
cargo run --manifest-path experiments/servo-preview/Cargo.toml --release
```

For Windows toolchain requirements, memory measurements and limitations, see
[`Servo evaluation`](docs/Servo-engine-evaluation.md). The preview has a
separate Cargo workspace and does not change the main `Cargo.lock`.

## What is currently implemented

- Native Tauri desktop shell with a Chrome-inspired title bar, address bar and
  Back / Forward / Reload.
- Interactive offline Home/new-tab page at `nova://home` with search, real
  bookmarks, recent visits, public quick-link suggestions and native settings
  access; selectable as the homepage or new-tab destination.
- Explicit Google anti-bot/search-challenge handling: a localized native notice,
  user-initiated Bing/DuckDuckGo alternatives, retry and view-original-page
  actions. The browser cannot guarantee access when Google refuses requests.
- Tab keyboard navigation and reopen-closed-tab (Ctrl+Shift+T).
- Consistent Lucide SVG icons for the browser chrome, hosted locally without
  icon fonts or runtime CDN dependencies.
- Multiple isolated page tabs with shared browser cookies, capped at eight tabs.
- Persistent tab session restore: reopen previous tabs and selected tab, loading
  inactive tabs only when selected.
- Local bookmarks and browsing history with quick access from the toolbar.
- Full browser Settings workspace (`nova://settings`): searchable sections,
  startup, appearance, language, search provider, tabs, privacy, and about;
  working Home button, bookmarks bar, and data-clearing controls.
- Offline English/Vietnamese interface localization with system-language
  default, instant preview and persisted preference.
- Integration with the existing Rust DOM, networking and V8 JavaScript runtime.
- Windows Unicode/CJK font fallback: a bounded set of installed system fonts
  for Han, Hangul, Japanese, Thai and Indic text. No font downloads or
  redistributions; falls back to bundled faces when system fonts are absent.
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
