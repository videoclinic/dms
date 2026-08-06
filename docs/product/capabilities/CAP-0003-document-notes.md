# CAP-0003 — Document notes

| Field | Value |
| --- | --- |
| ID | CAP-0003 |
| Status | not implemented |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Operator can add, edit, and list free-text notes attached to a registered
   document by its stable document ID (CAP-0001 / ADR-0015).
2. Each note stores body text, author display name (operator-provided or OS
   user default), and timestamp.
3. Notes persist in `.dms` and survive application restart and draft path
   renames that preserve the document ID.
4. Notes remain available across lifecycle states (including released and
   obsolete) unless the operator deletes them.
5. Deleting a note is an explicit action and does not delete the document file
   or workflow evidence comments (CAP-0011).

## Non-goals

- Real-time multi-user collaborative editing of notes
- Rich-text mandatory format (plain text is sufficient for v1)

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0003-document-notes.html`](../wireframes/html/CAP-0003-document-notes.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0003-document-notes.png`](../wireframes/exports/CAP-0003-document-notes.png)

- Privacy: [`../../privacy.md`](../../privacy.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
