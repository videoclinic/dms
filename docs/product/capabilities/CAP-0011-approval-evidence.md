# CAP-0011 — Approval evidence (change comments and decision comments)

| Field | Value |
| --- | --- |
| ID | CAP-0011 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` (canonical event chain) |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. Every workflow event follows the canonical event body defined in ADR-0013:
   stable document ID, event type, predecessor event hash, ISO-8601 UTC
   timestamp, requester, effective editor and approver IDs (when applicable),
   local OS user, revision digest (when applicable), confidentiality snapshot
   (when applicable), approved change class and rationale (when applicable),
   and operator comment text. The chain head is the SHA-256 of the canonical
   body.
2. Two comment types are first-class:
   - **Change comment** — entered at review-request time; explains what the
     author changed since the last release.
   - **Decision comment** — entered by the approver at decision time;
     explains the rationale.
   Both are required text (no empty comment) and are part of the canonical
   event body, not editable later.
3. Every workflow event records a single local OS user. The application does
   not enforce identity; it records what the OS reports.
4. The history of a document lists every event in chain order with its event
   hash, predecessor hash, type, timestamp, author comment, decision comment,
   requester, effective editor and approver, revision digest, approved change
   class, and confidentiality snapshot. The list is readable without leaving
   the desktop app.
5. The application exposes a **Verify workflow** routine that recomputes each
   event hash from its canonical body and confirms the chain. The result is
   `valid`, `tampered at <event-id>`, or `missing`. Verification never
   rewrites any data.
6. The event chain head is stored with each release record so a downstream
   reader can confirm that the released version was the approved revision.
7. Comment length, encoding, and disallowed characters are documented and
   enforced at entry. Comments are stored as UTF-8 text; line breaks are
   preserved. A line length limit applies for legibility (default 500
   characters per line, configurable per workspace).
8. The canonical event types are:
   - `review_requested` — review-notification sent; document is `in_review`
   - `review_decision_approved` — approver approved the current draft
   - `review_decision_rejected` — approver rejected; document returns to `draft`
   - `review_decision_changed_requested` — approver asked for changes; document returns to `draft`
   - `release` — successful versioned PDF write under the publish root
   - `release_withdrawn` — release record removed from the active current set; PDF preserved
   - `review_cancelled` — author (or operator) cancelled an open review
   - `revision_begun` — released document returned to `draft` for the next cycle
   - `document_obsoleted` — document moved to terminal `obsolete` state
   - `document_control_data_changed` — any editable document-control-data field changed
   - `content_conformance_overridden` — operator accepted a failed version or
     confidentiality content check for a review or release candidate; embeds
     the check phase, candidate version, effective confidentiality ID and
     display label, marker verdicts/locations, current draft digest, and a
     non-empty operator reason, but no other draft content
   - `report_generated` — audit or other report exported from the workspace
   Each event uses the canonical body and required comment/reason fields
   defined by the owning CAP. The release event embeds the version label
   (`VMAJOR.MINOR`), the produced PDF digest, and the approved Office-draft
   digest; the document-control-data event embeds before/after values.
9. Periodic-review events (`periodic_review_requested`,
   `periodic_review_reminded`, `periodic_review_cancelled`, and
   `periodic_review_completed`) use the same canonical chain and bind to the
   reviewed release record and PDF digest (CAP-0017).

## Non-goals

- Cross-workspace chain comparison
- Tamper repair or chain rewriting
- Replacing filesystem ACLs or audit logging of the OS

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0011-approval-evidence.html`](../wireframes/html/CAP-0011-approval-evidence.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0011-approval-evidence.png`](../wireframes/exports/CAP-0011-approval-evidence.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- Document control data: [`CAP-0015-document-master-data.md`](CAP-0015-document-master-data.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0004, ADR-0013: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
