// Dependency-free regression test for NovaBrowser's native content boundary.
// Run: node crates/nova-desktop/ui/tests/servo-native-surface.cjs
"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(path.join(__dirname, "..", "app.js"), "utf8");
// Parse all frontend JavaScript, including the optional native Servo path.
assert.doesNotThrow(() => new Function(source));

const start = source.indexOf("  function scheduleNativeLayout(force = false) {");
const end = source.indexOf("\n  let libraryMode =", start);
assert.ok(start >= 0 && end > start, "native layout function is present");
const body = source.slice(start, end);
const create = new Function(
  "pageArea", "settingsView", "libraryPanel", "blockedPanel",
  "requestAnimationFrame", "call", "currentTabAddress",
  '"use strict"; let nativeServo=true; let nativeLayoutQueued=false; '
  + 'let lastNativeLayout=""; ' + body
  + ';return {scheduleNativeLayout};'
);

function fixture({ url = "https://example.org/", pageHidden = false,
  settings = false, library = false, blocked = false, top = 111.5 } = {}) {
  const frames = [];
  const calls = [];
  const controls = create(
    { hidden: pageHidden, getBoundingClientRect: () => ({ top }) },
    { isOpen: () => settings }, { hidden: !library }, { hidden: !blocked },
    callback => frames.push(callback),
    (name, args) => { calls.push({ name, args }); return Promise.resolve(); },
    url,
  );
  return {
    ...controls,
    calls,
    flush() { while (frames.length) frames.shift()(); },
  };
}

const web = fixture();
web.scheduleNativeLayout();
web.flush();
assert.deepEqual(web.calls, [{
  name: "browser_surface_layout",
  args: { top: 111.5, visible: true },
}]);
web.scheduleNativeLayout();
web.flush();
assert.equal(web.calls.length, 1, "identical layout is deduplicated");
web.scheduleNativeLayout(true);
web.flush();
assert.equal(web.calls.length, 2, "window resize forces an updated layout");

for (const options of [
  { url: "nova://home", pageHidden: true },
  { settings: true },
  { library: true },
  { blocked: true },
]) {
  const state = fixture(options);
  state.scheduleNativeLayout();
  state.flush();
  assert.equal(state.calls.length, 1);
  assert.equal(state.calls[0].args.visible, false,
    "local browser panels must hide the Servo child window");
}

assert.match(source, /browser_get_engine_mode/, "native engine mode negotiated");
assert.match(source, /if \(nativeServo\) return; \/\/ Servo receives native mouse events/);
assert.match(source, /if \(nativeServo\) return;\s*event\.preventDefault\(\)/);
assert.match(source, /listen\("browser-frame", \(\{ payload \}\) => \{\s*if \(nativeServo\) return;/);

const html = fs.readFileSync(path.join(__dirname, "..", "index.html"), "utf8");
const settings = fs.readFileSync(path.join(__dirname, "..", "settings.js"), "utf8");
assert.doesNotThrow(() => new Function(settings));
assert.match(html, /id="settings-engine-value"/);
assert.match(html, /id="settings-engine-description"/);
assert.match(html, /id="settings-engine-status"/);
assert.match(settings, /invoke\("browser_get_engine_info"\)/);
assert.match(settings, /engine\?\.mode === "servo"/);
assert.match(settings, /engine\?\.running/);

console.log("NovaBrowser native Servo surface contracts passed");
