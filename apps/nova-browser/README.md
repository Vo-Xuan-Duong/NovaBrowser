# NovaBrowser — Servo engine evaluation (Windows)

## Scope and rationale

NovaBrowser aims to browse real sites while keeping **total process-tree
memory usage** low. Servo is a candidate for that goal, **not an already
measured winner**. The current desktop uses a Tauri/WebView2 shell and
Obscura-rendered PNG page frames. Simply replacing its Tauri runtime will
not automatically give Servo-native website tabs.

The opt-in project in \`experiments/servo-preview/\` is therefore a real
**single-webview Servo browsing test**. It uses the published experimental
[tauri-runtime-servo](https://github.com/copse-dev/tauri-runtime-servo) crate
to render both its local start screen and external HTTPS websites through
Servo. It does not link the Obscura engine and does not require WebView2 as
its rendering backend. It is intentionally not the default NovaBrowser
executable.

The preview **reuses** the actual
\`crates/nova-desktop/src/navigation.rs\` URL/search normalization through a
Rust \`#[path]\` import rather than reimplementing its search rules.
The main NovaBrowser Home, Settings, bookmarks and history remain untouched
until Servo integration and its security posture are verified.

## Build and run

Install Windows 10/11 x64, the Visual Studio C++ toolchain and a current
64-bit stable Rust toolchain **1.94 or newer**. Servo's first build is
substantial and fetches SpiderMonkey and other libraries. The runtime is
experimental and may require extra Windows build dependencies.

From the repository root:

\`\`\`cmd
rustup update stable
cargo check --manifest-path experiments/servo-preview/Cargo.toml
cargo test --manifest-path experiments/servo-preview/Cargo.toml
node experiments/servo-preview/tests/frontend-smoke.cjs
cargo run --manifest-path experiments/servo-preview/Cargo.toml --release
\`\`\`

The experimental runtime 0.1.0 currently uses **Servo 0.5.0**, not the
latest Servo 0.8.x. These are intentionally pinned to the runtime's tested
dependency graph; upgrading the engine requires an adapter/runtime update.

The preview is **a separate Cargo workspace** with its own lockfile.
The ordinary \`cargo run -p nova-desktop --release\` and the main workspace
\`Cargo.lock\` are not modified by building it.

### Using the preview

1. Enter a URL or text query and choose Google, Bing or DuckDuckGo.
   Press Enter. Servo navigates the webview to the real external site.
2. To navigate again, use **Navigate > Nova Home / Address** or **Ctrl+L**
   in the native Windows menu. The menu persists when the webview
   displays an external site.
3. Navigate > Back/Forward (Alt+Arrow) and Reload (Ctrl+R) are native
   menu commands. The start screen's quick links are lazy.
4. Evaluate Google Search, GitHub, Wikipedia and pages containing Unicode
   text. Servo may still lack features or face Google anti-bot checks;
   this prototype does not attempt to evade either.

### Security boundaries

- No privileged Tauri capability is granted to remote origins; the
  capability manifest only permits the bundled local Home window.
- The \`servo_navigate\` Rust command additionally checks that the
  current webview URL is **exactly** the packaged Home URL before accepting
  navigation requests. HTTP/HTTPS is whitelisted; \`file:\`, \`data:\`,
  \`javascript:\` and other privileged URL schemes are forbidden.
- Avoid logging in to sensitive accounts in this experimental preview.
  It has not undergone a browser security review.

## Memory benchmark

Run the preview, then open a **second PowerShell window**:

\`\`\`powershell
powershell -ExecutionPolicy Bypass -File experiments/servo-preview/measure-memory.ps1 -Samples 15 -IntervalSeconds 2 -CsvPath servo-memory.csv
\`\`\`

Repeat under comparable conditions for the existing app:

\`\`\`powershell
powershell -ExecutionPolicy Bypass -File experiments/servo-preview/measure-memory.ps1 -ProcessName nova-desktop -Samples 15 -IntervalSeconds 2 -CsvPath obscura-memory.csv
\`\`\`

The script includes the **complete process tree** of each selected
application, sums private bytes and working set, and exports timestamped
samples. For valid comparisons, use the same Windows machine, similar
monitor resolution, network conditions and browsing workloads:

- Startup at Home after 15–30 seconds idle
- A static site like Wikipedia after it settles
- Google Search with the same query (if permitted)
- A JavaScript-heavy site after it settles
- Return Home and observe retained memory and process count

Compare **PrivateMB** (process-private committed bytes) as the primary
metric, and WorkingSetMB separately (physical pages may be shared).
Do not count only the root process or claim a memory win from binary size.
Servo currently supports just one webview in this preview, so comparing
the old multi-tab app with many active tabs is *not* yet apples-to-apples.

## Gaps before adopting Servo as the default engine

According to the upstream experimental runtime's README, **multiple
Servo webviews within one native window are not supported yet**.
NovaBrowser needs a persistent tab/address/Settings chrome *and* a
separate document surface, so switching its production Tauri runtime
immediately would break the existing interface. The native menu in this
preview is a temporary, test-only navigation surface.

Before replacing Obscura in the main application:

1. Verify Windows build, Google navigation, Unicode, forms, IME and
   page security behavior on real devices.
2. Confirm measurable memory benefits across identical workloads.
3. Design a compositor or native shell solution supporting a persistent
   chrome and multiple independently managed Servo pages. Do not rely
   on cross-origin iframe embedding of arbitrary websites.
4. Add Servo tab lifecycle/visibility/discard support, then connect
   existing bookmarks, history, settings and session data.
5. Retire the legacy engine only after feature parity, local tests and
   a migration path for the saved profile.

No GitHub Actions or automated workflows are required or added.
