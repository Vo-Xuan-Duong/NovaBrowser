# NovaBrowser

**NovaBrowser uses Servo as its only selected website engine.** Obscura source
is retained only for migration of useful browser features; it is **not** a
second browser under active development.

> Engineering build, not a production-ready browser. The current Servo
> adapter has one webview per native window. Permanent toolbar, multi-tab
> interface, bookmarks/history, Settings, localization and profile migration
> are not yet integrated into the new application. Do not browse sensitive
> accounts with this build.

## Run on Windows

Install Rust stable 1.94+, Visual Studio C++ build tools / Windows SDK, and
the native dependencies needed by Servo. In the repository root:

```bat
rustup update stable
run-nova.cmd
```

Equivalent commands launch the **same** application:

```bat
cargo nova
cargo run --manifest-path apps/nova-browser/Cargo.toml --release
```

Local tests (do not use GitHub Actions):

```bat
cargo check --manifest-path apps/nova-browser/Cargo.toml
cargo test --manifest-path apps/nova-browser/Cargo.toml
node apps/nova-browser/tests/frontend-smoke.cjs
```

The selected application lives at [`apps/nova-browser/`](apps/nova-browser/).
It has a separate Cargo workspace to isolate Servo dependencies from the
retained Obscura lockfile. **This is dependency isolation, not two apps.**
The current adapter pins Servo 0.5.0; upgrading Servo requires an
adapter integration change and Windows validation.

Servo provides native page rendering, scripting and HTTP(S) navigation.
Google, Bing and DuckDuckGo search work through the existing URL normalizer
when the destination site accepts the request. Use the native Navigate
menu, Ctrl+L (Home), Alt+Arrow (history) and Ctrl+R (reload) while the
local toolbar is not yet persistent.

The previous `crates/nova-desktop/` and `crates/obscura-*/` folders
are **legacy source** retained for migrating tabs, localization, settings,
bookmarks, history and session state. The old `cargo run -p nova-desktop`
command still runs Obscura, **not** the selected NovaBrowser application.

**No memory results have been measured.** To evaluate RAM on Windows, use
[`measure-memory.ps1`](apps/nova-browser/measure-memory.ps1) and compare
complete process-tree private committed memory and working sets under
identical browsing workloads.

See [engine decision](docs/ENGINE_DECISION.md),
[Servo integration notes](docs/Servo-engine-evaluation.md)
and [third-party notices](THIRD_PARTY_NOTICES.md).
