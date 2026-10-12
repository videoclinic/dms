import test from "node:test";
import assert from "node:assert/strict";
import {
  appendBatchEntry,
  batchExecutionClick,
  batchExecutionMarkup,
  createBatchExecutionState,
  deleteShownBatchEntries,
  filterBatchEntries,
  visibleBatchEntries,
} from "./batch-execution.mjs";

const operation = {
  label: "Unregister 2 documents",
  targets: [
    { name: "Policy <A>.md", document_id: "doc-a", content: "SOURCE CONTENT" },
    { name: "Guide.docx", document_id: "doc-b", credential: "SYNTHETIC SECRET" },
  ],
  editRoot: "/private/Edit",
};

function populatedLog() {
  return ["success", "warning", "error"].reduce((state, type) => (
    appendBatchEntry(state, type, operation, `Message: ${type}`)
  ), createBatchExecutionState());
}

function target(selector, dataset = {}) {
  return { closest: (candidate) => candidate === selector ? { dataset } : null };
}

test("batch log appends immutable typed entries with stable IDs and only display identities", () => {
  const initial = createBatchExecutionState();
  const next = appendBatchEntry(initial, "success", operation, "Complete.");
  assert.equal(initial.entries.length, 0);
  assert.equal(next.filter, "all");
  assert.equal(next.entries[0].id, 1);
  assert.deepEqual(next.entries[0].targets, [
    { name: "Policy <A>.md", document_id: "doc-a" },
    { name: "Guide.docx", document_id: "doc-b" },
  ]);
  assert.notEqual(next.entries[0].targets[0], operation.targets[0]);
  assert.throws(() => appendBatchEntry(initial, "all", operation, "Invalid"), /Unknown/);
  assert.throws(() => appendBatchEntry(initial, "pending", operation, "Invalid"), /Unknown/);
  const cleared = deleteShownBatchEntries(next);
  assert.equal(appendBatchEntry(cleared, "error", operation, "Rejected.").entries[0].id, 2);
});

test("All, Success, Warning, and Error filter visibility without changing entries", () => {
  const state = populatedLog();
  for (const filter of ["all", "success", "warning", "error"]) {
    const filtered = filterBatchEntries(state, filter);
    assert.equal(filtered.entries, state.entries);
    assert.deepEqual(visibleBatchEntries(filtered).map((entry) => entry.type),
      filter === "all" ? ["success", "warning", "error"] : [filter]);
    const markup = batchExecutionMarkup(filtered);
    assert.match(markup, new RegExp(`data-batch-filter="${filter}" aria-pressed="true"`));
    assert.equal((markup.match(/aria-pressed="true"/g) ?? []).length, 1);
    for (const type of ["success", "warning", "error"]) {
      assert.equal(markup.includes(`Message: ${type}`), filter === "all" || filter === type);
    }
  }
  assert.equal(filterBatchEntries(state, "unknown"), state);
});

test("filter and delete clicks delete only shown messages and allow unrelated shell clicks", () => {
  const state = populatedLog();
  for (const filter of ["all", "success", "warning", "error"]) {
    const filtered = batchExecutionClick(state, target("[data-batch-filter]", { batchFilter: filter }));
    const deleted = batchExecutionClick(filtered, target("[data-batch-delete-shown]"));
    assert.deepEqual(deleted.entries, state.entries.filter((entry) => filter !== "all" && entry.type !== filter));
    assert.deepEqual(visibleBatchEntries(deleted), []);
    const markup = batchExecutionMarkup(deleted);
    assert.match(markup, /No messages shown/);
    assert.match(markup, /data-batch-delete-shown disabled/);
    assert.equal(filterBatchEntries(deleted, "all").entries.length, filter === "all" ? 0 : 2);
  }
  assert.equal(batchExecutionClick(state, target("[data-activity-remove]")), state);
  assert.equal(batchExecutionClick(state, target("[data-activity-open]")), state);
  assert.equal(state.entries.length, 3);
});

test("batch markup uses an accessible log and escapes identities without leaking other payload fields", () => {
  const markup = batchExecutionMarkup(populatedLog());
  assert.match(markup, /role="log" aria-label="Batch execution messages" aria-live="polite"/);
  assert.match(markup, /role="group" aria-label="Filter batch messages"/);
  assert.match(markup, /Policy &lt;A&gt;\.md/);
  assert.doesNotMatch(markup, /Policy <A>|SOURCE CONTENT|SYNTHETIC SECRET|\/private\/Edit/);
});
