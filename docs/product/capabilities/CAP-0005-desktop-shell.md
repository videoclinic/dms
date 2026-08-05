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
   roots, open workspace, library add/remove, explorer navigation, lifecycle
   transitions (including begin revision, cancel review, obsolete), review
   notification and decision, notes, confidentiality and document-type policy,
   master-data edit, release (version + Office PDF export), verify checksum,
   publish history, periodic review, audit export, backup/restore, and optional
   Claude Desktop change-comment handoff.
4. The app runs without a database service process.
5. Platform packaging produces installable artifacts for Windows and macOS
   (exact installer formats chosen at implementation).

## Non-goals

- Mobile targets
- Linux as a required supported platform in v1 (may work incidentally via Tauri)

## Links

- Architecture: [`../../architecture.md`](../../architecture.md)
- ADR-0002: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
