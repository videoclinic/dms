function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function formatTimestamp(value) {
  const timestamp = new Date(value);
  if (Number.isNaN(timestamp.getTime())) return String(value ?? "");
  return new Intl.DateTimeFormat("en-GB", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(timestamp);
}

export function createNoteDocumentState() {
  return {
    detail: null,
    editing_id: null,
    editing_body: null,
    delete_id: null,
    compose_body: "",
    compose_author: "",
    loading: false,
    error: "",
  };
}

export function noteDocumentState(noteDocuments, documentId) {
  return noteDocuments[documentId] ?? createNoteDocumentState();
}

export function updateNoteDocumentState(noteDocuments, documentId, changes) {
  return {
    ...noteDocuments,
    [documentId]: {
      ...noteDocumentState(noteDocuments, documentId),
      ...changes,
    },
  };
}

export function applyDocumentNotes(noteDocuments, detail) {
  return updateNoteDocumentState(noteDocuments, detail.document_id, {
    detail,
    editing_id: null,
    editing_body: null,
    delete_id: null,
    compose_body: "",
    compose_author: "",
    loading: false,
    error: "",
  });
}

function noteMarkup(note, state) {
  const metadata = `${formatTimestamp(note.created_at)} — ${note.author}`;
  if (state.editing_id === note.id) {
    return `<article class="document-note" data-note-id="${escapeHtml(note.id)}"><header><span>${escapeHtml(metadata)}</span></header><form id="document-note-edit-form" data-note-id="${escapeHtml(note.id)}" class="note-edit-form"><label for="edit-note-${escapeHtml(note.id)}">Edit note</label><textarea id="edit-note-${escapeHtml(note.id)}" name="body" required>${escapeHtml(state.editing_body ?? note.body)}</textarea><div class="note-actions"><button class="button" type="submit">Save changes</button><button class="button secondary" type="button" data-note-edit-cancel>Cancel</button></div></form></article>`;
  }
  const confirmation = state.delete_id === note.id
    ? `<div class="note-delete-confirmation" role="alert"><strong>Delete this note?</strong><p>The document file and workflow evidence remain unchanged.</p><div class="note-actions"><button class="button danger" type="button" data-note-delete-confirm="${escapeHtml(note.id)}">Delete note</button><button class="button secondary" type="button" data-note-delete-cancel>Cancel</button></div></div>`
    : `<p>${escapeHtml(note.body)}</p><div class="note-actions"><button class="text-button" type="button" data-note-edit="${escapeHtml(note.id)}">Edit</button><button class="text-button danger-text" type="button" data-note-delete-request="${escapeHtml(note.id)}">Delete</button></div>`;
  return `<article class="document-note" data-note-id="${escapeHtml(note.id)}"><header><span>${escapeHtml(metadata)}</span></header>${confirmation}</article>`;
}

export function documentNotesMarkup(activity, state) {
  const detail = state.detail;
  const title = detail?.title ?? activity.label;
  const number = detail?.document_number;
  const notes = detail?.notes ?? [];
  const list = state.loading && !detail
    ? '<p class="notes-empty">Loading notes…</p>'
    : notes.length === 0
      ? '<p class="notes-empty">No notes yet.</p>'
      : notes.map((note) => noteMarkup(note, state)).join("");
  return `<section class="notes-workspace"><header class="notes-heading"><div><span class="eyebrow">Document notes</span><h2>${escapeHtml(title)}</h2>${number ? `<p>${escapeHtml(number)}</p>` : ""}</div><span class="badge">Stable document ID · ${escapeHtml(activity.document_id)}</span></header>${state.error ? `<p class="notes-error" role="alert">${escapeHtml(state.error)}</p>` : ""}<div class="notes-card"><form id="document-note-compose-form" class="note-composer"><label for="new-note-body">New note</label><textarea id="new-note-body" name="body" required placeholder="Plain text — line breaks preserved. UTF-8.">${escapeHtml(state.compose_body)}</textarea><div class="note-composer-footer"><label>Author <input name="author" value="${escapeHtml(state.compose_author)}" autocomplete="name" placeholder="Optional — OS user default"></label><button class="button" type="submit">Save note</button></div></form><div class="note-list" aria-live="polite">${list}</div><p class="notes-hint">Newest first. Deleting a note never deletes the document or workflow evidence comments.</p></div></section>`;
}
