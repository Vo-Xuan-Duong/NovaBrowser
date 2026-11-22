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

## Desktop UI: Slint

The new desktop shell depends on Slint (https://slint.dev/). Slint offers
multiple licensing options including GPL and commercial options. Review
those terms for the intended product and distribution model; the Apache-2.0
license of the underlying engine does not automatically grant a license to
distribute Slint under all terms.
