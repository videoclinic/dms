# CAP-0016 — Publish-tree and release-set maintenance

| Field | Value |
| --- | --- |
| ID | CAP-0016 |
| Status | not implemented |
| Storage | Publish root + `<edit-root>/.dms/` release records |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The application can list **all known released versions** for a library
   document from `.dms` (version label, relative publish path, release time,
   checksum, confidentiality snapshot, withdrawn flag, approval-chain head).
2. Each non-missing release record offers **Open PDF**, which opens exactly its
   recorded versioned PDF in the host's registered PDF handler. The action is
   unavailable for a missing file and never substitutes another version.
3. The operator can open the publish folder for a document in the host file
   manager and can reveal a specific versioned PDF when the file exists.
4. The publish-tree view has a **Title** filter that case-insensitively matches
   the DMS-managed document title (for example, `Doc`), and clearing it restores
   the unfiltered release list.
5. The operator can select the number of matching release records shown per
   page before pagination begins. The selected page size applies to the current
   title-filter result; Previous/Next paging actions are enabled only when the
   result exceeds it.
6. **Verify all releases** for a document (or the whole workspace) runs
   CAP-0004 verification for every non-missing recorded PDF and reports
   per-version `match` / `mismatch` / `missing file`.
7. Removing a document from the library (CAP-0006 default non-destructive
   unlink) does **not** delete publish-root PDFs. The release records remain
   readable under an **orphaned releases** view keyed by former relative path
   and stable document ID until the operator confirms archival cleanup.
8. An explicit **Archive orphaned release files** action (destructive,
   double-confirm) moves selected orphan PDFs to an operator-chosen archive
   folder outside or under the publish root; it never silently deletes. The
   action records a workflow/workspace event with paths and checksums.
9. Withdrawn releases stay listed under version history with a withdrawn
   marker; they are excluded from “current released version” (CAP-0015).
10. If a versioned PDF path is occupied by a file the app did not create (manual
   drop), release fails closed (ADR-0007) and the conflict path is shown.
11. Changing the publish root (workspace config) requires validation that the
   new root is writable; existing release records keep relative paths and
   resolve against the new absolute root after confirm.
12. Release records are immutable. A wrong release cannot be edited in place or
   have its PDF replaced; correction requires withdrawal with a reason, Begin
   revision, renewed approval, and a new monotonically increasing release.

## Non-goals

- Automatic deletion of old versions on a retention schedule
- Content diff between two PDF versions
- Distributing released PDFs to a second remote store

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0016-publish-tree-maintenance.html`](../wireframes/html/CAP-0016-publish-tree-maintenance.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0016-publish-tree-maintenance.png`](../wireframes/exports/CAP-0016-publish-tree-maintenance.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- ADR-0006, ADR-0007: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
