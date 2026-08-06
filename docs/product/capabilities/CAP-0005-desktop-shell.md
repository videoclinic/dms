# CAP-0005 — Tauri desktop shell (Windows and macOS)

| Field | Value |
| --- | --- |
| ID | CAP-0005 |
| Status | not implemented |
| Framework | Tauri 2 |
| Supported OS | Windows, macOS (both required) |
| Tests | none |

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
   policy, document-control-data edit, release (version + Office PDF export),
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
   header opens the menu (temporary expand or overlay). Choosing a destination
   or dismissing the menu returns to the collapsed chrome unless the operator
   pins expanded.
9. While collapsed, primary destinations remain reachable as icon-only rail
   entries and/or via the hamburger menu; labels are not required on the rail.
10. **Open activities** are automatic, browser-like **panes/tabs** below the
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
11. An operator can explicitly save the current surface as a **saved view**.
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
12. The left menu foot shows the current workspace identity (display name or
    workspace ID) and root path summary when expanded; the foot is hidden
    when the menu is collapsed to keep the rail narrow.

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
