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
2. **Move outside the edit root.** If the draft can no longer be resolved
   under the edit root, the application flags the document as `missing` and
   refuses to perform lifecycle transitions on it until the operator
   reassigns the file, restores it from backup, or removes the document from
   the library.
3. **Rescan.** A maintenance **Rescan library** action walks the edit root and
   reports missing registered records with any safe reassociation suggestions.
   The regular CAP-0006 directory listing already shows every unregistered
   supported Office draft and lets the operator add the selected file. Rescan
   may offer batch add or reassociation after explicit confirmation; it never
   auto-adds or guesses an ambiguous match.
4. **Approver roster maintenance.** The operator can add, edit, and disable
   approver profiles (display name, email, optional role label). A profile
   referenced by an open review or by historical workflow events cannot be
   deleted; it can be disabled and superseded.
5. **Confidentiality catalogue maintenance.** The operator can add, rename the
   display label of, and disable confidentiality types; their stable type IDs
   never change. A type referenced by an inheriting folder policy or by a
   historical release cannot be deleted; it can be disabled so future documents
   cannot select it (CAP-0008).
6. **Withdraw a release.** The operator can mark an active release as
   `withdrawn`. The PDF remains on disk, the workflow event records the
   withdrawal, and the explorer no longer surfaces the release as the
   current version (CAP-0002 outcome 14).
7. **Reject a draft in review.** An approver records `rejected` with a
   required decision comment; the document returns to `draft` and the
   rejection is recorded in the canonical event chain.
8. **Supported draft extensions.** Library add accepts only the declared v1
   Office extensions (at least `.docx`; `.xlsx` / `.pptx` as implemented).
   Other extensions fail closed with a clear message (CAP-0007).
9. **Office lock/temp sidecars.** Files matching Office lock/temp patterns
   (e.g. `~$…`) are never offered as library candidates and are ignored by
   rescan.
10. **Document-type catalogue maintenance.** Same add/rename/disable rules as
    confidentiality types (CAP-0015).

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
- Document control data: [`CAP-0015-document-master-data.md`](CAP-0015-document-master-data.md)
- Publish tree: [`CAP-0016-publish-tree-maintenance.md`](CAP-0016-publish-tree-maintenance.md)
- ADR-0001, ADR-0013, ADR-0015: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
