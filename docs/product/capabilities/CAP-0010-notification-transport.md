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
3. When the transport is `mailto`, the desktop app opens the host's default
   mail handler with a pre-filled `mailto:` URI including the same notification
   fields. The lifecycle state does not advance to `in_review` until the
   operator explicitly confirms in the app that the message was sent.
4. A workspace may switch transport at any time. Switching from `mailto` to
   `smtp` requires a relay configuration; switching from `smtp` to `mailto`
   clears the relay settings but keeps the approver roster.
5. The workflow history records, for each review request, which transport was
   used, the SMTP response code (or `mailto`-sent confirmation), and the
   timestamp. An operator-visible report can filter by transport.
6. If the host has no registered mail handler and the transport is `mailto`,
   the action surfaces a clear message naming the missing handler; the
   workflow does not silently fall back to SMTP.

## Non-goals

- Multi-recipient review requests in v1 (single configured approver per request)
- Calendar or meeting invitations
- Server-issued read receipts (the operator confirms `mailto` send manually)
- Embedding or attaching document content

## Links

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0009, ADR-0012: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
