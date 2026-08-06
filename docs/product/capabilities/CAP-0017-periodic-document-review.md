# CAP-0017 — Periodic document review

| Field | Value |
| --- | --- |
| ID | CAP-0017 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/` |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The workspace defines a positive default review interval in months. A
   document may override the interval or be review-exempt; setting an exemption
   requires a reason comment recorded in the workflow event chain.
2. A successful release computes the next-review-due date from the release
   effective date and the document interval. Completing a periodic review
   computes the next date from the completion date. Dates use calendar months,
   clamping to the last valid day of the target month.
3. On workspace open, the explorer marks documents due within 30 days and
   overdue documents. The app does not claim background reminders while it is
   closed.
4. A periodic review can start only for a current, non-withdrawn released
   version. The request binds the stable document ID, release record ID,
   released PDF SHA-256 digest, confidentiality snapshot, and selected
   configured reviewer.
5. The request uses CAP-0010 notification transport and contains no document
   attachment. At most one periodic review or content-approval request may be
   open for a document at a time.
6. The reviewer records one result with a required comment:
   - **confirmed current** keeps the current release and version, records the
     review, and advances the next-review-due date;
   - **changes required** records the review and invokes CAP-0015 **Begin
     revision** while the last release remains current until superseded or
     withdrawn;
   - **obsolete** records the review and invokes CAP-0015 obsolescence.
7. If the released PDF is missing or its checksum no longer matches its release
   record, periodic review is blocked until the integrity problem is resolved.
8. Periodic-review request, cancellation, reminder, and result are canonical
   workflow events under CAP-0011. Prior review records are immutable and
   included in CAP-0012 audit exports.
9. An operator may cancel an open periodic review with a required comment. The
   existing release and due date remain unchanged.
10. Reminder email is an explicit operator action from the due/overdue list.
    Sending a reminder does not create another review request and cannot change
    lifecycle state.

## Non-goals

- Background scheduling while the desktop app is closed
- Calendar-service integration
- Automatic content approval based only on elapsed time
- Releasing a new PDF when a reviewer confirms unchanged content

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0017-periodic-document-review.html`](../wireframes/html/CAP-0017-periodic-document-review.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0017-periodic-document-review.png`](../wireframes/exports/CAP-0017-periodic-document-review.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Notification: [`CAP-0010-notification-transport.md`](CAP-0010-notification-transport.md)
- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Master data: [`CAP-0015-document-master-data.md`](CAP-0015-document-master-data.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
