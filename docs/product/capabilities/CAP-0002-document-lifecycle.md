# CAP-0002 — Draft → approval → versioned PDF release lifecycle

| Field | Value |
| --- | --- |
| ID | CAP-0002 |
| Status | not implemented |
| Draft formats | Markdown (`.md`) and Microsoft Office originals (e.g. `.docx`, `.xlsx`, `.pptx`) |
| Released format | Versioned, classified PDF only (`*_VMAJOR.MINOR_<confidentiality-type-id>.pdf`) |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Only documents **in the library** (CAP-0006) participate in versioning and
   release. Uncontrolled files under the edit root are invisible to lifecycle
   actions until added.
2. The workspace configuration contains a workflow-person roster with stable
   IDs, display names, and email addresses, plus non-secret SMTP relay settings.
   The relay password is resolved from the OS credential store, not `.dms`.
3. Each library document has explicit `draft`, `in_review`, `approved`,
   `released`, and `obsolete` lifecycle states. `rejected`, `changed_requested`
   (decision), `withdrawn` (release), and `cancelled` (review) are workflow
   outcomes recorded on the event chain; they are not separate long-lived
   primary states except where CAP-0015 defines `obsolete`.
4. Submitting a document for review requires a non-empty change summary, the
   requesting workflow person, the document's effective configured approver,
   and a SHA-256 digest of the current draft. The requester identity and email
   are snapshotted on the request. The approver is derived from the nearest
   workflow-role policy or a document override (CAP-0019), and must use the
   installed desktop app with access to the same workspace; approval is not
   available in email or a browser. The notification carries a CAP-0020
   permalink (workspace ID + document ID + review-request target) to this
   review request (CAP-0010).
   After the first release it also requires an operator-selected change class
   (`cosmetic/minor` or `substantive/major`) with rationale. The class is bound
   to the review and any change requires a new review.
   Notification uses the workspace transport (CAP-0010): SMTP acceptance or
   operator-confirmed `mailto:` send. The document enters `in_review` only after
   that transport step succeeds; a failed send leaves the document in `draft`
   and offers a retryable redelivery.
5. The effective approver records `approved`, `rejected`, or
   `changed_requested` in the application with a non-empty decision comment.
   The app records the requester and approver identities, local OS user,
   decision time, revision digest, and chained event hash in `.dms`. It sends a
   notification of the recorded outcome to the requester's snapshotted email
   through the workspace transport (CAP-0010). A notification failure records a
   retryable delivery attempt and never reverses the decision. A
   `changed_requested` decision returns the document to `draft`.
6. If draft bytes no longer match the requested-review digest, approval is
   invalidated and the document returns to `draft`; a new change summary and
   review request are required.
7. On release, the **application** performs versioning and PDF export (CAP-0007):
   it produces a new PDF under the publish root at the mirrored relative
   directory, named `<stem>_V<major>.<minor>_<confidentiality-type-id>.pdf`
   (examples: `Policy_V1.0_restricted.pdf`, `Policy_V1.1_restricted.pdf`,
   `Policy_V2.0_confidential.pdf`). The filename uses the effective
   confidentiality type ID snapshotted for that release.
8. The editable source file remains the working draft under the edit root;
   release does not replace or delete it. Release records link draft path
   → versioned PDF path.
9. Version numbers are monotonic per stable document ID. The first release is
   `V1.0`. An approved cosmetic change increments minor (`V1.0` → `V1.1`);
   an approved substantive change increments major and resets minor to zero
   (`V1.7` → `V2.0`). There is no minor default when classification is
   uncertain: uncertain changes are substantive/major.
   A committed release number is never reused, including after withdrawal. A
   failed attempt before commit does not consume the number and leaves no final
   PDF. The app refuses to overwrite an existing PDF path.
10. Released state always points at a PDF produced by the app export path; the
    app does not accept an arbitrary operator-dropped PDF as a substitute for
    that export in the normal release flow.
11. Release is allowed only from a current `approved` revision and stores the
    approved source-draft SHA-256 digest, effective confidentiality type,
    effective editor and approver, approval-chain head, effective date, and
    next-review-due (CAP-0015 / CAP-0017 / CAP-0019) with the immutable release
    record. Release fails if the document is `obsolete` or `missing`.
12. Lifecycle, approval, and version history are readable after restart from
    `.dms`. Git is not required for lifecycle progression.
13. Each library document carries a stable document ID assigned at library add.
    The current draft **relative path** under the edit root is the locator, not
    the sole durable identity. Rename of the draft inside the edit root updates
    the locator and preserves ID and history (CAP-0013).
14. A `withdrawn` release moves its `released` history entry out of the
    active set but preserves the PDF on disk. A `rejected` review request
    leaves the document in `draft` and records the rejection reason in the
    canonical event chain. The decision event types recorded in the
    canonical event chain (CAP-0011) are `review_decision_approved`,
    `review_decision_rejected`, and `review_decision_changed_requested`.
15. If the operator renames or moves a controlled source draft outside the app,
    the next open of the workspace flags the document as `missing` until the
    operator reassigns it, removes it, or restores the file. A draft modified
    while a review is open invalidates the open approval (per outcome 6) without
    removing the request from history.
16. The release and approval history of a document is queryable by date range,
    approver, and confidentiality type, and is exportable as documented in
    CAP-0012.
17. After release, further content change uses **Begin revision** (CAP-0015),
    which returns the document to `draft` without deleting released PDFs.
    A new review/release cycle is required before the next versioned PDF.
18. Cancel-review, obsolescence, document control data, and current-version
    supersession follow CAP-0015. Publish-tree listing, orphan handling, and
    bulk verify follow CAP-0016. Due-date review of an unchanged current release
    follows CAP-0017.
19. Before submitting a review request and again immediately before release,
    the application derives the candidate release label from outcome 9 and
    checks the current source draft for its two canonical visible-content
    markers:
    - `Version: <major>.<minor>` must equal the candidate label without its
      filename `V` prefix (for example, candidate `V2.0` requires
      `Version: 2.0`).
    - `Vertraulichkeitsstufe: <display label>` must equal the effective
      confidentiality type's current display label (CAP-0008).
    A marker is absent, malformed, or mismatched when no canonical occurrence
    is found or when multiple occurrences do not all resolve to the expected
    value. Caption matching normalizes surrounding whitespace and casing;
    version and confidentiality values must match exactly after whitespace
    normalization. The DOCX scanner covers body text, tables, text boxes, and
    every header/footer part, including section-specific footers. The Markdown
    scanner checks rendered body text outside front matter, HTML comments, and
    fenced or indented code blocks. Other draft formats may not enter review or
    release until equivalent visible-content coverage is implemented and tested
    alongside CAP-0007.
    A failed check blocks the transition by default and reports the expected and
    detected marker values and locations without retaining other draft content.
    The operator may explicitly proceed after accepting a false-positive
    warning, but only with a non-empty reason. That override applies only to
    the current draft digest, candidate label, effective confidentiality type,
    and check phase; it is recorded as CAP-0011 evidence, is visible to the
    approver, and must be accepted again for the release-time check. It never
    edits the source draft or turns the failed check into a passing result.

## Capability-local rules

- Approval is operator-maintained (ADR-0004).
- The workflow hash chain is tamper-evident only within the trusted filesystem
  boundary; it is not identity verification or a digital signature.
- General notes remain governed by CAP-0003; review change summaries and
  approval decision comments are workflow evidence and cannot be edited or
  deleted through the notes UI.
- Each workflow event follows the canonical event body defined in ADR-0013;
  recomputing and re-hashing is the verification routine exposed to operators
  (CAP-0012).
- PDF export and file versioning are application responsibilities using
  format-specific local exporters (CAP-0007, ADR-0008).
- Content-conformance overrides are exceptional workflow evidence, not a
  substitute for correcting the source draft or its configured classification.
- Naming pattern and dual-root placement: ADR-0006, ADR-0007.
- Cosmetic means spelling, grammar, formatting, pagination, or equivalent
  presentation-only correction that does not change meaning, obligation,
  process step, role, control, scope, decision, or data handling. Any such
  semantic change—or uncertainty about semantic impact—is substantive/major.
- Claude Desktop may suggest a class and change-summary wording under CAP-0018,
  but the operator remains responsible for both and approval remains mandatory.

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0002-document-lifecycle.html`](../wireframes/html/CAP-0002-document-lifecycle.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0002-document-lifecycle.png`](../wireframes/exports/CAP-0002-document-lifecycle.png)

- Architecture: [`../../architecture.md`](../../architecture.md)
- ADR-0003, ADR-0004, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0010,
  ADR-0012, ADR-0013, ADR-0015, ADR-0016, ADR-0019: [`../../design-decisions.md`](../../design-decisions.md)
- Export: [`CAP-0007-draft-pdf-export.md`](CAP-0007-draft-pdf-export.md)
- Library: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Classification: [`CAP-0008-confidentiality-classification.md`](CAP-0008-confidentiality-classification.md)
- Editor: [`CAP-0009-release-editor.md`](CAP-0009-release-editor.md)
- Notification: [`CAP-0010-notification-transport.md`](CAP-0010-notification-transport.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Audit/export: [`CAP-0012-audit-export.md`](CAP-0012-audit-export.md)
- Library maintenance: [`CAP-0013-library-maintenance.md`](CAP-0013-library-maintenance.md)
- Workspace integrity: [`CAP-0014-workspace-integrity.md`](CAP-0014-workspace-integrity.md)
- Document control data / revision cycle: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Publish tree: [`CAP-0016-publish-tree-maintenance.md`](CAP-0016-publish-tree-maintenance.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Claude Desktop assistance: [`CAP-0018-claude-desktop-change-assistance.md`](CAP-0018-claude-desktop-change-assistance.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
