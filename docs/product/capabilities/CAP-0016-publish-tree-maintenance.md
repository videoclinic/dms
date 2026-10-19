# CAP-0016 — Publish-tree and release-set maintenance

| Field | Value |
| --- | --- |
| ID | CAP-0016 |
| Status | not implemented |
| Storage | Publish root + `<edit-root>/.dms/` release records |
| Tests | Bounded implemented evidence: [index projection and safe-write tests](../../../crates/dms-core/src/publish_index_tests.rs), [index search/sort/filter tests](../../../crates/dms-core/tests/publish_index.test.mjs), [withdrawal/history/current-release core tests](../../../crates/dms-core/tests/lifecycle.rs), [`dms-desktop` refresh/open adapter tests](../../../crates/dms-desktop/src/lib.rs), [release-maintenance frontend tests](../../../crates/dms-desktop/ui/maintenance.test.mjs), [manual refresh and close/switch ordering/failure tests](../../../crates/dms-desktop/ui/app.test.mjs) |

**Primary surface:** human

The capability remains `not implemented` as a whole: the generated index and
bounded history/integrity actions have executable evidence, not every maintenance
outcome below. In particular, orphan archival and publish-root reconfiguration
are not established by the index tests.

## Generated released-document index (implemented)

1. `<publish-root>/index.html` is a DMS-owned derived artifact, not release
   authority. It lists one current non-withdrawn released PDF per registered
   document, selected by the core current-release rule (highest non-withdrawn
   version), not by file discovery or timestamp. A missing current PDF excludes
   the document rather than resurrecting an older version. Drafts, unregistered
   documents' orphaned releases, manual PDF drops, and withdrawn records are absent.
2. Nested folders come only from the selected releases' recorded relative PDF
   parent paths. Selecting a folder includes its subtree; All folders restores
   the whole catalog. Each row opens its exact PDF through a URI-encoded relative
   link beneath the publish root. Invalid or escaping paths are refused.
3. Rows show immutable captured title, document number when recorded, version,
   release timestamp, confidentiality type ID, and relative PDF path. Legacy
   omissions are labelled unrecorded, never filled from mutable draft metadata.
   The page uses inline CSS and JavaScript only, with no network requests,
   external assets, or runtime dependencies. Labels are escaped display text.
4. Default order is last release date, newest first, with deterministic
   title/path/identity tie-breakers. The operator can choose oldest first or
   document name ascending/descending; document name here means captured title,
   not source filename. Case-insensitive search matches displayed release
   metadata only, never PDF/source content. Each present confidentiality type ID
   has an on/off checkbox plus All. Search, type selection, folder selection, and
   sorting combine before the result count and empty state; turning all types
   off yields no rows. Without JavaScript, the current PDF links remain readable.
5. Releases offers **Update released document index**, reporting written or
   already current, and **Open released document index** when an owned index
   exists. Open revalidates ownership and uses the host handler; a missing index
   requires Update first. Errors stay separate from history/integrity results
   and preserve its filters and paging.
6. Clean window close and different-library activation refresh the still-active
   source workspace before session teardown or destination lock acquisition,
   including each delayed authorization poll that can activate the destination.
   Only differing deterministic output replaces the owned file; unchanged bytes
   cause no replacement. Refresh failure aborts close/switch, reports the error,
   and retains the source window, session, activities, and owner-matched lock.
   Same-library focus performs no refresh or handoff.
7. Refresh writes only `index.html` through a flushed temporary sibling and
   atomic replacement. A foreign pre-existing index (without the exact DMS
   ownership prologue), symlink, or non-regular target is refused. Failed writes
   preserve the previous owned index. Refresh never changes `.dms`, release
   records, or PDFs, and performs no checksum verification or source-content scan.
8. The catalog supplements, never replaces, DMS **Release history and integrity**.
   Historical, withdrawn, orphaned, missing-file, and integrity-verification
   records remain maintenance concerns, not catalog rows or integrity badges.
   The existing per-version Open PDF, title filter, pagination, and verification
   actions stay available there; catalog presence is not proof of PDF integrity.

## Maintenance outcomes (full contract — not all implemented)

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
   have its PDF replaced; correction requires withdrawal with a reason, editing
   the draft (digest-derived Draft state), and a new monotonically increasing
   release. Approval follows CAP-0002.

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
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
