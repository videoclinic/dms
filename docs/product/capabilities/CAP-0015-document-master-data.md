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
2. Master data for the **currently selected library document** is shown on the
   **same CAP-0006 library page** in the selection pane (right column), together
   with document actions for that selection. The pane header uses the **same
   title string** as the list row (plus document number and relative path).
   Editing opens an inline or modal master-data form without requiring a
   separate top-level navigation destination. CAP-0006 owns navigation and
   selection; this CAP owns fields, validation, revision/obsolescence actions,
   and audit events. Action labels do not repeat the document identity.
3. After a successful release, the document remains `released` for the current
   version. Starting the next change cycle is an explicit **Begin revision**
   action that returns the document to `draft` while retaining the released
   PDF and history. The Office draft stays the editable working file.
4. While a newer draft exists after a release (or after Begin revision), the
   explorer surfaces **released version label** and **draft is newer than last
   release** when current draft bytes differ from the approved Office-draft
   digest stored on the last successful release record.
5. The **current released version** is the latest non-withdrawn release
   record. A later release supersedes the previous current version for
   explorer display; prior PDFs remain on disk and remain verifiable.
6. An operator can mark a library document **obsolete**: lifecycle becomes
   `obsolete`, no further review or release is allowed, released PDFs remain
   on disk, and the explorer filters obsolete documents behind an explicit
   show/hide control. Obsolescence records a canonical workflow event with a
   required reason comment.
7. An author (any operator with the workspace open) can **cancel** an open
   `in_review` request with a required comment. The document returns to
   `draft`; the cancellation is recorded in the event chain. Cancel does not
   delete prior events.
8. An operator may change a document's effective approver only through its
   applicable folder policy or document override (CAP-0019). Changing the
   effective approver while a request is open invalidates that request; a new
   review request is required. There is no silent mid-flight approver swap.
9. Workspace catalogues for **document type** support add/rename/disable with
   the same reference-protection rule as confidentiality types (CAP-0008 /
   CAP-0013): types referenced by documents or history cannot be hard-deleted.
10. Master-data edits are recorded as `master_data_changed` events with
    before/after values. Changing title, document number, document type, owner,
    effective date, or effective confidentiality while content review is open
    or after approval invalidates that request/approval. Review-schedule changes
    do not invalidate content approval but remain auditable.
11. The CAP-0006 selection pane and any expanded master-data form show master
    data, current released version (with recent release list when space allows),
    next review due (with **overdue** highlight when past due and not obsolete),
    lifecycle state, and effective editor/approver (CAP-0019). Document actions
    for the single selection sit in the same pane under the master-data block.
    When a current released version exists, **Open latest released PDF** opens
    only that release record's recorded PDF; it is unavailable for a
    never-released document or when the current release file is missing.
12. Title, document type, and owner are required before review submission.
    Document number, when set, is trimmed and case-insensitively unique across
    active, unregistered, and obsolete records; a historical number is not
    silently reused. Date validation rejects a next-review date earlier than
    its effective date.
13. Draft Office files are working copies, not application-versioned source
    archives. The application restores draft content only from an operator
    workspace backup; released PDFs and their evidence remain the durable
    application-managed version history.
14. **Begin revision**, **Mark obsolete**, and **Cancel review** are available
    from the CAP-0006 selection pane (single selection) when preconditions hold;
    disabled states explain why. They are not offered as multi-select batch
    actions unless a future CAP explicitly allows bulk obsolescence.

## Non-goals

- Multi-owner RACI matrices
- Automatic calendar reminders outside the app
- Enforcing legal retention deletion of PDFs
- Cross-document dependency graphs
- Automatic Office-draft version history or source-document diff/revert
- A separate primary app section whose only job is master data outside the
  library navigator (detail may still expand full-width for editing)

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0015-document-master-data.html`](../wireframes/html/CAP-0015-document-master-data.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0015-document-master-data.png`](../wireframes/exports/CAP-0015-document-master-data.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Library: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Maintenance: [`CAP-0013-library-maintenance.md`](CAP-0013-library-maintenance.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- ADR-0004, ADR-0013, ADR-0015, ADR-0016: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
