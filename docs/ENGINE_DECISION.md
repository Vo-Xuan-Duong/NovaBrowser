# ADR-001: Servo as the sole NovaBrowser website engine

Status: Accepted (2026-11-22).

## Decision

Develop **one NovaBrowser application** using Servo for website layout,
scripting, input and rendering. Do not maintain Obscura as an active fallback
or develop two desktop browsers. Retain Obscura code only to selectively migrate
features already implemented in the old browser shell.

Servo's embedder WebView API is architecturally suitable for a native Rust
browser. The previous Obscura desktop displays CPU-painted PNG screenshots
of pages and synthesizes user input, an unsuitable long-term basis for a
general interactive browser. Windows WebView2 provides mature web compatibility
and multi-view integration, but retaining it conflicts with the preference
for an independent rendering engine.

**No comparative RAM benchmark has been performed.** Servo may or may not
use less total memory; do not claim savings without measuring the full process
tree on the same Windows hardware and identical workloads.

## Implementation

- Official runtime: `apps/nova-browser/`.
- Official launch: `run-nova.cmd`, `cargo nova`.
- Historical reference source: `crates/nova-desktop/` and `crates/obscura-*/`.
- Separate Cargo workspace: isolate incompatible engine dependency graphs,
  **not** create another browser product.
- Servo 0.5.0 is pinned to the adapter's tested dependency graph. Servo 0.8.0
  was available on 2026-11-22, but cannot be substituted without updating
  the `tauri-runtime-servo` adapter.

## Remaining integration milestones

1. Support a persistent local chrome and several independently managed Servo
   website views. The current Tauri adapter does not support several Servo
   webviews in one native window.
2. Bring existing UI, settings, tab management, history, bookmarks, English/
   Vietnamese localization, and session persistence across without exposing
   Tauri IPC to untrusted website origins.
3. Verify Google, UTF-8/IME, JavaScript/forms, download flows, permissions and
   URL security on Windows.
4. Benchmark complete application-process trees for idle, static, and
   script-heavy pages, using equal conditions and stable settle policies.
5. Implement and test profile migration before removing historical code.

Until these steps are complete, the selected Servo executable is a technical
preview, not a feature-complete replacement for the former app.

Sources:
- https://servo.org/
- https://doc.servo.org/servo/
- https://github.com/copse-dev/tauri-runtime-servo
- https://docs.rs/crate/servo/0.8.0

No GitHub Actions or hosted test workflows are used.
