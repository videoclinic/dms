# CAP-0006 — Folder-first controlled library explorer

| Field | Value |
| --- | --- |
| ID | CAP-0006 |
| Status | not implemented |
| Tests | Partial phases 3, 9b.1, 9e, 9f.1, 9f.2, 9f.5, 9g, 9k.3, and 9k.4 evidence: [`dms-core` Library and lifecycle tests](../../../crates/dms-core/tests/lifecycle.rs), [`dms-core` recursive Library counter tests](../../../crates/dms-core/tests/library.rs), [`dms-desktop` adapter tests](../../../crates/dms-desktop/src/lib.rs), [shell, direct folder activation, bounded splitter, and permalink-target tests](../../../crates/dms-desktop/ui/app.test.mjs), [hierarchical Library, file-visibility, counter, icon, document-control, placeholder, and workflow frontend tests](../../../crates/dms-desktop/ui/library.test.mjs), [Windows/macOS smoke](https://github.com/videoclinic/dms/actions/runs/31359360786) |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The primary library surface is a **folder-dominant, edit-root-relative
   view** of the accessible directory and file structure below the edit root.
   Library membership annotates files; it never determines whether a file is
   visible. It is not an unmanaged dump of the entire disk and not a standalone
   multi-page file-manager application.
2. Folder navigation uses familiar **Windows File Explorer-like conventions**
   without copying the OS file manager wholesale:
   - The default Library layout has three full-height panes: a persistent,
     resizable **folder tree** on the left, current-folder contents in the
     centre, and the selection/details pane on the right. The folder tree is a
     primary navigation surface, not a compact filter card beside a
     document-dominant table.
   - The tree shows every accessible folder below the edit root, including
     empty folders and folders without registered documents. `<edit-root>/.dms`
     is never shown.
   - The tree is a semantic and visual hierarchy, not a flat list with depth
     indentation. A folder with children has an explicit expand/collapse
     control independent from its navigation action. Navigating to a folder
     expands its ancestor chain and keeps unrelated branch expansion stable for
     the application session.
   - Every tree folder and current-folder folder row shows the same recursive
     summary immediately after its name: `~N` counts visible controlled files
     whose lifecycle is exactly `draft`, `+N` counts visible supported source
     files available to add, and `!N` counts visible unsupported regular files.
     Counts include descendants, omit zero buckets, and exclude `.dms`, Office
     `~$` sidecars, directories, the configured Markdown export-template asset,
     and controlled non-draft files. Each badge has
     an accessible text equivalent; visibility controls never change these
     unfiltered values.
   - A current-folder toolbar exposes **Back**, **Forward**, and **Up** controls
     plus a clickable breadcrumb rooted at the workspace/edit-root display
     name. Up is unavailable at the root. Tree selection, breadcrumb, current
     folder heading, and centre contents stay synchronized.
   - The current-folder toolbar stays available while pane content scrolls.
     The folder tree, centre contents, and right selection/details pane are
     separate vertical scroll regions. Exhaustive document control data,
     lifecycle forms, release history, or workflow evidence in the right pane
     never moves the toolbar, folder tree, centre contents, main activity
     header, or application sidebar.
   - A Windows Explorer-like **Refresh** icon in the path toolbar, immediately
     before the breadcrumb, re-enumerates the edit-root structure after external
     filesystem changes. It never adds a file to the library or changes a
     document's membership.
   - The centre pane lists the current folder's immediate child folders followed
     by its immediate child files. Every regular file is represented, except
     internal `.dms` content and Office lock/temp sidecars defined by CAP-0013.
     A file row's **Name** is always the exact filesystem file name, including
     its extension. Each file row states whether it is **In library**, **Not in
     library** (a supported source draft), **Workspace template** (the configured
     CAP-0007 asset), or not a supported draft. The template remains visible at
     its exact edit-root-relative location but is not selectable for document,
     batch-add, lifecycle, notes, or permalink actions. A
     registered document additionally shows its DMS-managed document title and
     control data in a separate **Title** field; that title never replaces
     the source file name. Primary-click or Enter on a folder row opens that
     folder directly without first selecting it or loading document detail.
     File rows retain single- and modifier-assisted multi-selection. Selecting
     a tree node or breadcrumb segment opens that folder directly.
   - Back/Forward history is session-only. Navigating alone does not create a
     saved view; the operator must use CAP-0005's explicit bookmark control.
3. Selecting one or more **Not in library** supported source files
   shows their names, relative paths, and membership state in the right selection
   pane. For exactly one file, that pane exposes **Add to library**. For two or
   more selected files, it exposes **Add _N_ documents to library** only when
   every selected row is a supported in-root source file that is not already
   registered. Add fails for a path outside the edit root or an already
   registered file. There is no header-level `Add documents` picker: the
   selection makes every action target explicit.
4. Operator can **unregister** a document from the active library only when no
   content or periodic review is open. Unregister preserves its stable ID,
   document control data, notes, workflow/release history, and checksums in read-only
   history; it never deletes the source file or a released PDF. Re-registering
   that record associates a confirmed in-root draft path with the same ID.
5. **In library** document rows keep the exact source file name in **Name** and
   surface enough DMS-managed data to scan the current folder without leaving
   the explorer: lifecycle state, latest released version label, document title,
   document number (when set), document type, current owner, effective
   confidentiality, current release effective date when recorded, next review
   due with overdue highlight, and a **draft newer than last
   release** indicator when known (CAP-0015). Non-library and unsupported file
   rows show their filesystem name and membership/support state instead. File
   rows are for selection only — they do not host per-row action menus (no
   per-row hamburger / overflow menu). Folder rows navigate as defined above.
6. **Selecting exactly one document** keeps the operator on the same page and
   shows an **on-page selection pane** (right column) that combines:
   - the selected document’s DMS-managed **title** and document number
   - an always-visible **Source file** identity with the exact file name and its
     edit-root-relative folder/path, derived from the filesystem
   - CAP-0015 **Document control data** and related status (mutable profile,
     immutable current-release profile and effective date, effective
     confidentiality with its source and inherited/overridden status, effective
     editor/approver, current release, draft-newer marker)
   - **document actions** for that selection
   CAP-0015 owns field rules and revision/obsolescence semantics; CAP-0006 owns
   navigation, selection, and pane placement. Its **Document control data**,
   **Actions**, **Revision cycle**, and **Releases** sections are independently
   foldable as defined by CAP-0015; the selection header and Source file
   identity remain visible. The centre/details divider is pointer- and
   keyboard-resizable for the active Library session between 280 and 640 pixels
   while leaving at least 360 pixels for the directory table; Escape during a
   drag restores its starting width. Width is not persisted in workspace
   metadata, preferences, or saved views. Action labels do **not** repeat the
   document title or number — the selection header already identifies it.
7. **Multi-select** of two or more file rows uses the **same selection pane**.
   It provides a batch summary (count, short identity list, clear) and
   **multi-applicable actions only**. A homogeneous selection of **Not in
   library** supported source files exposes **Add _N_ documents to library**.
   A homogeneous selection of **In library** documents replaces document
   control data with its applicable batch actions. Mixed selections and
   unsupported files expose no action that cannot apply to every selected row. Single-document
   actions are hidden until the selection returns to exactly one document. There
   is no separate hamburger or list-embedded action strip for batch work.
8. Selection-pane actions invoke capabilities owned elsewhere; they do not
   redefine those contracts. With one document selected, actions include at
   least:
   - **Open draft** (CAP-0009)
   - **Open latest released PDF** (CAP-0015) when a current released version
     exists
   - **Edit document control data** (CAP-0015)
   - **Override confidentiality** (CAP-0008) for the selected document; the
     action selects or clears that document's explicit type and is unavailable
     for multi-select
   - **Submit for review**, **Begin revision**, **Cancel review**, **Release**
     when the lifecycle allows (CAP-0002 / CAP-0015)
   - **Mark obsolete** (CAP-0015)
   - **Notes** (CAP-0003)
   - **Workflow chain / evidence** (CAP-0011)
   - **Verify release integrity** (CAP-0004)
   - **Start periodic review** when due rules allow (CAP-0017)
   - **Rename / reassociate source file** when applicable (CAP-0013)
   - **Unregister** (this CAP)
   - **Copy permalink** (CAP-0020) — clipboard receives the stable
     workspace+document URI; never a path- or version-based link
   - **Claude change assistance** when enabled (CAP-0018)
   Multi-select exposes only multi-applicable actions (including batch add for
   a homogeneous selection of unregistered supported source files, plus bulk
   verify where defined and multi-unregister with per-item precondition checks).
   Copy permalink is single-selection only. Per-document actions such as Submit
   for review, Mark obsolete, Start periodic review, and Copy permalink are not
   exposed as batch actions — "Send reminder" is a per-document periodic
   reminder action (CAP-0017) and is also not a batch action. Actions refuse
   closed with a clear reason when preconditions fail.
9. Entering the Library creates or focuses one CAP-0005 **Library activity tab**
   labeled `Library · <edit-root-relative folder path>`. Folder navigation
   updates that tab's current path and label in place; it does not create one
   tab per visited folder. Opening or focusing a document-scoped surface
   (selection, audit, review, notes, or equivalent) creates or focuses the
   matching **task + stable document ID** activity. Its label is `Task ·
   <DMS-managed title> · <document number>` (omitting the last segment when no
   number is set); it never uses the source filename or a path as document
   identity. Repeating navigation to the same task and document focuses the
   existing tab. The same document may have multiple tabs open only for different
   tasks. Closing a document-scoped tab clears that surface without unregistering
   the document. Returning from Document Notes with **Back to Library** focuses
   this singleton activity with the same stable document selected. An unchanged
   in-memory Library view retains its folder, search, sort, history, scroll, and
   selection state without reloading. If the activity was closed or its
   selection changed, the stable document ID resolves its current folder and
   selection detail. A missing registered source retains document detail and
   missing-source state without fabricating a filesystem row.
10. CAP-0005's `Bookmark this view` control saves the current library folder and
    sort order; when exactly one document is selected it also uses that
    document's stable ID as the target. It does not retain a multi-select batch
    selection or an absolute path. Restoring the saved view applies that state
    and creates or focuses the corresponding open-activity tab. **Copy
    permalink** remains a separate single-document action: it never creates or
    modifies a saved view.
11. Documents start versioning only after they are in the library; add is the
    gate into CAP-0002.
12. An obsolete document's draft remains in its filesystem location and is shown
    in the directory list with an `obsolete` lifecycle state. A missing
    registered document has no fabricated directory row; it is reported as a
    maintenance issue until resolved (CAP-0013).
13. A labelled **Show in folder** group above the directory table has three
    independent, all-on session controls: **Draft documents** shows controlled
    rows whose lifecycle is exactly `draft`; **Available to add** shows
    supported `Not in library` files; and **Unsupported files** shows
    unsupported regular files. Folder rows, controlled non-draft rows, and
    unclassified rows remain visible. The state follows the Library activity
    across folders and search, filters before sorting and pagination, returns to
    page zero when changed, and prunes hidden selections and detail. It is a
    file-visibility aid, not CAP-0012 metadata reporting, and never changes
    recursive folder counters.
14. Explorer search starts at the current folder and includes its descendants,
    with an explicit **Entire library** scope. It matches registered-document
    title and document number plus every file's exact source file name and
    relative path case-insensitively. Results retain their relative path and can
    be sorted by title, document number, lifecycle state, latest release, or
    next-review-due date. Search is an explicit result state whose matching rows
    use the same session-wide file-visibility controls before CAP-0005's
    growing-table sorting and pagination; clearing search restores the complete
    current-folder listing without resetting those controls.
15. A CAP-0020 document permalink that resolves successfully lands here: the
    library navigator selects that document (revealing its folder as needed)
    and shows the selection pane. Resolution never keys off file name or
    version label.

## Non-goals

- Full OS file manager replacement (copy/move/rename of arbitrary non-library
  files)
- Pixel-for-pixel Windows File Explorer imitation or arbitrary path entry
- Watching the entire edit root and auto-adding every new supported source file without
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
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Claude handoff: [`CAP-0018-claude-desktop-change-assistance.md`](CAP-0018-claude-desktop-change-assistance.md)
- Permalinks: [`CAP-0020-document-permalinks.md`](CAP-0020-document-permalinks.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
