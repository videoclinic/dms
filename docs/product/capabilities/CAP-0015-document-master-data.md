# CAP-0015 — Document master data and revision cycle

| Field | Value |
| --- | --- |
| ID | CAP-0015 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Each library document stores master data separate from the Office file:
   - **title** (required operator-editable display name; defaults to draft stem)
   - **document number** (optional control identifier, unique in the workspace)
   - **document type** (from a workspace catalogue, e.g. policy, procedure,
     form, record, other)
   - **owner** (display name of the maintaining operator)
   - **effective date** of the current released version (set on release;
     empty while never released)
   - **next review due date** (computed and maintained by CAP-0017)
2. After a successful release, the document remains `released` for the current
   version. Starting the next change cycle is an explicit **Begin revision**
   action that returns the document to `draft` while retaining the released
   PDF and history. The Office draft stays the editable working file.
3. While a newer draft exists after a release (or after Begin revision), the
   explorer surfaces **released version label** and **draft is newer than last
   release** when current draft bytes differ from the approved Office-draft
   digest stored on the last successful release record.
4. The **current released version** is the latest non-withdrawn release
   record. A later release supersedes the previous current version for
   explorer display; prior PDFs remain on disk and remain verifiable.
5. An operator can mark a library document **obsolete**: lifecycle becomes
   `obsolete`, no further review or release is allowed, released PDFs remain
   on disk, and the explorer filters obsolete documents behind an explicit
   show/hide control. Obsolescence records a canonical workflow event with a
   required reason comment.
6. An author (any operator with the workspace open) can **cancel** an open
   `in_review` request with a required comment. The document returns to
   `draft`; the cancellation is recorded in the event chain. Cancel does not
   delete prior events.
7. The author may **reassign** the selected approver on an open `in_review`
   request only by cancelling and submitting a new review request (no silent
   mid-flight approver swap).
8. Workspace catalogues for **document type** support add/rename/disable with
   the same reference-protection rule as confidentiality types (CAP-0008 /
   CAP-0013): types referenced by documents or history cannot be hard-deleted.
9. Master-data edits are recorded as `master_data_changed` events with
   before/after values. Changing title, document number, document type, owner,
   effective date, or effective confidentiality while content review is open
   or after approval invalidates that request/approval. Review-schedule changes
   do not invalidate content approval but remain auditable.
10. The explorer and document detail view show master data, current released
    version, next review due (with **overdue** highlight when past due and not
    obsolete), and lifecycle state.
11. Title, document type, and owner are required before review submission.
    Document number, when set, is trimmed and case-insensitively unique across
    active, unregistered, and obsolete records; a historical number is not
    silently reused. Date validation rejects a next-review date earlier than
    its effective date.
12. Draft Office files are working copies, not application-versioned source
    archives. The application restores draft content only from an operator
    workspace backup; released PDFs and their evidence remain the durable
    application-managed version history.

## Non-goals

- Multi-owner RACI matrices
- Automatic calendar reminders outside the app
- Enforcing legal retention deletion of PDFs
- Cross-document dependency graphs
- Automatic Office-draft version history or source-document diff/revert

## Links

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Library: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Maintenance: [`CAP-0013-library-maintenance.md`](CAP-0013-library-maintenance.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- ADR-0004, ADR-0013, ADR-0015, ADR-0016: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
