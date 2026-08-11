# CAP-0005 — Tauri desktop shell (Windows and macOS)

| Field | Value |
| --- | --- |
| ID | CAP-0005 |
| Status | not implemented |
| Framework | Tauri 2 |
| Supported OS | Windows, macOS (both required) |
| Tests | Partial phases 1–9g evidence: [`dms-desktop` adapter tests](../../../crates/dms-desktop/src/lib.rs), [shell/setup/sidebar-state/permalink frontend tests](../../../crates/dms-desktop/ui/app.test.mjs), [Configuration route and form tests](../../../crates/dms-desktop/ui/configuration.test.mjs), [Library document and lifecycle-action tests](../../../crates/dms-desktop/ui/library.test.mjs), [release-maintenance frontend tests](../../../crates/dms-desktop/ui/maintenance.test.mjs), [Windows/macOS startup and packaging smoke](../../../.github/workflows/desktop-platform-smoke.yml) |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. A packaged or dev-run desktop app starts on **Windows and on macOS** and
   shows the DMS UI with the same core workflows.
2. Filesystem access for edit root, publish root, and library files works
   through Tauri-mediated APIs on both platforms.
3. Core DMS commands are invokable from the UI via the Tauri backend: configure
   roots, open workspace, library add/remove, directory navigation, selection
   pane (Source file identity + document control data + document/batch actions),
   lifecycle transitions (including begin revision, cancel review, obsolete),
   review notification and decision, notes, confidentiality and document-type
   policy, document-control-data edit, release (version + format-specific PDF export),
   verify checksum, publish history,
   periodic review, audit export, backup/restore, optional Claude Desktop
   change-comment handoff, and document permalink copy/resolve (CAP-0020).
4. The app runs without a database service process.
5. Platform packaging produces installable artifacts for Windows and macOS
   (exact installer formats chosen at implementation).
6. The primary chrome includes a **foldable left menu** (sidebar) with the
   primary destinations (Library, Releases, Audit & Reports, Maintenance,
   Configuration). Expanded and collapsed states are operator-toggled.
7. Collapsed/expanded preference persists in the OS user app-config store (not
   in `.dms`) and restores on next launch for that OS user.
8. When the left menu is **collapsed**, a **hamburger** control in the main
   header unfolds the menu for the current session. Choosing a destination,
   saved view, open pane, or in-surface action preserves that unfolded state;
   only an explicit menu toggle or dismiss action folds it again. The persisted
   expanded/collapsed preference changes only through an explicit sidebar
   control, never as a side effect of navigation.
9. While collapsed, primary destinations remain reachable as icon-only rail
   entries and/or via the hamburger menu; labels are not required on the rail.
10. **Configuration** is one primary destination with persistent in-surface
    navigation, rather than a collection of sidebar destinations or unrelated
    full-page screens. With a workspace open, that navigation presents these
    v1 routes and marks the current route: **Workspace**, **Document defaults**,
    **Workflow**, and **Notifications**. Choosing a route keeps the single
    Configuration activity current and changes its route state; it does not
    create another primary destination or a duplicate Configuration tab.
    Confidentiality catalogue administration and Microsoft Entra identity-source
    setup are explicit secondary surfaces from Document defaults and Workflow
    respectively; dismissing either returns to its invoking Configuration route.
    Before a workspace is open, the app exposes only **Set up workspace** for
    choosing the roots; workspace-bound routes are unavailable with an explicit
    explanation, never presented as empty or broken pages.
11. While collapsed, the rail also exposes distinct, icon-only **Saved views**
    and **Open panes** controls. Activating either opens an adjacent temporary
    flyout for that group without expanding the whole left menu:
    - the Saved views flyout lists each view's full canonical label, opens its
      target under the existing reuse rules, and offers its remove action;
    - the Open panes flyout lists each pane's full canonical label, focuses that
      pane, and offers its close action.
    The flyout dismisses without changing the persisted sidebar preference.
    Rail icons have accessible names and hover labels; a constrained flyout may
    elide a label only when its full canonical label remains available as the
    item's accessible name.
12. **Open activities** are automatic, browser-like **panes/tabs** below the
    primary destinations. A pane label states its **task** and current target:
    - a document-scoped pane is `Task · <DMS-managed title> · <document
      number>` (for example, `Audit · HR Data Privacy Policy · DOC-014`); the
      optional document-number segment is omitted when no number is set
    - a folder-scoped pane is `Task · <edit-root-relative folder path>` (for
      example, `Library · policies/HR`)
    - a task with no folder or document target uses its task name alone
    Document pane labels use DMS-managed title and document number, never the
    source filename, path, version label, or a different document's data.
    The visible label is not an activity identity: changing a title or number
    updates the existing pane label in place. A constrained chrome may elide a
    label visually, but its full canonical label remains available on hover and
    as the pane's accessible name.

    Opening or focusing a document surface uses the stable key **workspace ID +
    task + document ID**. If that key is already open, the app focuses the
    existing pane rather than creating a duplicate. Different tasks may remain
    open for the same document (for example, Audit, Review, and Notes), because
    their task keys differ. The Library keeps one session pane; navigating
    folders updates its path and folder label in place rather than creating a
    pane per visited folder. Other folder-scoped activities use workspace ID +
    task + normalized edit-root-relative folder path to reuse an already-open
    matching pane.

    Exactly one open activity is current: its tab has the selected treatment
    and its label matches the main-header activity label. Activating a tab
    brings that surface forward without losing unrelated open tabs. Closing a
    tab dismisses only that surface; primary destinations stay available.
    Closing the last document-scoped tab does not exit the app. Open activities
    end with the application session and are not restored automatically after a
    relaunch.
13. An operator can explicitly save the current surface as a **saved view**.
    The main header always exposes `☆ Bookmark this view`; after saving it,
    that control shows `★ Bookmarked` and exposes an explicit remove-bookmark
    action. Saved views appear in their own **Saved views** group above Open
    panes, use a star rather than the `×` close affordance, and persist across
    relaunch for that OS user. A saved-view target contains the stable workspace
    ID, primary destination, and compatible route state; a document target uses
    the stable document ID, never a path, file name, or version label. Activating
    a saved view opens its target and creates or focuses the matching open
    activity under the reuse rules above.
    An inaccessible workspace or missing document is shown as unavailable and
    remains removable. Saved views are per-user app preferences in the OS
    app-config store, not `.dms` workflow or process evidence.
14. The left menu foot shows the current workspace identity (display name or
    workspace ID) and root path summary when expanded; the foot is hidden
    when the menu is collapsed to keep the rail narrow.
15. The application shell is contained to the current window viewport. The
    sidebar brand, primary destinations, workspace foot, and main activity
    header do not move when activity content scrolls. Ordinary activities
    scroll inside the main-content region. A multi-pane workspace may give its
    navigation, list, and detail panes independent scroll regions; scrolling
    one region never shifts the shell chrome or sibling navigation. When Saved
    views and Open panes exceed the sidebar height, those lists scroll without
    moving the primary destinations or workspace foot.
16. Every operator-facing data table whose result set can grow as workspace
    content or durable records accumulate provides a case-insensitive,
    surface-appropriate text filter and a **Rows per page** choice of 10, 25,
    50, or 100. Filtering applies before pagination; clearing the filter,
    changing the filter, or changing the page size starts at the first matching
    row. Previous/Next pagination is available only when the filtered result
    exceeds the selected page size. Tables that enumerate a bounded fixed
    product configuration are excluded.
17. Before a workspace is open, the shell shows the ten most recently opened
    libraries from per-user preferences. Each entry opens that edit root and
    has a separate removal control; removal never modifies workspace metadata
    or files.

## Non-goals

- Mobile targets
- Linux as a required supported platform in v1 (may work incidentally via Tauri)
- Multiple top-level application windows per workspace in v1 (one main window;
  activities are in-window panes/tabs)
- Persisting open activity tabs inside `.dms` as process evidence; tabs are
  session-only, while saved views are per-user OS app-config preferences

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0005-desktop-shell.html`](../wireframes/html/CAP-0005-desktop-shell.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0005-desktop-shell.png`](../wireframes/exports/CAP-0005-desktop-shell.png)

- Architecture: [`../../architecture.md`](../../architecture.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0002: [`../../design-decisions.md`](../../design-decisions.md)
- Permalinks: [`CAP-0020-document-permalinks.md`](CAP-0020-document-permalinks.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
