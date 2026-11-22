# NovaBrowser desktop MVP

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
