// Regression guard for native Servo tab sleeping and website focus shortcuts.
// Run: node crates/nova-desktop/ui/tests/tab-sleep.cjs
"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const root = path.resolve(__dirname, "../../../..");
const read = relative => fs.readFileSync(path.join(root, relative), "utf8");
const main = read("crates/nova-desktop/src/main.rs");
const adapter = read("crates/nova-desktop/src/servo_embed.rs");
const renderer = read("engines/servo-native/src/main.rs");
const ui = read("crates/nova-desktop/ui/app.js");
const css = read("crates/nova-desktop/ui/app.css");
const locale = read("crates/nova-desktop/ui/i18n.js");

// The tab stays in the browser's session, only the background WebView is dropped.
assert.match(main, /SleepTab\(String\)/);
assert.match(main, /fn browser_sleep_tab\(/);
assert.match(main, /browser_sleep_tab,/);
assert.match(main, /id != active_id/, "Never discard the active site");
assert.match(main, /tab\.restore_url = Some\(tab\.address\.clone\(\)\)/,
  "Retain the URL for lazy reactivation");
assert.match(main, /bridge\.send\(servo_embed::Control::Close \{ tab: id\.clone\(\) \}\)/,
  "Release the native WebView on sleep");
assert.match(main, /tab\.restore_url\.take\(\)/,
  "Awakening a tab reuses normal lazy session restore");
assert.match(main, /if tab\.restore_url\.is_some\(\) \{ continue; \}/,
  "Late notifications from sleeping pages must be ignored");
assert.match(main, /sleeping: tab\.restore_url\.is_some\(\)/,
  "Tab metadata must include its sleeping state");

// Make the action a trusted NovaBrowser chrome control, never page JavaScript.
assert.match(ui, /addEventListener\("contextmenu"/);
assert.match(ui, /browser_sleep_tab/);
assert.match(ui, /browser_activate_tab/);
assert.match(ui, /tab\.sleeping \? "moon"/);
assert.match(ui, /event\.key === "ContextMenu"/);
assert.match(ui, /event\.key === "Escape"/);
assert.match(css, /\.tab-context-menu\[hidden\]/);
assert.match(css, /:root\[data-theme="dark"\] \.tab-context-menu/);
assert.match(locale, /"Sleep tab": "Cho tab ngủ"/);
assert.match(locale, /"Wake tab": "Đánh thức tab"/);

// Native focus shortcuts go back through the same browser tab manager.
assert.match(renderer, /"event":"cycle_tab"/);
assert.match(renderer, /"event":"activate_tab_index"/);
assert.match(renderer, /KeyCode::Tab/);
assert.match(renderer, /KeyCode::Digit8/);
assert.match(adapter, /"cycle_tab" =>/);
assert.match(adapter, /"activate_tab_index" =>/);
assert.match(main, /ServoCycleTab\(bool\)/);
assert.match(main, /ServoActivateIndex\(usize\)/);
assert.match(main, /Command::ActivateTab\(tabs\[next\]\.id\.clone\(\)\)/);
assert.match(main, /if !show_page \{\s*let _ = chrome\.set_focus\(\);/);
assert.doesNotThrow(() => new Function(ui));
assert.doesNotThrow(() => new Function(locale));

console.log("NovaBrowser tab sleep and native tab shortcut contract checks passed");
