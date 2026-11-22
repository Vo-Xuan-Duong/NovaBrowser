# Third-party notices and provenance

## Obscura browser engine

NovaBrowser incorporates and modifies substantial source code originating
from **Obscura**, developed by h4ckf0r0day and other contributors:

- Original project: https://github.com/h4ckf0r0day/obscura
- Original website: https://obscura.sh
- License: Apache License 2.0
- Code retained in `crates/obscura-*`, `vendor/`, and some project-level
  utilities, tests, and documentation.

The original copyright and license notices must remain in applicable files.
The upstream Apache-2.0 license is retained as `LICENSE`. Repository
independence on GitHub does not alter the origin or license of code.

## Desktop UI: Tauri

NovaBrowser uses Tauri 2 and WebView2 (Windows) for the desktop shell.
Review each runtime dependency's license before redistribution.

## Desktop icons: Lucide

NovaBrowser embeds a curated subset of SVG icons from Lucide:
https://github.com/lucide-icons/lucide

- Source revision: e00b2b26350fe4dfb6f5273aa366f94974430903
- License: ISC, with MIT terms for Feather-derived icons
- Full license text: `crates/nova-desktop/ui/LUCIDE_LICENSE`
- Embedded icon registry: `crates/nova-desktop/ui/icons.js`

These icons are self-hosted. NovaBrowser's chrome does not request SVG assets
from external servers when it starts.
