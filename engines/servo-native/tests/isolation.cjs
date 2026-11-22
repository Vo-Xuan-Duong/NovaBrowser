// Dependency-free guard against reintroducing the ICU version conflict.
// Run from the repository root: node engines/servo-native/tests/isolation.cjs
"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const root = path.resolve(__dirname, "../../..");
const load = name => fs.readFileSync(path.join(root, name), "utf8");

const desktop = load("crates/nova-desktop/Cargo.toml");
const workspace = load("Cargo.toml");
const engine = load("engines/servo-native/Cargo.toml");
const adapter = load("crates/nova-desktop/src/servo_embed.rs");
const renderer = load("engines/servo-native/src/main.rs");
const launcher = load("run-nova-servo.cmd");

assert.match(desktop, /^servo-embed\s*=\s*\[\s*\]\s*$/m,
  "servo-embed must not activate Servo crates in the V8 workspace");
assert.doesNotMatch(desktop, /^\s*(servo|servo-embedder-traits|mozjs|webrender_api)\s*=/m);
assert.doesNotMatch(desktop, /^\s*dep:servo/m);
assert.match(workspace, /"engines\/servo-native"/,
  "Servo must be excluded from the legacy Cargo workspace");
assert.match(engine, /^\[workspace\]\s*$/m,
  "Servo must own its own Cargo workspace and lockfile");
assert.match(engine, /^servo\s*=\s*"=0\.8\.0"\s*$/m);
assert.match(renderer, /^use embedder_traits::EventLoopWaker;$/m,
  "servo-embedder-traits exports the Rust crate as embedder_traits");
assert.doesNotMatch(renderer, /use servo_embedder_traits::/,
  "Do not import using the Cargo package's hyphenated name");

assert.doesNotMatch(engine, /^\s*(obscura-browser|deno_core|v8)\s*=/m);
assert.match(adapter, /strip_prefix\("NOVA\\t"\)/);
assert.match(renderer, /writeln!\(stdout,\s*"NOVA\\t\{event\}"\)/);
assert.match(adapter, /NOVA_SERVO_SIDECAR_BIN/);
assert.match(adapter, /stdin\(Stdio::piped\(\)\)/);
assert.match(adapter, /stdout\(Stdio::piped\(\)\)/);

for (const [op, variant] of [
  ["navigate", "Navigate"], ["select", "Select"],
  ["close", "Close"], ["back", "Back"],
  ["forward", "Forward"], ["reload", "Reload"],
  ["layout", "Layout"], ["shutdown", "Shutdown"],
]) {
  assert.ok(adapter.includes('"op":"' + op + '"'),
    "Browser must serialize " + op);
  assert.ok(renderer.includes("    " + variant + (variant === "Shutdown" ? "," : " {")),
    "Sidecar must accept " + variant);
}
for (const event of ["ready", "fatal", "location", "title", "history",
  "error", "focus_address", "new_tab", "reopen_tab", "close_tab",
  "back", "forward", "reload"]) {
  assert.ok(adapter.includes('"' + event + '"'),
    "Browser must accept " + event);
  assert.ok(renderer.includes('"' + event + '"'),
    "Renderer must emit " + event);
}
// Servo 0.8.0 has explicit one-step traversal, and WheelEvent takes exactly
// (WheelDelta, WebViewPoint); guard against API drift from newer examples.
assert.match(renderer, /page\.go_back\(1\);/,
  "Servo 0.8 back navigation requires a traversal amount");
assert.match(renderer, /page\.go_forward\(1\);/,
  "Servo 0.8 forward navigation requires a traversal amount");
assert.match(renderer, /WheelDelta \{ x, y, z: 0\.0, mode \},\s*state\.cursor\.into\(\),\s*\)\)/,
  "Servo 0.8 WheelEvent::new accepts delta and point only");
assert.doesNotMatch(renderer, /\bModifiers::default\(\)/,
  "Servo 0.8 WheelEvent no longer accepts a third modifiers argument");

assert.match(launcher, /engines\\servo-native\\Cargo\.toml/);
assert.match(launcher, /cargo run -p nova-desktop --release --features servo-embed/);
console.log("NovaBrowser Servo/Obscura Cargo isolation and IPC contract checks passed");
