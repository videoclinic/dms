# CAP-0002 — Draft → approval → versioned PDF release lifecycle

| Field | Value |
| --- | --- |
| ID | CAP-0002 |
| Status | implemented |
| Draft formats | Markdown (`.md`) and Microsoft Office originals (e.g. `.docx`, `.xlsx`, `.pptx`) |
| Released format | Versioned, classified PDF only (`*_VMAJOR.MINOR_<confidentiality-type-id>.pdf`) |
| Tests | Phases 9e, 9f.2, 9k.3, 9k.4, and 9l plus CHG-0004 Phase 2 evidence: [withdrawal/current-release/version-allocation, release-snapshot, frontmatter conformance, identity-stability, migration, and atomic-handover core tests](../../../crates/dms-core/tests/lifecycle.rs), [desktop lifecycle adapter tests](../../../crates/dms-desktop/src/lib.rs), [Library lifecycle and candidate-default frontend tests](../../../crates/dms-desktop/ui/library.test.mjs) |

## Outcomes

1. Only documents **in the library** (CAP-0006) participate in versioning and
   release. Uncontrolled files under the edit root are invisible to lifecycle
   actions until added.
2. App-global OS-user configuration supplies the Microsoft Entra public-client
   and tenant IDs, while workspace configuration binds workflow routing to one
   Microsoft Entra group plus non-secret SMTP relay settings. The group supplies eligible
   people on demand; it is not copied into an application user roster. The relay
   password and Microsoft Entra delegated-token cache are resolved from the OS
   credential store, not `.dms`.
3. Each library document has explicit `draft`, `in_review`, `approved`,
   `released`, and `obsolete` lifecycle states. `rejected`, `changed_requested`
   (decision), `withdrawn` (release), and `cancelled` (review) are workflow
   outcomes recorded on the event chain; they are not separate long-lived
   primary states except where CAP-0015 defines `obsolete`. The **publish root**
   is the storage destination for released PDFs; `published` is not a state or
   workflow. The explicit release action is the sole transition that creates a
   released PDF.
4. Before every release, the editor records a non-empty changelog, a required
   effective date, the requesting workflow person, and a SHA-256 digest of the
   current draft. The Library **Revision cycle** candidate form is titled
   **Create release candidate** and explains that it records the candidate in
   this workspace (not an external “submit” destination). That title and submit
   label stay **Create release candidate** whether the selected target skips
   approval or opens review. The form always includes **Review content-check
   override reason (only when needed)** for both approval-optional and
   approval-required targets. The form defaults its
   target-version control to **Next minor** and always shows the **effective
   target version** that the selected mode resolves to. After the first
   release, the Next minor option is labeled **approval optional**; Next major
   remains **approval required**. First-release Next minor stays
   `Next minor · V1.0 (first release)` because `V1.0` still requires approval.
   **Manual major** and
   **Manual minor** stay disabled until **Manual target** is selected. The first
   release still resolves to `V1.0`. The requester identity and email are
   snapshotted with the release candidate. The first release proposes `V1.0`.
   For every later release, the editor selects exactly one target-version mode:
   - **Minor version change** proposes the next minor version of the current
     release (`V1.3` → `V1.4`; never-released → `V1.0`).
   - **Major version change** proposes the next major version and resets the
     minor component (`V1.3` → `V2.0`; never-released → `V1.0`).
   - **Manual version set** supplies `V<major>.<minor>` with non-negative integer
     components, numerically greater than the current released version, and not
     equal to any committed release version for that document. It may skip
     otherwise-unused values.
   The candidate snapshots the changelog, effective date, target-version mode,
   label, mutable document profile, effective confidentiality, and resolved
   workflow people. It may also stage a newly selected eligible Owner and Editor
   for application only by the same successful release commit. The candidate is
   not a reservation. `V1.0` and every candidate whose major component is greater
   than the current released major component require approval. A manual target
   follows the same rule from its target's major component. For an approval-required
   candidate, the effective approver is derived from the nearest workflow-role
   policy or a document override (CAP-0019), resolves as an eligible member of
   the configured Microsoft Entra group (CAP-0021), and must use the installed
   desktop app with access to the same workspace; approval is not available in
   email or a browser. The review notification carries a CAP-0020 permalink
   (workspace ID + document ID + review-request target) and uses the workspace
   transport (CAP-0010). The document enters `in_review` only after SMTP
   acceptance or operator-confirmed `mailto:` send; a failed send leaves it in
   `draft` and offers a retryable redelivery. A minor candidate does not create a
   review request or enter `in_review`; it remains in `draft` until direct release.
5. For an approval-required candidate, the effective approver records `approved`, `rejected`, or
   `changed_requested` in the application. A decision comment is optional. On a
   `rejected` or `changed_requested` decision, the UI asks **Why was approval not
   granted?** but permits an empty response; any supplied comment is immutable
   workflow evidence. The app requires interactive Microsoft Entra sign-in and
   accepts the decision
   only when the signed-in tenant/object ID equals the request's snapshotted
   approver and the person remains eligible in the configured group. It records
   requester, approver, Entra actor, local OS user, decision time, revision
   digest, and chained event hash in `.dms`. It sends a notification of the
   recorded outcome to the requester's snapshotted email through the workspace
   transport (CAP-0010). A notification failure records a retryable delivery
   attempt and never reverses the decision. A `changed_requested` decision
   returns the document to `draft`.
6. If draft bytes no longer match an approval-required requested-review digest, approval is
   invalidated and the document returns to `draft`; a new changelog, target
   version selection, and review request are required.
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
   `V1.0`. Each later release carries its editor-selected candidate from outcome
   4. Approval records an accepted approval-required target, while a minor target
   proceeds without a decision. Only successful atomic export commits either
   target as a release number. A candidate in a rejected, changes-requested,
   cancelled, invalidated, or failed-export review does not consume or occupy a
   version and may be selected again on a later review. A committed release
   number is never reused, including after withdrawal. The app refuses to
   overwrite an existing PDF path.
10. Released state always points at a PDF produced by the app export path; the
    app does not accept an arbitrary operator-dropped PDF as a substitute for
    that export in the normal release flow.
11. Release is explicit. **Release approved version** is allowed only from a
    current `approved` approval-required revision. **Release minor version** is
    allowed from a current `draft` minor candidate after all release-time checks.
    Both actions store the source-draft SHA-256 digest, target-version mode and
    label, changelog, immutable profile and owner presentation snapshots,
    effective confidentiality type, effective editor and approver, effective
    date, and next-review-due (CAP-0015 / CAP-0017 / CAP-0019) with the immutable
    release record. Display-name or email changes after candidate submission do
    not change identity equality, which uses tenant-scoped Entra object IDs. An
    approval-required release
    additionally stores its approval-chain head. A successfully released minor
    version sends the effective approver a CAP-0010 publication notification for
    that document. Notification failure creates a retryable delivery attempt and
    never reverses the committed release. Release fails if the document is
    `obsolete` or `missing`.
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
17. After release, further content change is automatic: when the draft digest no
    longer matches the latest non-withdrawn release source digest, the document
    becomes `draft` without a **Begin revision** action (CAP-0015 / ADR-0016).
    Never-released registered documents remain `draft`. A new review/release
    cycle is required before the next versioned PDF.
18. Cancel-review, obsolescence, document control data, and current-version
    supersession follow CAP-0015. Publish-tree listing, orphan handling, and
    bulk verify follow CAP-0016. Due-date review of an unchanged current release
    follows CAP-0017.
19. Before submitting an approval-required review request and again immediately
    before every release, the application derives the candidate release label
    from outcome 9 and validates the source metadata that the draft format owns:
    - an Office draft must contain the two canonical visible-content markers;
    - a Markdown draft must contain matching flat YAML frontmatter fields. Its
      generated temporary Word document receives visible fields from the release
      snapshot under CAP-0007, so the Markdown body need not duplicate them.
    The canonical values are:
    - `Version: <major>.<minor>` must equal the candidate label without its
      filename `V` prefix (for example, candidate `V2.0` requires
      `Version: 2.0`).
    - `Vertraulichkeitsstufe: <display label>` (Office drafts) must equal the
      effective confidentiality type's current display label (CAP-0008).
    - Markdown frontmatter `confidentiality: <type-id>` must equal the effective
      confidentiality type's stable ID (CAP-0008), not the display label.
    A marker is absent, malformed, or mismatched when no canonical occurrence
    is found or when multiple occurrences do not all resolve to the expected
    value. Caption matching normalizes surrounding whitespace and casing;
    version and confidentiality values must match exactly after whitespace
    normalization. The DOCX scanner covers body text, tables, text boxes, and
    every header/footer part, including section-specific footers (including
    unresolved `{VERSION}` / `{CONFIDENTIALITY}` tokens, which do not satisfy
    the gate until export would replace them — the gate reads the draft on
    disk, not the temp export copy). The Markdown scanner requires scalar
    `version` and `confidentiality` frontmatter fields. Optional `title` and
    `document_number` fields must match the candidate snapshot when present;
    duplicate, structured, or malformed controlled fields fail closed.
    For registered Markdown library members, DMS owns those controlled
    frontmatter keys: it prefills them from document control and library
    settings on add/reassociate and overwrites them whenever DMS control data,
    effective confidentiality, or the candidate target version changes.
    Frontmatter `confidentiality` stores the catalogue **type ID**; the display
    label is used for Office markers and export chrome only. Frontmatter never
    supplies authoritative values into `.dms`. Additional flat
    frontmatter scalars may fill non-controlled Word-template `{KEY}` variables
    under CAP-0007 during temporary DOCX assembly only. Other draft formats may
    not enter review or release until equivalent visible-content coverage is
    implemented and tested alongside CAP-0007.
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
- General notes remain governed by CAP-0003; review changelogs and supplied
  decision comments are workflow evidence and cannot be edited or deleted
  through the notes UI.
- Each workflow event follows the canonical event body defined in ADR-0013;
  recomputing and re-hashing is the verification routine exposed to operators
  (CAP-0012).
- PDF export and file versioning are application responsibilities using
  format-specific local exporters (CAP-0007, ADR-0008).
- Content-conformance overrides are exceptional workflow evidence, not a
  substitute for correcting the source draft or its configured classification.
- Naming pattern and dual-root placement: ADR-0006, ADR-0007.
- Claude Desktop may suggest a target-version mode and changelog wording under
  CAP-0018, but the editor remains responsible for the selected target and
  approval is required only for approval-required candidates.

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0002-document-lifecycle.html`](../wireframes/html/CAP-0002-document-lifecycle.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0002-document-lifecycle.png`](../wireframes/exports/CAP-0002-document-lifecycle.png)

- Architecture: [`../../architecture.md`](../../architecture.md)
- ADR-0003, ADR-0004, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0010,
  ADR-0012, ADR-0013, ADR-0015, ADR-0016, ADR-0019, ADR-0021: [`../../design-decisions.md`](../../design-decisions.md)
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
- Membership vs obsolescence: [`../../library-membership-and-obsolescence.md`](../../library-membership-and-obsolescence.md)
- Publish tree: [`CAP-0016-publish-tree-maintenance.md`](CAP-0016-publish-tree-maintenance.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Claude Desktop assistance: [`CAP-0018-claude-desktop-change-assistance.md`](CAP-0018-claude-desktop-change-assistance.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- Workflow identity: [`CAP-0021-microsoft-entra-workflow-identity.md`](CAP-0021-microsoft-entra-workflow-identity.md)
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
