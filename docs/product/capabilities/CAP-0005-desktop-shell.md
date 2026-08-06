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
   pane (master data + document/batch actions), lifecycle transitions
   (including begin revision, cancel review, obsolete), review notification and
   decision, notes, confidentiality and document-type policy, master-data edit,
   release (version + Office PDF export), verify checksum, publish history,
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
10. **Open activities** appear in the left menu as browser-like **panes/tabs**
    below the primary destinations. Each tab is a quicklink that restores that
    activity’s main surface (for example Library at a folder, a document
    selection, an open review decision, notes, releases list, or a config
    screen). Tabs show a short label derived from the activity, not a full path
    dump.
11. Opening a document-scoped or multi-step activity adds or focuses its tab.
    Activating a tab brings that activity forward without losing unrelated open
    tabs. Closing a tab dismisses only that activity; primary destinations stay
    available. Closing the last document-scoped tab does not exit the app.
12. The left menu foot still shows the current workspace identity (display name
    or workspace ID) and root path summary when expanded; collapsed chrome may
    hide the foot.

## Non-goals

- Mobile targets
- Linux as a required supported platform in v1 (may work incidentally via Tauri)
- Multiple top-level application windows per workspace in v1 (one main window;
  activities are in-window panes/tabs)
- Persisting open activity tabs inside `.dms` as process evidence (session UI
  state may live in OS user config; workflow truth stays in `.dms`)

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0005-desktop-shell.html`](../wireframes/html/CAP-0005-desktop-shell.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0005-desktop-shell.png`](../wireframes/exports/CAP-0005-desktop-shell.png)

- Architecture: [`../../architecture.md`](../../architecture.md)
- ADR-0002: [`../../design-decisions.md`](../../design-decisions.md)
- Permalinks: [`CAP-0020-document-permalinks.md`](CAP-0020-document-permalinks.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
