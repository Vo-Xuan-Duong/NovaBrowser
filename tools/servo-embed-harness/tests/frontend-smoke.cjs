// Dependency-free Node.js smoke checks for the Servo preview landing page.
// Run: node tools/servo-embed-harness/tests/frontend-smoke.cjs
"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

class Node {
  constructor() {
    this.value = "";
    this.textContent = "";
    this.disabled = false;
    this.handlers = {};
    this.attrs = {};
    this.classes = new Set();
    this.classList = { toggle: (key, enabled) => {
      if (enabled) this.classes.add(key);
      else this.classes.delete(key);
    }};
  }
  addEventListener(type, callback) { this.handlers[type] = callback; }
  getAttribute(name) { return this.attrs[name] || null; }
  focus() { this.focused = true; }
}

(async () => {
  const file = path.join(__dirname, "..", "ui", "app.js");
  const source = fs.readFileSync(file, "utf8");
  const ids = ["search-form", "address", "provider", "status", "go-button"];
  const nodes = Object.fromEntries(ids.map(id => [id, new Node()]));
  const shortcuts = ["https://github.com/", "https://www.wikipedia.org/"].map(target => {
    const button = new Node();
    button.attrs["data-target"] = target;
    return button;
  });
  const commands = [];
  const local = {
    __TAURI__: { core: { invoke: async (name, args) => {
      commands.push({ name, args });
      if (args.address === "fail") throw new Error("forced failure");
    }}},
  };
  const document = {
    getElementById: id => {
      assert.ok(nodes[id], "unknown element " + id);
      return nodes[id];
    },
    querySelectorAll: selector => selector === "[data-target]" ? shortcuts : [],
  };
  new Function("window", "document", source)(local, document);
  const flush = () => new Promise(resolve => setImmediate(resolve));

  nodes.provider.value = "google";
  nodes.address.value = "C++ tiếng Việt";
  nodes["search-form"].handlers.submit({ preventDefault() {} });
  await flush();
  assert.deepEqual(commands[0], {
    name: "servo_navigate",
    args: { address: "C++ tiếng Việt", searchEngine: "google" },
  });
  assert.equal(nodes["go-button"].disabled, false);

  shortcuts[0].handlers.click();
  await flush();
  assert.equal(commands[1].args.address, "https://github.com/");

  nodes.address.value = "fail";
  nodes["search-form"].handlers.submit({ preventDefault() {} });
  await flush();
  assert.equal(nodes["go-button"].disabled, false);
  assert.ok(nodes.status.classes.has("error"));
  assert.match(nodes.status.textContent, /forced failure/);

  nodes.address.value = "  ";
  nodes["search-form"].handlers.submit({ preventDefault() {} });
  await flush();
  assert.equal(commands.length, 3, "empty input must not navigate");
  assert.equal(nodes.address.focused, true);
  process.stdout.write("Servo embedding harness frontend checks passed\n");
})().catch(error => {
  process.stderr.write(String(error.stack || error) + "\n");
  process.exitCode = 1;
});
