const FILTERS = ["all", "success", "warning", "error"];

export const UNREGISTER_WARNING = "Unregister removes library membership only. It does not delete source files, stable IDs, document control data, workflow/release history, or checksums.";
export const BATCH_FAILURE = "Batch request failed; success was not confirmed. The selection is retained. Check the selected files and workspace access before trying again.";

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function createBatchExecutionState() {
  return { session: Symbol("batch activity"), entries: [], next_id: 1, filter: "all" };
}

export function appendBatchEntry(state, type, operation, message) {
  if (!FILTERS.includes(type) || type === "all") throw new Error("Unknown batch message type.");
  return {
    ...state,
    next_id: state.next_id + 1,
    entries: [...state.entries, {
      id: state.next_id,
      type,
      action: operation.label,
      targets: operation.targets.map(({ name, document_id }) => ({ name, document_id })),
      message,
    }],
  };
}

export function filterBatchEntries(state, filter) {
  return FILTERS.includes(filter) ? { ...state, filter } : state;
}

export function visibleBatchEntries(state) {
  return state.entries.filter((entry) => state.filter === "all" || entry.type === state.filter);
}

export function deleteShownBatchEntries(state) {
  return {
    ...state,
    entries: state.filter === "all" ? [] : state.entries.filter((entry) => entry.type !== state.filter),
  };
}

export function batchExecutionClick(state, target) {
  const filter = target.closest("[data-batch-filter]")?.dataset.batchFilter;
  if (filter) return filterBatchEntries(state, filter);
  if (target.closest("[data-batch-delete-shown]")) return deleteShownBatchEntries(state);
  return state;
}

export function batchExecutionMarkup(state) {
  const visible = visibleBatchEntries(state);
  const filters = FILTERS.map((filter) => `<button class="text-button" type="button" data-batch-filter="${filter}" aria-pressed="${state.filter === filter}">${filter[0].toUpperCase()}${filter.slice(1)}</button>`).join("");
  const entries = visible.map((entry) => {
    const targets = entry.targets.map((target) => `<li>${escapeHtml(target.name)}${target.document_id ? ` · ${escapeHtml(target.document_id)}` : ""}</li>`).join("");
    return `<li data-batch-entry="${entry.id}"><article><h3><span class="badge">${entry.type[0].toUpperCase()}${entry.type.slice(1)}</span> ${escapeHtml(entry.action)}</h3><p>${escapeHtml(entry.message)}</p><ul class="identity-list">${targets}</ul></article></li>`;
  }).join("");
  return `<section class="card" aria-labelledby="batch-execution-heading"><h2 id="batch-execution-heading">Batch execution</h2><p>Session-only messages, not audit evidence. Closing this pane or leaving this workspace discards them.</p><div role="group" aria-label="Filter batch messages">${filters}</div><button class="text-button" type="button" data-batch-delete-shown ${visible.length ? "" : "disabled"}>Delete shown messages</button><div role="log" aria-label="Batch execution messages" aria-live="polite" aria-relevant="additions text">${entries ? `<ol>${entries}</ol>` : "<p>No messages shown.</p>"}</div></section>`;
}
