# CAP-0015 — Document control data and revision cycle

| Field | Value |
| --- | --- |
| ID | CAP-0015 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Each library document stores DMS-managed **document control data** under
   `<edit-root>/.dms/`, separate from the source file:
   - **title** (required operator-editable display name; defaults to draft stem)
   - **document number** (optional control identifier, unique in the workspace)
   - **document type** (from a workspace catalogue, e.g. policy, procedure,
     form, record, other)
   - **owner** (display name of the maintaining operator)
   - **effective date** of the current released version (set on release;
     empty while never released)
   - **next review due date** (computed and maintained by CAP-0017)
   These values are not imported from or synchronized with Office built-in or
   custom document properties, or Markdown front matter. The draft filename stem
   supplies the title's one-time default only when the document is added; it is
   not a continuing metadata source.
2. Document control data for the **currently selected library document** is
   shown on the **same CAP-0006 library page** in the selection pane (right
   column), together with document actions for that selection. The pane keeps
   two visibly distinct sources:
   - **Source file** — exact filesystem file name and edit-root-relative
     folder/path; read-only in this pane and identified as filesystem-derived
   - **Document control data** — identified as managed by DMS Desktop and stored
     in workspace metadata under `.dms`
   The pane header uses the DMS-managed title and document number. Editing opens
   an inline or modal **Edit document control data** form without requiring a
   separate top-level navigation destination. CAP-0006 owns navigation and
   selection; this CAP owns fields, validation, revision/obsolescence actions,
   and audit events. Action labels do not repeat the document identity.
3. After a successful release, the document remains `released` for the current
   version. Starting the next change cycle is an explicit **Begin revision**
   action that returns the document to `draft` while retaining the released
   PDF and history. The source draft stays the editable working file.
4. While a newer draft exists after a release (or after Begin revision), the
   explorer surfaces **released version label** and **draft is newer than last
   release** when current draft bytes differ from the approved source-draft
   digest stored on the last successful release record.
5. The **current released version** is the latest non-withdrawn release
   record. A later release supersedes the previous current version for
   explorer display; prior PDFs remain on disk and remain verifiable.
6. An operator can mark a library document **obsolete**: lifecycle becomes
   `obsolete`, no further review or release is allowed, and released PDFs remain
   on disk. Its draft remains visible at its filesystem location in the
   CAP-0006 directory listing with an `obsolete` state. Obsolescence records a
   canonical workflow event with a required reason comment.
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
10. Document-control-data edits are recorded as
    `document_control_data_changed` events with
    before/after values. Changing title, document number, document type, owner,
    effective date, or effective confidentiality while content review is open
    or after approval invalidates that request/approval. Review-schedule changes
    do not invalidate content approval but remain auditable.
11. The CAP-0006 selection pane and any expanded document-control-data form show
    document control data, current released version (with a recent release list
    when space allows),
    next review due (with **overdue** highlight when past due and not obsolete),
    lifecycle state, and effective editor/approver (CAP-0019). For exactly one
    selected document, the pane groups **Document control data**, **Actions**,
    **Revision cycle**, and **Releases** into independently foldable sections;
    the document identity and lifecycle state remain visible while any section
    is folded. Document actions sit in the same pane under the document control
    data block. When a current released version exists, **Open latest released PDF**
    opens only that release record's recorded PDF; it is unavailable for a
    never-released document or when the current release file is missing.
12. Title, document type, and owner are required before review submission.
    Document number, when set, is trimmed and case-insensitively unique across
    active, unregistered, and obsolete records; a historical number is not
    silently reused. Date validation rejects a next-review date earlier than
    its effective date.
13. Draft source files are working copies, not application-versioned source
    archives. Renaming or reassociating a source file updates only its stored
    locator and the filesystem-derived **Source file** display; it does not
    change the document's title, number, type, owner, dates, lifecycle state,
    or history. Office document properties and Markdown front matter are likewise
    not authoritative for document control data. The application restores draft
    content only from an operator workspace backup; released PDFs and their
    evidence remain the
    durable application-managed version history.
14. **Begin revision**, **Mark obsolete**, and **Cancel review** are available
    from the CAP-0006 selection pane (single selection) when preconditions hold;
    disabled states explain why. They are not offered as multi-select batch
    actions unless a future CAP explicitly allows bulk obsolescence.

## Non-goals

- Multi-owner RACI matrices
- Automatic calendar reminders outside the app
- Enforcing legal retention deletion of PDFs
- Cross-document dependency graphs
- Automatic source-draft version history or source-document diff/revert
- Importing or synchronizing document control data from source-file metadata
- A separate primary app section whose only job is document control data
  outside the library navigator (detail may still expand full-width for editing)

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0015-document-control-data.html`](../wireframes/html/CAP-0015-document-control-data.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0015-document-control-data.png`](../wireframes/exports/CAP-0015-document-control-data.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Library: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Maintenance: [`CAP-0013-library-maintenance.md`](CAP-0013-library-maintenance.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- ADR-0004, ADR-0013, ADR-0015, ADR-0016: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
