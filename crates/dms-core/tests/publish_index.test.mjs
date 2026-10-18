import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const script = readFileSync(new URL("../src/publish_index.js", import.meta.url), "utf8");

// Minimal DOM port: execute the actual embedded script and its event handlers.
// This tests control behavior, not browser layout or accessibility tooling.
class Element {
  constructor(dataset = {}) {
    this.dataset = dataset;
    this.listeners = new Map();
    this.attributes = new Map();
    this.children = [];
    this.value = "";
    this.checked = true;
    this.hidden = false;
  }
  addEventListener(name, handler) { this.listeners.set(name, handler); }
  fire(name) { this.listeners.get(name)(); }
  setAttribute(name, value) { this.attributes.set(name, value); }
  querySelectorAll() { return this.children; }
  replaceChildren(...children) { this.children = children; }
}

function fixture(empty = false) {
  const data = empty ? [] : [
    { id: "new", folder: "Policies/People", type: "internal", dateOrder: "0", nameOrder: "3", cells: ["Zulu handbook", "HR-42", "V2.1", "2026-03-01T00:00:00Z", "internal", "Policies/People/Zulu.pdf"] },
    { id: "root", folder: "", type: "public", dateOrder: "1", nameOrder: "0", cells: ["Alpha", "ROOT-2", "V1.0", "2026-02-01T00:00:00Z", "public", "Root.pdf"] },
    { id: "child", folder: "Policies", type: "public", dateOrder: "2", nameOrder: "1", cells: ["Beta", "POL-7", "V1.1", "2026-01-01T00:00:00Z", "public", "Policies/Beta.pdf"] },
    { id: "similar", folder: "Policies-other", type: "internal", dateOrder: "3", nameOrder: "2", cells: ["Gamma", "OTHER-9", "V1.2", "2025-01-01T00:00:00Z", "internal", "Policies-other/Gamma.pdf"] },
  ];
  const elements = Object.fromEntries(["documents", "search", "sort", "all-types", "result-count", "empty"].map((id) => [id, new Element()]));
  elements.sort.value = "date-desc";
  elements.documents.children = data.map(({ cells, ...dataset }) => {
    const node = new Element(dataset);
    node.children = cells.map((textContent) => ({ textContent }));
    return node;
  });
  const types = [...new Set(data.map((row) => row.type))].map((type) => new Element({ type }));
  const folders = ["", "Policies", "Policies/People", "Policies-other"].map((folder) => new Element({ folder }));
  const document = {
    getElementById: (id) => elements[id],
    querySelectorAll: (selector) => {
      if (selector === "input[data-type]") return types;
      if (selector === "button[data-folder]") return folders;
      throw new Error(`Unexpected selector: ${selector}`);
    },
  };
  vm.runInNewContext(script, { document }, { timeout: 1000 });
  return {
    ...elements, types, folders,
    ids: () => elements.documents.children.map((node) => node.dataset.id),
    searchFor(value) { elements.search.value = value; elements.search.fire("input"); },
    sortBy(value) { elements.sort.value = value; elements.sort.fire("change"); },
    folder(path) { folders.find((button) => button.dataset.folder === path).fire("click"); },
    type(id, checked) { const input = types.find((input) => input.dataset.type === id); input.checked = checked; input.fire("change"); },
    all(checked) { elements["all-types"].checked = checked; elements["all-types"].fire("change"); },
  };
}

test("publish_index starts newest-first and sorts both fields in both directions", () => {
  const f = fixture();
  assert.deepEqual(f.ids(), ["new", "root", "child", "similar"]);
  f.sortBy("date-asc");
  assert.deepEqual(f.ids(), ["similar", "child", "root", "new"]);
  f.sortBy("name-asc");
  assert.deepEqual(f.ids(), ["root", "child", "similar", "new"]);
  f.sortBy("name-desc");
  assert.deepEqual(f.ids(), ["new", "similar", "child", "root"]);
  f.sortBy("date-desc");
  assert.deepEqual(f.ids(), ["new", "root", "child", "similar"]);
});

test("publish_index searches every displayed metadata field case-insensitively", () => {
  const f = fixture();
  for (const query of ["ZULU HANDBOOK", "hr-42", "v2.1", "2026-03-01", "policies/people/zulu.pdf"]) {
    f.searchFor(query);
    assert.deepEqual(f.ids(), ["new"], query);
    assert.equal(f["result-count"].textContent, "1 documents");
  }
  f.searchFor("INTERNAL");
  assert.deepEqual(f.ids(), ["new", "similar"]);
  f.searchFor("  ");
  assert.equal(f.ids().length, 4);
});

test("publish_index folder buttons select exact subtrees, not similarly prefixed siblings", () => {
  const f = fixture();
  f.folder("Policies");
  assert.deepEqual(f.ids(), ["new", "child"]);
  f.folder("Policies/People");
  assert.deepEqual(f.ids(), ["new"]);
  assert.equal(f.folders.find((button) => button.dataset.folder === "Policies/People").attributes.get("aria-pressed"), "true");
  assert.equal(f.folders.filter((button) => button.attributes.get("aria-pressed") === "true").length, 1);
  f.folder("");
  assert.deepEqual(f.ids(), ["new", "root", "child", "similar"]);
});

test("publish_index type on/off and All controls preserve explicit empty selection", () => {
  const f = fixture();
  f.type("internal", false);
  assert.deepEqual(f.ids(), ["root", "child"]);
  assert.equal(f["all-types"].checked, false);
  assert.equal(f["all-types"].indeterminate, true);
  f.type("public", false);
  assert.deepEqual(f.ids(), []);
  assert.equal(f.empty.hidden, false);
  assert.equal(f["all-types"].indeterminate, false);
  f.all(true);
  assert.equal(f.ids().length, 4);
  assert.equal(f["all-types"].checked, true);
  f.all(false);
  assert.deepEqual(f.ids(), []);
  assert.equal(f["result-count"].textContent, "0 documents");
  f.type("internal", true);
  assert.deepEqual(f.ids(), ["new", "similar"]);
});

test("publish_index combines search, type, subtree and sort before count and empty state", () => {
  const f = fixture();
  f.folder("Policies");
  f.searchFor("V");
  f.sortBy("date-asc");
  assert.deepEqual(f.ids(), ["child", "new"]);
  f.type("internal", false);
  assert.deepEqual(f.ids(), ["child"]);
  assert.equal(f["result-count"].textContent, "1 documents");
  assert.equal(f.empty.hidden, true);
  f.searchFor("HR-42");
  assert.deepEqual(f.ids(), []);
  assert.equal(f["result-count"].textContent, "0 documents");
  assert.equal(f.empty.hidden, false);
  f.type("internal", true);
  assert.deepEqual(f.ids(), ["new"]);
  assert.equal(f.empty.hidden, true);
});

test("publish_index empty catalog is usable and disables the vacuous All filter", () => {
  const f = fixture(true);
  assert.deepEqual(f.ids(), []);
  assert.equal(f["result-count"].textContent, "0 documents");
  assert.equal(f.empty.hidden, false);
  assert.equal(f["all-types"].checked, false);
  assert.equal(f["all-types"].disabled, true);
  f.searchFor("anything");
  f.sortBy("name-desc");
  f.folder("");
  assert.deepEqual(f.ids(), []);
});
