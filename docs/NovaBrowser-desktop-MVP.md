# NovaBrowser desktop MVP (legacy Obscura reference)

> Superseded as the active browser architecture by
> [ADR-001: Servo engine](ENGINE_DECISION.md). Retained for UI,
> settings, i18n, bookmarks, history and profile migration.


## Intent

Build an independent native desktop browser shell on top of the Rust engine
retained from Obscura. Retain the existing engine crates and add NovaBrowser
components alongside them.

## Native window / page pipeline

```text
Tauri window (HTML/CSS chrome, WebView2 on Windows)
    -> Tauri commands and events
        -> Engine thread (Tokio current_thread + V8)
            -> obscura-browser::Page (navigation / JS / DOM)
                -> screenshot(viewport) -> PNG frame event
```

The Tauri webview draws only NovaBrowser's application interface. Website
content continues to come from the Obscura engine as rendered PNG frames.
A single engine thread owns `Page` and its V8 isolate. The frontend sends
navigation and input commands, and receives status, title, history, and frame
events.

## Functionality and security

- Chrome-inspired custom title bar, window controls, tab strip, toolbar and omnibox.
- Address bar: HTTP(S), hostname shortcuts and search terms.
- Browser navigation: Back, Forward, Reload, page URL and title.
- Local bookmarks and browsing history, stored in the app's local data folder.
- Dedicated, full-width browser Settings page with sections for General,
  Appearance (light/dark/system), Search engine (Google/Bing/DuckDuckGo), Tabs,
  Privacy & data and About.
- Configurable startup mode, homepage, new-tab behavior, local history recording,
  and a live theme preview with persistence.
- Restores open tabs and the selected tab across restarts. Inactive restored
  tabs are loaded on demand rather than fetching all sites at startup.
- Clear browsing history without removing saved bookmarks.
- Provisional pointer events, scrolling and simple editable text inputs.
- Obscura's default SSRF restrictions remain enabled; localhost and private IP
  URLs may be blocked by the engine.
- `file:`, `javascript:` and other unsupported schemes are rejected from the
  browser's address bar.

This is not a secure multi-process browser. Do not treat this preview as a
production security boundary or a replacement for Chromium/Firefox.

## Known limitations

- Rendering uses PNG screenshots rather than an efficient GPU compositor.
- Up to eight tabs are supported. Each tab has a separate DOM/JS page state;
  tabs share cookies through one browser context.
- Session restore persists only tab URLs/titles and the selected tab in
  `browser-data.json`. It does **not** restore DOM, scroll position, form
  input, JS heap state or per-tab navigation history. A restored tab reloads
  its website on first activation.
- Site sandboxing and process isolation are not present.
- DOM-level synthetic input is incomplete, especially for IME, text editing,
  selection, drag/drop, complex frameworks and some form controls.
- Back/Forward are managed by the shell, not full engine session history.
- Animated/interactive content receives a periodic best-effort refresh.
- Site compatibility is limited by the current browser engine.
- Tauri's webview is used for browser chrome only; it does not render websites.

## Settings audit

The settings panel configures only the browser's native chrome and new-tab
homepage. Light/dark is **not** a forced color scheme for websites rendered
by the Obscura engine. Changing the homepage does not navigate the current
tab or overwrite restored tabs.

Settings now show a live appearance preview and revert it when the dialog is
cancelled, closed or dismissed. A theme is persisted only after Save succeeds.
Homepage validation rejects blank and overlong input before invoking Rust;
the Rust command also enforces the 4,096-byte bound and normalizes
HTTP(S) URLs/search queries. Invalid saved theme/homepage values fall back to
defaults while retaining other stored profile data.

The save button is disabled during writes. If a settings write fails, the
in-memory settings value is restored and the panel displays an error; no
successful save is reported. The profile file itself is still written using
ordinary filesystem writes and is not yet crash-safe/atomic.

The popup is non-modal: click outside or Escape to dismiss without saving.
Use Tab to move through form controls; its open/close buttons restore focus
for keyboard users.

## Local validation

```powershell
cargo check -p nova-desktop
cargo test -p nova-desktop
cargo run -p nova-desktop --release
```

On Windows, Tauri uses Microsoft Edge WebView2 for the interface. WebView2 is
included with supported Windows versions or can be installed separately. The
Obscura engine still requires the dependencies needed to build V8.

## Tokio reactor regression (legacy Obscura desktop)

The engine thread owns a `tokio::runtime::Runtime`, but user input calls
`Page::evaluate` synchronously. JavaScript event listeners can immediately
start an asynchronous fetch/XHR operation while `runtime.block_on` is not
active. Without an entered Tokio runtime, the `op_fetch_url` FFI callback can
panic with `there is no reactor running`, aborting the whole process.

Keep the `runtime.enter()` guard alive on the engine thread, starting **before**
creating the browser context or pages and ending **after** page teardown. The
entered context makes the Tokio reactor available to synchronous callbacks;
`runtime.block_on` is still necessary to *drive* spawned tasks. Do not move
these callbacks to an unrelated thread or drop the guard while pages exist.

### Manual Windows reproduction check

Run a temporary local HTTP server from the repo root:

```powershell
python -m http.server 8765 --directory crates/nova-desktop/tests/fixtures
```

In a second terminal:

```powershell
$env:OBSCURA_ALLOW_PRIVATE_NETWORK = "1"
$env:RUST_BACKTRACE = "1"
cargo run -p nova-desktop --release
```

Visit `http://127.0.0.1:8765/tokio-reactor-input.html`. Click the large
**Fetch on click** button and type into **Fetch on input**. Both actions should
change their respective result text to **Fetch OK (HTTP 200)**, and the
`nova-engine` process should remain alive. Repeat after switching tabs,
reloading, and navigating Back/Forward. If the page is refused, check the
private-network opt-in and local server before investigating the runtime.
The fixture does not access an external site and does not measure browser
compatibility, memory usage, or Servo behavior. Run actual Windows cargo
check/tests as well; CI is intentionally disabled for this project.

## Session restore and startup synchronization

The frontend registers Tauri event listeners before calling
`browser_frontend_ready`. This resolves the prior startup race where the
engine could publish the initial tab list before the webview was listening.

The browser stores open tab URLs and titles, plus the active tab index,
alongside history/bookmarks/settings in the app-local `browser-data.json`.
Session snapshots are written when tab selection/closure or navigation changes,
and during orderly engine shutdown. First launch (no session) uses the saved
homepage. Restoring a session reloads the selected page immediately, while
inactive pages reload only when activated. Invalid/privileged URL schemes,
unreasonably long URLs and tabs beyond the eight-tab limit are ignored.

Closing the last tab is intentionally disabled; restoring tab metadata is not
full crash/session recovery. This preview does not encrypt the profile file,
restore form/scroll/JS state, or guarantee recovery from an unclean shutdown.


## Icon system

NovaBrowser uses a curated selection of official Lucide SVG shapes (ISC/MIT)
in `ui/icons.js`, self-hosted without a CDN, npm install or runtime download.
All static icon slots are marked `data-icon="name"` in `ui/index.html`.
The UI bootstraps them with `window.NovaIcons.hydrate()`; dynamically created
tabs/bookmarks use `window.NovaIcons.create("name")`.

The registry exposes `NovaIcons.names` to inspect available glyphs and uses
CSS `currentColor` for consistent Light/Dark appearance. New glyphs can be
added from the full Lucide catalog at https://lucide.dev/icons/ using the
official SVG data, while keeping the project bundle small. Avoid substituting
a lock or shield icon for browser security indicators: these require actual
certificate/security-state integration before they can convey trust.

Source provenance and notices are in `THIRD_PARTY_NOTICES.md` and
`crates/nova-desktop/ui/LUCIDE_LICENSE`.

## Full Settings page (native route)

The former two-field popup has been replaced by a full-width settings workspace
inside the Tauri browser chrome. Open it from the Settings toolbar icon, Ctrl+,
or by entering `nova://settings` (optionally with a section suffix such as
`nova://settings/privacy`). This internal route is handled by the UI and is
never sent to Obscura as a network URL. The current browsing tab remains intact.

The page uses a sidebar and separate content views:

- **General:** startup mode (restore previous session or open homepage) and
  editable homepage URL/search query.
- **Appearance:** system, light or dark theme; preview changes immediately
  and revert them when exiting without saving. System mode observes the
  operating system preference.
- **Search engine:** Google, Bing or DuckDuckGo for omnibox search queries.
- **Tabs:** choose homepage or a blank tab when opening new tabs.
- **Privacy & data:** stop recording future visits; clear locally saved
  browsing history explicitly without deleting bookmarks. This does **not**
  clear cookies, site storage or form state.
- **About:** runtime/engine information and implemented keyboard shortcuts.

Changes are saved through Tauri's existing `browser_save_settings` command
and persisted with the existing bookmarks/session profile. New settings use
Serde defaults so older profile files continue to load. All select values
and homepage size/scheme are validated again in Rust. Reset changes restores
the last persisted values, while Back to browsing discards unsaved previews.
Page rendering remains handled by Obscura, not by WebView2.

## Settings visual refinement and toolbar controls

Settings no longer uses large decorative cards and artificial status headers.
It has a compact sidebar with a functional section search, understated page
heading, slim bordered setting groups, and a bottom save bar. Save/Discard
are enabled only when settings differ from persisted values. Three native
radio controls offer System/Light/Dark theme previews with keyboard access.
The About panel documents actual capabilities; unsupported cookie/permission
controls are not falsely exposed as working preferences.

The Windows titlebar now keeps its Minimize, Maximize and Close buttons
in the same 42px row as the tabs, instead of visually overlapping the
toolbar. While Settings is open, the current tab displays "Settings"
and restores the original title and icon upon returning.

Two new persisted Appearance settings have visible behavior:
- **Show Home button** adds a functional home action to the browser navigation
  controls. Clicking it navigates to the saved homepage.
- **Show bookmarks bar** renders locally saved bookmarks in a toolbar below
  the omnibox. Adding, removing or clearing bookmarks refreshes the bar.

Privacy & data now includes separate confirmation-based controls to clear
history or all bookmarks. Clearing one does not silently remove the other.
The Settings search filters actual sections by their text; it does not search
the web or claim unsupported engine preferences.

Local verification checklist: open Settings, search for "homepage" or
"history", switch theme and discard, save/show Home and bookmark bar, visit
the home URL, add/remove a bookmark, clear bookmark/history separately,
resize the Tauri window and inspect titlebar control alignment, and restart
to confirm preferences survive. Test the engine with cargo check/test locally;
no GitHub Actions workflows are used.

## Browser interface languages

NovaBrowser now supports English and Vietnamese for the native browser
interface, including the tab strip, navigation tooltips, bookmark/history
library, settings navigation, preference labels, confirmations and other
UI-generated messages. This does **not** translate website content or
Obscura/V8 engine diagnostic messages.

In `nova://settings/language` choose **Follow system language** (the default),
**Tiếng Việt**, or **English**. The system option uses the WebView's primary
preferred language: Vietnamese for `vi`/`vi-*`, English otherwise. Changes
are previewed immediately; Save persists them to the existing user profile,
while Discard or Close restores the last saved preference. Older profiles
without the `language` property migrate to `system`, and unsupported locale
values are replaced with `system` during load. No restart, network
connection, external translation service, or package download is needed.

Translations are stored in `ui/i18n.js` as a keyed Vietnamese catalog with
English source-text fallbacks. UI text is localized without replacing icon
elements. Dynamically generated tab labels, history dates and bookmark
notifications use the same catalog. The code is structured for additional
locale catalogs later. Do not use this preference to claim full website
translation or set a misleading web security indicator.

## NovaBrowser Home (internal new tab)

NovaBrowser has a dedicated local Home/new-tab page under `nova://home`.
It is not a screenshot of a website and is **never** fetched over HTTP or
sent to an Obscura JavaScript context. The trusted Tauri UI renders it in
`ui/index.html`, `ui/home.js`, and `ui/app.css`. Its features are
wired to existing Rust endpoints:

- Search / address input submits to `browser_navigate` and respects the
  selected search engine; `nova://home` is a whitelisted internal URL.
- Quick access displays real saved bookmarks; when none exist it clearly
  labels public suggestions (GitHub, Wikipedia, MDN and YouTube).
- Recently visited shows at most four distinct entries from the local
  history store. Nothing is invented when there is no history.
- View Bookmarks, View History and Customize open actual browser controls.
- English/Vietnamese localization and light/dark appearance are shared
  with the rest of the browser chrome.

The default homepage is now `nova://home`, and the new-tab destination
offers three choices: **NovaBrowser Home**, configured homepage or blank.
To avoid replacing a user's actual work, session restoration continues
to honor saved web tabs; only a legacy demo-only profile whose every open tab
is the shipped Example Domain page is migrated to Home. Older settings with
the shipped demo homepage are also migrated to the new local address.
This does not affect non-demo browsing histories or saved bookmarks.

The Rust tab navigator records Home in its history, publishes a blank
screenshot to clear stale website frames, and skips DOM input/screenshot
polling while Home is selected. Back/Forward/Reload works with the virtual
URL. Session persistence accepts exactly `nova://home`; other internal,
`file:`, `data:`, and `javascript:` schemes stay forbidden.

Implemented additional shortcuts: **Ctrl+Shift+T** reopens the most
recently closed tab (up to 16 in-memory entries; reloads its URL, not
form/JS/scroll state); **Ctrl+Tab**/**Ctrl+Shift+Tab** and
**Ctrl+PageDown**/**Ctrl+PageUp** switch tabs, **Ctrl+1–8** activates a tab
by index, and **Alt+Home** navigates to the configured homepage.

### Local regression checks

1. Build with `cargo check -p nova-desktop` and
   `cargo test -p nova-desktop` on Windows.
2. On a fresh profile, verify `nova://home` opens without a network request.
3. Type a URL/search in the Home search field; verify the address bar, tab
   title, Back and Forward work, including returning to Home.
4. Create multiple tabs, switch with keyboard shortcuts, close and reopen
   with Ctrl+Shift+T.
5. Bookmark a page, return to Home and verify the new shortcut. Navigate
   from a shortcut and from a recent-history row.
6. Switch English/Vietnamese and Light/Dark; verify Home and Settings.
7. Set the New Tab preference to NovaBrowser Home, homepage or blank and
   verify each behavior, then restart to verify session restoration.
8. Resize the window and confirm Home does not show an image from another
   tab. The Obscura website renderer still has unresolved glyph/font
   fallbacks for some non-Latin pages and is not production-ready.


## Windows Unicode/CJK font fallback (engine improvement)

The Obscura renderer originally loaded only its embedded Latin-centric
Liberation/DejaVu faces (and an on-demand emoji font). This can produce
missing-glyph boxes for Chinese, Japanese, Korean and other scripts.

NovaBrowser now calls obscura_render::configure_font_files before the
first page/render pass, selecting a short, known list of installed Windows
fonts (YaHei, JhengHei, SimSun, MingLiU, Meiryo/Yu Gothic, Malgun Gothic,
Leelawadee UI, Nirmala, Mangal, Segoe UI and Segoe UI Symbol). Missing
fonts are simply skipped. The engine accepts only regular OpenType/TrueType
files and limits an individual selected file to 48 MiB and the total
selected font data to 128 MiB. It does not recursively scan every installed
system font or download additional fonts.

The renderer's existing bundled fonts still govern ordinary Latin text;
the additional system faces are available for cosmic-text's missing-glyph
fallback. This means display may vary by installed Windows fonts, and it is
not a guarantee of full-script, color-emoji or CSS web-font fidelity. Other
platforms currently keep the original embedded-font behavior.

Windows validation: run cargo check -p nova-desktop,
cargo test -p nova-desktop, cargo test -p obscura-render --features paint
selected_font_file_tests, and cargo run -p nova-desktop --release.
Visit an HTTP(S) page containing mixed English, simplified/traditional
Chinese, Japanese, Korean, Vietnamese and Thai text. Confirm the previous
missing-glyph boxes are reduced, and compare text wrapping and baseline
alignment. If the font names differ on the local machine, inspect
%WINDIR%\Fonts. It is safe for a requested optional font to be absent.
A separate renderer unit test covers invalid/oversized files, extension
filtering and duplicate paths; the desktop unit tests check font selection.

The site-content renderer still relies on CPU PNG screenshots and synthetic
DOM input. This fallback work does not resolve native IME, GPU
compositing, security isolation, or every possible Unicode font issue.


## Google search challenge handling

When the active search provider is Google, Obscura sends the normal search
URL `https://www.google.com/search?q=...`. Google can refuse that request
and redirect to `https://www.google.com/sorry/index` with an unusual-traffic
warning. This is a **server-side anti-bot restriction**, not evidence that
the address bar encoded the query incorrectly. NovaBrowser does not spoof
Chrome fingerprints, defeat verification or automatically rotate identities.

To handle it clearly:

- `navigation::blocked_google_search_query` recognizes only a genuine
  HTTPS Google search followed by Google's own `/sorry` path. It
  validates both origins, extracts and size-checks the search query, and
  rejects lookalike hostnames or off-domain `continue` URLs.
- The engine emits `browser-search-blocked` for the active tab and shows a
  native explanatory screen instead of painting a confusing empty Google
  challenge image. Its state is per-tab and restored when switching tabs.
- **Retry Google** retries the same query. **View Google's response** shows
  the unmodified page from Google so the user can decide how to proceed.
- **Search with Bing** / **Search with DuckDuckGo** sends that query to the
  selected service **only after a click**. The default search-engine setting
  is not changed and no fallback is triggered automatically.
- URL/search navigation, provider selection, and blocked-redirect recognition
  are covered by focused Rust unit tests. The screen is localized in
  English/Vietnamese.

The Google provider remains selectable, but Google may still block searches
from this experimental engine. Official search services can impose their
own traffic restrictions; the application does not promise to bypass them.
The notice does not request or transmit CAPTCHA tokens, passwords or account
data. No additional web-search API credentials or services are added.

Test locally with `cargo test -p nova-desktop` and
`cargo check -p nova-desktop`. Search a Unicode query in Google and verify
a normal result is rendered unchanged when Google accepts it. If Google
redirects to `/sorry`, verify the notice, manual Bing/DuckDuckGo choices,
Retry Google, View Google's response, Back, Forward, and tab switching.
