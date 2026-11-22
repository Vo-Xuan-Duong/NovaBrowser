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

## Optional Servo experimental runtime

The opt-in `experiments/servo-preview` prototype depends on the Servo web
engine (https://servo.org/, MPL-2.0) and the third-party
`tauri-runtime-servo` integration (https://github.com/copse-dev/tauri-runtime-servo,
Apache-2.0 OR MIT). These sources are fetched through Cargo when the preview
is built and are **not copied** into the NovaBrowser repository. Review their
complete transitive licensing requirements before distributing binaries.
