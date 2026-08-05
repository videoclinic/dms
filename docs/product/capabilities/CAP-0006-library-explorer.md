# CAP-0006 — Controlled library and file explorer

| Field | Value |
| --- | --- |
| ID | CAP-0006 |
| Status | not implemented |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The UI presents a **file explorer** of documents currently under DMS control
   (the library), not an unmanaged dump of the entire disk.
2. Operator can **add** a Microsoft Office file that lives under the edit root
   into the library. Add fails if the path is outside the edit root or already
   registered.
3. Operator can **unregister** a document from the active library only when no
   content or periodic review is open. Unregister preserves its stable ID,
   master data, notes, workflow/release history, and checksums in read-only
   history; it never deletes the Office file or a published PDF. Re-registering
   that record associates a confirmed in-root draft path with the same ID.
4. Explorer shows enough structure to navigate the controlled set (relative
   folder tree and/or list with relative paths).
5. Explorer surfaces lifecycle state, latest released version label, title,
   document number (when set), effective confidentiality, next review due with
   overdue highlight, and a **draft newer than last release** indicator when
   known (CAP-0015).
6. Documents start versioning only after they are in the library; add is the
   gate into CAP-0002.
7. Obsolete documents are hidden by default; an explicit control shows them.
   Missing documents remain visible with a `missing` marker until resolved
   (CAP-0013).
8. Explorer filters support lifecycle state, confidentiality type, document
   type, owner, and overdue-only.
9. Explorer search matches title, document number, draft file name, and relative
   path case-insensitively. Results can be sorted by title, document number,
   lifecycle state, latest release, or next-review-due date.

## Non-goals

- Full OS file manager replacement (copy/move/rename of arbitrary files)
- Watching the entire edit root and auto-adding every new Office file without
  operator action

## Links

- CAP-0001 dual roots: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- CAP-0002 lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Master data: [`CAP-0015-document-master-data.md`](CAP-0015-document-master-data.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
