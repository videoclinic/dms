# CAP-0006 — Controlled library directory navigator

| Field | Value |
| --- | --- |
| ID | CAP-0006 |
| Status | not implemented |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The primary library surface is a **directory navigator** over documents
   currently under DMS control (the library), not an unmanaged dump of the
   entire disk and not a standalone multi-page “file manager app.”
2. Navigation is folder-first: a **relative folder tree** (nested expand/collapse
   of library paths under the edit root — not a flat path list) plus a document
   list for the current folder and active filters. Breadcrumbs or equivalent
   show the current relative path.
3. Operator can **add** a Microsoft Office file that lives under the edit root
   into the library. Add fails if the path is outside the edit root or already
   registered.
4. Operator can **unregister** a document from the active library only when no
   content or periodic review is open. Unregister preserves its stable ID,
   master data, notes, workflow/release history, and checksums in read-only
   history; it never deletes the Office file or a published PDF. Re-registering
   that record associates a confirmed in-root draft path with the same ID.
5. The list surfaces enough **row metadata** to scan the controlled set without
   leaving the navigator: lifecycle state, latest released version label, title,
   document number (when set), document type, owner, effective confidentiality,
   next review due with overdue highlight, and a **draft newer than last
   release** indicator when known (CAP-0015). The list is for navigation and
   selection only — it does not host per-row action menus.
6. **Selecting exactly one document** keeps the operator on the same page and
   shows an **on-page selection pane** (right column) that combines:
   - the selected document’s **title** (same string as the list row title),
     document number, and relative draft path
   - CAP-0015 **master data** and related status (effective editor/approver,
     current release, draft-newer marker)
   - **document actions** for that selection
   CAP-0015 owns field rules and revision/obsolescence semantics; CAP-0006 owns
   navigation, selection, and pane placement. Action labels do **not** repeat
   the document title or number — the selection header already identifies it.
7. **Multi-select** (two or more rows checked) uses the **same selection pane**.
   Master data is replaced by a batch summary (count, short identity list, clear)
   and **multi-applicable actions only**. Single-document actions are hidden
   until the selection returns to exactly one document. There is no separate
   hamburger or list-embedded action strip for batch work.
8. Selection-pane actions invoke capabilities owned elsewhere; they do not
   redefine those contracts. With one document selected, actions include at
   least:
   - **Open draft** (CAP-0009)
   - **Edit master data** (CAP-0015)
   - **Submit for review**, **Begin revision**, **Cancel review**, **Release**
     when the lifecycle allows (CAP-0002 / CAP-0015)
   - **Mark obsolete** (CAP-0015)
   - **Notes** (CAP-0003)
   - **Workflow chain / evidence** (CAP-0011)
   - **Verify release integrity** (CAP-0004)
   - **Start periodic review** / **Send reminder** when due rules allow
     (CAP-0017)
   - **Rename / reassociate locator** when applicable (CAP-0013)
   - **Unregister** (this CAP)
   - **Claude change assistance** when enabled (CAP-0018)
   Multi-select exposes only multi-applicable actions (for example bulk verify
   where defined, multi-unregister with per-item precondition checks). Actions
   refuse closed with a clear reason when preconditions fail.
9. Documents start versioning only after they are in the library; add is the
   gate into CAP-0002.
10. Obsolete documents are hidden by default; an explicit control shows them.
    Missing documents remain visible with a `missing` marker until resolved
    (CAP-0013).
11. Explorer filters support lifecycle state, confidentiality type, document
    type, owner, and overdue-only.
12. Explorer search matches title, document number, draft file name, and
    relative path case-insensitively. Results can be sorted by title, document
    number, lifecycle state, latest release, or next-review-due date.

## Non-goals

- Full OS file manager replacement (copy/move/rename of arbitrary non-library
  files)
- Watching the entire edit root and auto-adding every new Office file without
  operator action
- Per-row hamburger / overflow menus in the list
- Replacing dedicated deep screens for long workflows (approval decision form,
  full audit export) — the selection pane opens those flows; they need not all
  fit inline

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0006-library-explorer.html`](../wireframes/html/CAP-0006-library-explorer.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0006-library-explorer.png`](../wireframes/exports/CAP-0006-library-explorer.png)

- CAP-0001 dual roots: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- CAP-0002 lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Notes: [`CAP-0003-document-notes.md`](CAP-0003-document-notes.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Open draft: [`CAP-0009-release-editor.md`](CAP-0009-release-editor.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Maintenance: [`CAP-0013-library-maintenance.md`](CAP-0013-library-maintenance.md)
- Master data: [`CAP-0015-document-master-data.md`](CAP-0015-document-master-data.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Claude handoff: [`CAP-0018-claude-desktop-change-assistance.md`](CAP-0018-claude-desktop-change-assistance.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
