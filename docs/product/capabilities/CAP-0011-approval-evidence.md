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
   timestamp, configured approver identity (when applicable), local OS user,
   revision digest (when applicable), confidentiality snapshot (when
   applicable), approved change class and rationale (when applicable), and
   operator comment text. The chain head is the SHA-256 of the canonical body.
2. Two comment types are first-class:
   - **Change comment** — entered at review-request time; explains what the
     author changed since the last release.
   - **Decision comment** — entered by the approver at `approved`,
     `rejected`, or `changes_requested` time; explains the rationale.
   Both are required text (no empty comment) and are part of the canonical
   event body, not editable later.
3. Every workflow event records a single local OS user. The application does
   not enforce identity; it records what the OS reports.
4. The history of a document lists every event in chain order with its event
   hash, predecessor hash, type, timestamp, author comment, decision comment,
   revision digest, approved change class, and confidentiality snapshot. The
   list is readable without leaving the desktop app.
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
8. Additional workflow event types required for maintenance are first-class in
   the same chain: `review_cancelled`, `revision_begun`, `document_obsoleted`,
   `master_data_changed`, `release_withdrawn`, and `report_generated`. Each
   uses the canonical body and required comment/reason fields defined by the
   owning CAP.
9. Periodic-review events (`periodic_review_requested`,
   `periodic_review_reminded`, `periodic_review_cancelled`, and
   `periodic_review_completed`) use the same canonical chain and bind to the
   reviewed release record and PDF digest (CAP-0017).

## Non-goals

- Cross-workspace chain comparison
- Tamper repair or chain rewriting
- Replacing filesystem ACLs or audit logging of the OS

## Links

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Master data: [`CAP-0015-document-master-data.md`](CAP-0015-document-master-data.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0004, ADR-0013: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
