# CAP-0006 — Folder-first controlled library explorer

| Field | Value |
| --- | --- |
| ID | CAP-0006 |
| Status | not implemented |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The primary library surface is a **folder-dominant workspace** over the
   accessible directory structure below the edit root and the documents under
   DMS control. It is not an unmanaged dump of the entire disk and not a
   standalone multi-page file-manager application.
2. Folder navigation uses familiar **Windows File Explorer-like conventions**
   without copying the OS file manager wholesale:
   - The default Library layout has three full-height panes: a persistent,
     resizable **folder tree** on the left, current-folder contents in the
     centre, and the selection/details pane on the right. The folder tree is a
     primary navigation surface, not a compact filter card beside a
     document-dominant table.
   - The tree shows accessible folders below the edit root, including empty
     folders and folders without registered documents. `<edit-root>/.dms` is
     never shown. Unmanaged files remain hidden outside Add/Rescan flows.
   - A current-folder toolbar exposes **Back**, **Forward**, and **Up** controls
     plus a clickable breadcrumb rooted at the workspace/edit-root display
     name. Up is unavailable at the root. Tree selection, breadcrumb, current
     folder heading, and centre contents stay synchronized.
   - The centre pane lists immediate child folders before controlled documents
     directly in the current folder. Single-click selects a folder row;
     double-click or Enter opens it. Selecting a tree node or breadcrumb segment
     opens that folder directly.
   - Back/Forward history is session-only. Navigating alone does not create a
     saved view; the operator must use CAP-0005's explicit bookmark control.
3. Operator can **add** a Microsoft Office file that lives under the edit root
   into the library. Add fails if the path is outside the edit root or already
   registered.
4. Operator can **unregister** a document from the active library only when no
   content or periodic review is open. Unregister preserves its stable ID,
   master data, notes, workflow/release history, and checksums in read-only
   history; it never deletes the Office file or a published PDF. Re-registering
   that record associates a confirmed in-root draft path with the same ID.
5. Controlled-document rows surface enough **metadata** to scan the current
   folder without leaving the explorer: lifecycle state, latest released
   version label, title, document number (when set), document type, owner,
   effective confidentiality, next review due with overdue highlight, and a
   **draft newer than last release** indicator when known (CAP-0015). Document
   rows are for selection only — they do not host per-row action menus (no
   per-row hamburger / overflow menu). Folder rows navigate as defined above.
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
   - **Start periodic review** when due rules allow (CAP-0017)
   - **Rename / reassociate locator** when applicable (CAP-0013)
   - **Unregister** (this CAP)
   - **Copy permalink** (CAP-0020) — clipboard receives the stable
     workspace+document URI; never a path- or version-based link
   - **Claude change assistance** when enabled (CAP-0018)
   Multi-select exposes only multi-applicable actions (for example bulk verify
   where defined, multi-unregister with per-item precondition checks). Copy
   permalink is single-selection only. Per-document actions such as Submit
   for review, Mark obsolete, Start periodic review, and Copy permalink are
   not exposed as batch actions — "Send reminder" is a per-document periodic
   reminder action (CAP-0017) and is also not a batch action. Actions refuse
   closed with a clear reason when preconditions fail.
9. Entering the Library creates or focuses one CAP-0005 **Library activity tab**.
   Folder navigation updates that tab's current path and label in place; it does
   not create one tab per visited folder. Opening or focusing a document-scoped
   surface (selection, review, notes, or equivalent) creates or focuses the
   matching activity labeled from the document title (falling back to document
   number or a short ID prefix). The same document may have multiple tabs open
   (for example Library - selection, Review - decision, Notes) — each tab names
   the focused surface. Closing a document-scoped tab clears that surface
   without unregistering the document.
10. CAP-0005's `Bookmark this view` control saves the current library folder,
    filters, and sort order; when exactly one document is selected it also uses
    that document's stable ID as the target. It does not retain a multi-select
    batch selection or an absolute path. Restoring the saved view applies that
    state and creates or focuses the corresponding open-activity tab. **Copy
    permalink** remains a separate single-document action: it never creates or
    modifies a saved view.
11. Documents start versioning only after they are in the library; add is the
    gate into CAP-0002.
12. Obsolete documents are hidden by default; an explicit control shows them.
    Missing documents remain visible with a `missing` marker until resolved
    (CAP-0013).
13. Explorer filters support lifecycle state, confidentiality type, document
    type, owner, and overdue-only. They filter controlled-document rows within
    the active folder/search scope and never hide the folder tree.
14. Explorer search starts at the current folder and includes its descendants,
    with an explicit **Entire library** scope. It matches title, document number,
    draft file name, and relative path case-insensitively. Results retain their
    relative path and can be sorted by title, document number, lifecycle state,
    latest release, or next-review-due date.
15. A CAP-0020 document permalink that resolves successfully lands here: the
    library navigator selects that document (revealing its folder as needed)
    and shows the selection pane. Resolution never keys off file name or
    version label.

## Non-goals

- Full OS file manager replacement (copy/move/rename of arbitrary non-library
  files)
- Pixel-for-pixel Windows File Explorer imitation or arbitrary path entry
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
- Desktop shell / activity tabs: [`CAP-0005-desktop-shell.md`](CAP-0005-desktop-shell.md)
- Open draft: [`CAP-0009-release-editor.md`](CAP-0009-release-editor.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Maintenance: [`CAP-0013-library-maintenance.md`](CAP-0013-library-maintenance.md)
- Master data: [`CAP-0015-document-master-data.md`](CAP-0015-document-master-data.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Claude handoff: [`CAP-0018-claude-desktop-change-assistance.md`](CAP-0018-claude-desktop-change-assistance.md)
- Permalinks: [`CAP-0020-document-permalinks.md`](CAP-0020-document-permalinks.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
