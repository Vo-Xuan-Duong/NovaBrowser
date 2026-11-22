# Servo embedding harness (non-product)

This directory is **only a local Windows integration test** for the Servo
rendering engine and the experimental `tauri-runtime-servo` adapter.

**This is not the NovaBrowser application.** Never point `run-nova.cmd`,
`cargo nova`, or `README.md`'s main browser launch instructions here.
The product UI, tabs, Settings, Home, bookmarks, history and session data are
in `crates/nova-desktop/` and must be preserved while engine integration
proceeds.

This harness can navigate to external sites through Servo directly. It has
only one Servo webview, so leaving its local Home replaces its UI.
It is not evidence that Servo can already be embedded behind NovaBrowser's
persistent toolbar or manage NovaBrowser tabs.

## Windows local test

Install Windows 10/11 x64, Rust 1.94+, Visual Studio C++ Build Tools /
Windows SDK and Servo's native dependencies. From repository root:

```cmd
cargo check --manifest-path tools/servo-embed-harness/Cargo.toml
cargo test --manifest-path tools/servo-embed-harness/Cargo.toml
node tools/servo-embed-harness/tests/frontend-smoke.cjs
cargo run --manifest-path tools/servo-embed-harness/Cargo.toml --release
```

The harness is pinned to `tauri-runtime-servo = 0.1.0` with
`servo = 0.5.0` for adapter compatibility. Do not claim that these are
the versions integrated into the real NovaBrowser product. The direct
Servo 0.8+ WebView API is being evaluated independently for the embedder.

## Benchmark

In another PowerShell window while the harness runs:

```powershell
powershell -ExecutionPolicy Bypass -File tools/servo-embed-harness/measure-memory.ps1 -Samples 15 -IntervalSeconds 2 -CsvPath servo-harness-memory.csv
```

For the actual NovaBrowser shell, rerun with `-ProcessName nova-desktop`.
Compare total process-tree private bytes, working set and page workloads.
Do not claim a RAM win from executable size or a single process reading.

## Security

Commands are accepted only from the packaged local start page.
Remote HTTP(S) origins should never be authorized for privileged Tauri
invocations. Do not use this harness for passwords or sensitive browsing.

See [NovaBrowser architecture](../../docs/ENGINE_DECISION.md).
