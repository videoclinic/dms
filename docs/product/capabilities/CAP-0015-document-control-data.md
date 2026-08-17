# CAP-0015 — Document control data and revision cycle

| Field | Value |
| --- | --- |
| ID | CAP-0015 |
| Status | implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | Phases 9e–9f.5, 9k.3, 9k.4, and 9l plus CHG-0004 Phase 2 evidence: [release-snapshot, frontmatter comparison, migration, identity-stability, atomic-handover, and unregister-independence core tests](../../../crates/dms-core/tests/lifecycle.rs), [document-profile and legacy-owner migration tests](../../../crates/dms-core/tests/workspace.rs), [desktop adapter commands](../../../crates/dms-desktop/src/lib.rs), [Library document-control, bounded details-pane, release-profile, placeholder, and lifecycle tests](../../../crates/dms-desktop/ui/library.test.mjs), [document-type catalogue tests](../../../crates/dms-desktop/ui/configuration.test.mjs) |

## Outcomes

1. Each library document stores DMS-managed **document control data** under
   `<edit-root>/.dms/`, separate from the source file:
   - **title** (required operator-editable display name; defaults to draft stem)
   - **document number** (optional control identifier, unique in the workspace)
   - **document type** (from a workspace catalogue, e.g. policy, procedure,
     form, record, other)
   - **owner** (the selected eligible person's current-group binding and immutable
     Entra object ID; refreshed name/email are presentation only)
   These values are not imported from Office built-in or custom document
   properties. For registered Markdown library members, DMS is the source of
   truth for the controlled frontmatter keys `title`, `document_number`,
   `version`, and `confidentiality`: it prefills them from library defaults and
   overwrites them when DMS control data, effective confidentiality, or the
   candidate target version changes. Frontmatter `confidentiality` holds the
   catalogue type ID (not the display label). Frontmatter never mutates `.dms`
   control data. The draft filename stem supplies the title's one-time default only
   when the document is added; it is not a continuing metadata source.
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
3. After a successful release, the document is `released` while the current
   draft bytes still match that release's source digest. When the draft changes
   (digest no longer matches the latest non-withdrawn release) or the document
   was never released, lifecycle is `draft`. Released PDFs and history remain.
   The source draft stays the editable working file. There is no **Begin
   revision** control.
4. While a newer draft exists after a release, the explorer surfaces
   **released version label** and **draft is newer than last
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
10. Document-profile edits are recorded as
    `document_control_data_changed` events with before/after values. Changing
    title, document number, document type, or owner while
    content review is open or after approval invalidates that request/approval.
    Changing the document's effective confidentiality through CAP-0008 has the
    same invalidation effect. Review-schedule changes do not invalidate content
    approval but remain auditable. Effective date is not mutable profile data.
11. The CAP-0006 selection pane and any expanded document-control-data form show
    mutable document profile, current released version (with its immutable
    release-time profile, Owner snapshot, required candidate effective date, and
    a recent release list
    when space allows),
    next review due (with **overdue** highlight when past due and not obsolete),
    lifecycle state, effective confidentiality with its source and
    inherited/overridden status (CAP-0008), and effective editor/approver
    (CAP-0019). For exactly one selected document, the pane's main
    document-detail scroller groups **Document control data**, **Document
    review schedule**, **Revision cycle**, and **Releases** into independently
    foldable, frameless sections (no surrounding card frames around those
    topics). Each section summary shows a disclosure chevron and Expand/Collapse
    cue so foldability is visible. **Actions** is a separately foldable
    disclosure docked at the bottom of the pane, outside that scroller, expanded
    by default, using the same chevron and cue. Its summary stays fully visible
    at heading height. Unfolding it shows every document action, including
    Lost-source reassociate when applicable. The main details scroller shrinks
    first; the Actions body scrolls only if the pane is shorter than the
    heading plus those actions. The document
    identity, lifecycle badge, and Source file identity remain visible while any
    section is folded. Fold open/closed state, including Actions, is
    session-only Library UI state shared across document switches in the open
    Library activity; it is not stored in `.dms`, preferences, or saved views.
    When a
    current released version exists, **Open latest released PDF**
    opens only that release record's recorded PDF; it is unavailable for a
    never-released document or when the current release file is missing. When
    the main detail sections exceed the available window height, only that
    scroller moves under CAP-0006; it never moves Library or application
    navigation or the Actions summary. CAP-0006's
    bounded session splitter may widen this
    pane without changing its field rules, independent-scroll contract, or
    persisted document data.
12. Title, document type, a resolved eligible owner, and a candidate effective
    date are required before review submission.
    Document number, when set, is trimmed and case-insensitively unique across
    active, unregistered, and obsolete records; a historical number is not
    silently reused. Date validation rejects a next-review date earlier than
    the release effective date used to anchor that schedule.
13. Draft source files are working copies, not application-versioned source
    archives. Renaming or reassociating a source file updates only its stored
    locator and the filesystem-derived **Source file** display; it does not
    change the document's title, number, type, owner, release dates, review
    schedule, lifecycle state,
    or history. Office document properties are not authoritative for document
    control data. Registered Markdown drafts receive controlled frontmatter keys
    written from DMS (CAP-0002 / CAP-0007); that one-way projection never imports
    frontmatter into `.dms`. The application restores draft content only from an
    operator workspace backup; released PDFs and their evidence remain the
    durable application-managed version history.
14. **Create release candidate** (when the document is an idle draft with no
    active candidate), **Mark obsolete**, and **Cancel review** are available
    from the CAP-0006 selection pane (single selection) when preconditions hold;
    disabled states explain why. In **Revision cycle**, the candidate form is
    listed first when available, uses **Create release candidate** whether the
    selected target skips approval or opens review, always includes **Review
    content-check override reason (only when needed)**, shows the resolved
    effective target version for Next minor / Next major / Manual, labels later
    Next minor options **approval optional**, and keeps Manual major/minor
    disabled unless Manual target is selected. Canonical workflow evidence is a single
    foldable disclosure in that section. There is no **Begin revision** action:
    Draft/Released follows CAP-0002 digest rules. These actions are not offered
    as multi-select batch actions unless a future CAP explicitly allows bulk
    obsolescence.
15. A candidate may stage a replacement Owner and responsible Editor selected
    from currently eligible people. Their object-ID references apply atomically
    only after PDF export and release metadata save succeed. Export or save
    failure leaves the prior owner and routing state unchanged. Release history
    retains the candidate-time display snapshots even if Entra later changes a
    person's name or email.
16. A non-empty schema-v11 free-text owner migrates only to a display-only legacy
    label and never assigns authority by matching text. The current non-withdrawn
    v11 release receives the stored effective date when one exists; earlier or
    withdrawn releases remain visibly unrecorded. Existing review due dates,
    candidates, versions, paths, digests, and workflow evidence are preserved.

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
- Membership vs obsolescence: [`../../library-membership-and-obsolescence.md`](../../library-membership-and-obsolescence.md)
- Maintenance: [`CAP-0013-library-maintenance.md`](CAP-0013-library-maintenance.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- ADR-0004, ADR-0013, ADR-0015, ADR-0016, ADR-0026: [`../../design-decisions.md`](../../design-decisions.md)
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
