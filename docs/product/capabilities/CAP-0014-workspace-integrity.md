# CAP-0014 — Workspace integrity (locks, backups, restore)

| Field | Value |
| --- | --- |
| ID | CAP-0014 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | Partial phases 9d and 9f.5.2 evidence: [core integrity tests](../../../crates/dms-core/tests/integrity.rs), [core backup tests](../../../crates/dms-core/tests/lifecycle.rs), [CLI command tests](../../../crates/dms-cli/tests/cli.rs), [desktop adapter tests](../../../crates/dms-desktop/src/lib.rs), and [desktop workspace/maintenance tests](../../../crates/dms-desktop/ui/app.test.mjs) / [maintenance tests](../../../crates/dms-desktop/ui/maintenance.test.mjs) |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. **Advisory lock.** The application writes `<edit-root>/.dms/lock`
   containing the OS user, hostname, process id, and ISO-8601 UTC timestamp
   while it has the workspace open. The lock is advisory only; the
   application never blocks read-only filesystem access by other tools and
   does not coordinate with non-app writers.
2. **Lock staleness.** On open, if the lock is older than the staleness
   threshold (default 24 hours, configurable per workspace), the application
   warns the operator and offers to take over. A take-over rewrites the
   lock with the current operator's data. Setup also permits an explicit
   override of any existing lock after warning that another application
   instance may still be writing; ordinary open and stale-only takeover never
   overwrite a current lock.
3. **Atomic metadata write.** Every authoritative `.dms` metadata file is
   written to a sibling temporary file and atomically replaced. A crash before
   replacement leaves the previous valid file intact. Temporary artifacts are
   reported and can be removed after the valid generation is confirmed.
4. **Backup and restore.** A **Backup workspace** action creates a compressed
   archive containing `.dms`, controlled source drafts under the edit root,
   and every recorded release PDF under the publish root, plus a manifest of
   relative paths, sizes, and SHA-256 digests. Restore verifies the manifest
   before writing and lets the operator choose replacement edit/publish roots;
   it never writes a path outside those confirmed roots.
5. **Corruption detection.** On open, the application parses every `.dms`
   file. A parse failure or chain mismatch surfaces a clear message, the
   offending path, and a **Restore from backup** prompt. The application
   does not auto-rewrite corrupt state.
6. **Cross-machine portability.** A workspace copy (or a backup restored on
   another machine) opens successfully when the absolute edit and publish roots
   are resolvable. Missing roots can be reassigned only after confirmation.
   Stable document IDs remain identity; relative paths remain locators.
7. **Backup retention.** Backups are operator-managed. The application does
   not implement its own retention or expiry policy.
8. **Schema compatibility.** `.dms` stores a schema version. Opening an older
   supported version creates a metadata backup, migrates atomically, and records
   the migration. An unknown newer version opens read-only and is never
   rewritten. Failed migration restores the pre-migration metadata.
9. The growing backup-archive table follows CAP-0005's table interaction; its
   text filter case-insensitively matches backup filename, creation time, and
   manifest digest before pagination.

## Non-goals

- Encrypting the backup archive
- Cloud upload or off-machine storage of backups
- Multi-writer coordination (a workspace is single-writer per app instance)

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0014-workspace-integrity.html`](../wireframes/html/CAP-0014-workspace-integrity.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0014-workspace-integrity.png`](../wireframes/exports/CAP-0014-workspace-integrity.png)

- Storage: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0001, ADR-0014, ADR-0017: [`../../design-decisions.md`](../../design-decisions.md)
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
