# CAP-0003 — Document notes

| Field | Value |
| --- | --- |
| ID | CAP-0003 |
| Status | implemented |
| Tests | [`dms-core` persistence tests](../../../crates/dms-core/tests/workspace.rs), [`dms-desktop` adapter tests](../../../crates/dms-desktop/src/lib.rs), [frontend interaction tests](../../../crates/dms-desktop/ui/notes.test.mjs), and [Windows/macOS package smoke](https://github.com/videoclinic/dms/actions/runs/31385996931) |

## Outcomes

The document-notes workflow provides the following behaviour:

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
6. The notes list is ordered **newest first** by note timestamp. The **New note**
   compose field (body input, author, save) sits **above** the latest note in
   that list so the operator writes without scrolling past existing entries.
   After a successful save, the new note becomes the top list entry under the
   compose field; the compose field clears and stays above the list.
7. Every Document Notes activity exposes **Back to Library** before the compose
   field. It targets the same workspace and stable document ID. When the
   singleton Library activity still has that document selected, the action
   focuses it without reloading so its folder, search, sort, history, scroll,
   and selection state remain intact. Otherwise the app resolves the stable ID
   and reveals the document's current folder and selection detail. A missing
   source file produces no fabricated row but retains the registered document
   detail and missing-source state. The Notes activity remains open with its
   compose, edit, and delete-confirmation state unchanged. A resolution failure
   leaves Notes current, preserves that state, and shows a document-scoped
   error. This control returns to the selected document; it is not global
   browser history.

## Non-goals

- Real-time multi-user collaborative editing of notes
- Rich-text mandatory format (plain text is sufficient for v1)

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0003-document-notes.html`](../wireframes/html/CAP-0003-document-notes.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0003-document-notes.png`](../wireframes/exports/CAP-0003-document-notes.png)

- Privacy: [`../../privacy.md`](../../privacy.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
