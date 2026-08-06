# CAP-0010 — Notification transport (SMTP or host mail handler)

| Field | Value |
| --- | --- |
| ID | CAP-0010 |
| Status | not implemented |
| Transports | Configured SMTP relay (ADR-0009) and host default mail handler via `mailto:` (ADR-0012) |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The workspace configuration stores the selected notification transport
   (`smtp` or `mailto`) and the non-secret SMTP relay settings. The relay
   password is always resolved from the OS credential store, never stored in
   `.dms`.
2. When the transport is `smtp`, a review request uses the configured relay to
   send the notification. Successful relay acceptance is recorded in the
   workflow event chain; failure leaves the document in `draft` and offers a
   retry. Email contains only the relative path, action, confidentiality
   label, and local-app deep link (privacy rules).
3. The local-app deep link is a URI registered by the desktop application. It
   identifies the workspace, stable document ID, and review-request ID without
   embedding document content or an absolute filesystem path. On a host where
   the app is installed and the workspace is registered and accessible,
   activating the URI opens that document's review request and its decision UI.
   If no eligible local workspace is available, the app reports that condition
   and does not open an arbitrary path or record a workflow decision.
4. When the transport is `mailto`, the desktop app opens the host's default
   mail handler with a pre-filled `mailto:` URI including the same notification
   fields. The lifecycle state does not advance to `in_review` until the
   operator explicitly confirms in the app that the message was sent.
5. After an `approved`, `rejected`, or `changes_requested` decision is recorded,
   the app notifies the requester's snapshotted email address. The notification
   contains the relative path, decision outcome, confidentiality label, and a
   local-app deep link to the review detail; it does not include document
   content or the decision comment. Failure to send or confirm this outcome
   notification records a retryable delivery attempt and never reverses the
   recorded decision.
6. A workspace may switch transport at any time. Switching from `mailto` to
   `smtp` requires a relay configuration; switching from `smtp` to `mailto`
   clears the relay settings but keeps the approver roster.
7. The workflow history records each review-request and decision-outcome
   notification with its recipient, transport, delivery status, SMTP response
   code (or `mailto`-sent confirmation), and timestamp. An operator-visible
   report can filter by transport and delivery status.
8. If the host has no registered mail handler and the transport is `mailto`,
   the action surfaces a clear message naming the missing handler; the
   workflow does not silently fall back to SMTP.

## Non-goals

- Multi-recipient review requests in v1 (single configured approver per request)
- Calendar or meeting invitations
- Server-issued read receipts (the operator confirms `mailto` send manually)
- Embedding or attaching document content

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0010-notification-transport.html`](../wireframes/html/CAP-0010-notification-transport.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0010-notification-transport.png`](../wireframes/exports/CAP-0010-notification-transport.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0009, ADR-0012: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
