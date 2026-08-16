# CAP-0013 — Library maintenance beyond add/remove

| Field | Value |
| --- | --- |
| ID | CAP-0013 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. **Rename inside the edit root.** Rename/move performed through the app
   updates the relative locator while preserving stable document ID and
   history. It does not change CAP-0015 document control data. For an external
   move, rescan may suggest a unique match using available filesystem identity
   and the last stored draft digest, but requires operator confirmation. It
   never auto-links an ambiguous or edited candidate.
2. **Lost source ((re-)moved).** If a registered document's draft can no longer
   be resolved at its stored edit-root-relative path, the Library surfaces it as
   **Lost source** in the folder of that stored path (CAP-0006). Lifecycle and
   other source-dependent transitions refuse until the operator reassociates the
   source, restores the file to the stored path, or unregisters the document.
3. **Reassociate source.** Selection-pane reassociate is Lost-source-only and
   lives in the pinned **Actions** block. The operator uses a path field plus
   native **Browse…** that can select only supported drafts (`.md`, `.docx`,
   `.xlsx`, `.pptx`). Location and registration are validated only when the
   operator presses **Reassociate source**. Success:
   - updates only the surviving document's relative locator and restores
     registered source presence;
   - appends a canonical `source_reassociated` workflow event recording
     `old-folder/old-name => new-folder/new-name` (edit-root-relative, `/`
     separators);
   - never changes CAP-0015 document control data by itself.
4. **Reassociate onto a path already in the library.** Desktop **Actions**
   refuse an already-registered target and do not absorb or merge audit
   history from this control. `Workspace::reassociate_document` and the CLI
   keep absorb: when the chosen file is already a **registered** library
   document (the target):
   - the operator is told that the target document must leave the library;
   - the surviving identity is the document being reassociated;
   - the target's canonical workflow events are incorporated into the surviving
     document's audit chain only when every target event timestamp is strictly
     later than every surviving event timestamp (no overlap; target history is
     not older than the previous document's audit log);
   - on success the target is unregistered (leaves the library) and keeps its
     own retained metadata archive; the surviving document owns the path and
     the merged chain plus the `source_reassociated` event (with absorbed
     target document ID);
   - if timestamps overlap or the target has any event older than or equal to
     the surviving document's newest event when the surviving chain is
     non-empty, or any target event older than the surviving chain's oldest
     event, reassociation fails closed with an explicit reason and no mutation.
5. **Rescan.** A maintenance **Rescan library** action walks the edit root and
   reports Lost source registered records with any safe reassociation
   suggestions. The regular CAP-0006 directory listing already shows every
   unregistered supported source draft and every Lost source row and lets the
   operator act from selection. Rescan may offer batch add or reassociation
   after explicit confirmation; it never auto-adds or guesses an ambiguous match.
6. **No application user roster.** The application does not add, edit, disable,
   or delete workflow users. Microsoft Entra group owners maintain eligible
   people; the app only routes a folder or document role to a read-only eligible
   person from that group (CAP-0019 / CAP-0021).
7. **Confidentiality catalogue maintenance.** The operator can add, rename the
   display label of, and disable confidentiality types; their stable type IDs
   never change. A type referenced by an inheriting folder policy or by a
   historical release cannot be deleted; it can be disabled so future documents
   cannot select it (CAP-0008).
8. **Withdraw a release.** The operator can mark an active release as
   `withdrawn`. The PDF remains on disk, the workflow event records the
   withdrawal, and the explorer no longer surfaces the release as the
   current version (CAP-0002 outcome 14).
9. **Reject a draft in review.** An approver records `rejected`; the UI asks
   why approval was not granted but the decision comment is optional. The
   document returns to `draft`, and the rejection, candidate version, changelog,
   and any comment are recorded in the canonical event chain.
10. **Supported draft extensions.** Library add accepts `.md` plus the declared
    v1 Office extensions (at least `.docx`; `.xlsx` / `.pptx` as implemented).
    Other extensions fail closed with a clear message (CAP-0007).
11. **Office lock/temp sidecars.** Files matching Office lock/temp patterns
    (e.g. `~$…`) are never offered as library candidates and are ignored by
    rescan.
12. **Document-type catalogue maintenance.** Same add/rename/disable rules as
    confidentiality types (CAP-0015).
13. The **Drafts requiring attention** rescan-result table follows CAP-0005's
    growing-table interaction; its text filter case-insensitively matches DMS
    title, old path, finding status, and reassociation suggestion before
    pagination.

## Non-goals

- Bulk operations on thousands of documents in a single action
- Automated retention/cleanup of old PDFs
- Cross-workspace deduplication

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0013-library-maintenance.html`](../wireframes/html/CAP-0013-library-maintenance.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0013-library-maintenance.png`](../wireframes/exports/CAP-0013-library-maintenance.png)

- Library: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Classification: [`CAP-0008-confidentiality-classification.md`](CAP-0008-confidentiality-classification.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Publish tree: [`CAP-0016-publish-tree-maintenance.md`](CAP-0016-publish-tree-maintenance.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- ADR-0001, ADR-0013, ADR-0015: [`../../design-decisions.md`](../../design-decisions.md)
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
