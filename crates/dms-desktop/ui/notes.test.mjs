import test from "node:test";
import assert from "node:assert/strict";

import {
  applyDocumentNotes,
  createNoteDocumentState,
  documentNotesMarkup,
  noteDocumentState,
  updateNoteDocumentState,
} from "./notes.mjs";

const activity = {
  label: "Notes · HR policy · HR-001",
  document_id: "doc-1",
};

const detail = {
  document_id: "doc-1",
  title: "HR policy",
  document_number: "HR-001",
  notes: [
    {
      id: "note-2",
      body: "Latest line one\nLatest line two",
      author: "Raphael",
      created_at: "2026-08-10T10:00:00Z",
      updated_at: "2026-08-10T10:00:00Z",
    },
    {
      id: "note-1",
      body: "Earlier <review>",
      author: "OS user",
      created_at: "2026-08-09T10:00:00Z",
      updated_at: "2026-08-09T10:00:00Z",
    },
  ],
};

test("document notes keep independent per-document interaction state", () => {
  let states = {};
  assert.deepEqual(noteDocumentState(states, "doc-1"), createNoteDocumentState());
  states = updateNoteDocumentState(states, "doc-1", {
    compose_body: "Unsaved body",
    compose_author: "Raphael",
  });
  states = applyDocumentNotes(states, detail);
  states = updateNoteDocumentState(states, "doc-1", { editing_id: "note-2" });

  assert.equal(states["doc-1"].detail.title, "HR policy");
  assert.equal(states["doc-1"].compose_body, "");
  assert.equal(states["doc-1"].compose_author, "");
  assert.equal(states["doc-1"].editing_id, "note-2");
  assert.equal(noteDocumentState(states, "doc-2").detail, null);
});

test("composer appears above the newest-first note list and preserves safe plain text", () => {
  const state = { ...createNoteDocumentState(), detail };
  const markup = documentNotesMarkup(activity, state);

  assert.ok(markup.indexOf("document-note-compose-form") < markup.indexOf('data-note-id="note-2"'));
  assert.ok(markup.indexOf('data-note-id="note-2"') < markup.indexOf('data-note-id="note-1"'));
  assert.match(markup, /Latest line one\nLatest line two/);
  assert.match(markup, /Earlier &lt;review&gt;/);
  assert.match(markup, /Optional — OS user default/);
  assert.match(markup, /data-note-edit="note-2"/);
  assert.match(markup, /data-note-delete-request="note-2"/);
});

test("editing and deletion use explicit save, cancel, and confirmation controls", () => {
  const editing = documentNotesMarkup(activity, {
    ...createNoteDocumentState(),
    detail,
    editing_id: "note-2",
  });
  assert.match(editing, /id="document-note-edit-form"/);
  assert.match(editing, /Save changes/);
  assert.match(editing, /data-note-edit-cancel/);

  const deleting = documentNotesMarkup(activity, {
    ...createNoteDocumentState(),
    detail,
    delete_id: "note-1",
  });
  assert.match(deleting, /Delete this note\?/);
  assert.match(deleting, /workflow evidence remain unchanged/);
  assert.match(deleting, /data-note-delete-confirm="note-1"/);
  assert.match(deleting, /data-note-delete-cancel/);
});

test("a failed mutation can show its document-scoped error without discarding the draft", () => {
  const markup = documentNotesMarkup(activity, {
    ...createNoteDocumentState(),
    detail,
    compose_body: "Retry this <note>",
    compose_author: "Raphael",
    error: "Could not save",
  });

  assert.match(markup, /role="alert">Could not save/);
  assert.match(markup, /Retry this &lt;note&gt;/);
  assert.match(markup, /value="Raphael"/);
});
