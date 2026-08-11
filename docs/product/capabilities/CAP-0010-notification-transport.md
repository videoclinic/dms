# CAP-0010 — Notification transport (SMTP or host mail handler)

| Field | Value |
| --- | --- |
| ID | CAP-0010 |
| Status | not implemented |
| Transports | Configured SMTP relay (ADR-0009) and host default mail handler via `mailto:` (ADR-0012) |
| Tests | Partial configuration and Phase 9k fake-backed lifecycle evidence: [core lifecycle tests](../../../crates/dms-core/tests/lifecycle.rs), [desktop adapter commands](../../../crates/dms-desktop/src/lib.rs), [notification adapter tests](../../../crates/dms-desktop/src/notify.rs), and [Library request tests](../../../crates/dms-desktop/ui/library.test.mjs); configured external delivery remains Phase 9l work |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. **Configuration → Notifications** stores the selected notification transport
   (`smtp` or `mailto`) and the non-secret SMTP relay settings. The persistent
   Configuration navigation also exposes Workspace, Document defaults, and
   Workflow so notification settings remain a discoverable peer rather than an
   isolated page. The relay password is always resolved from the OS credential
   store, never stored in `.dms`.
2. When the transport is `smtp`, a review request uses the configured relay to
   send the canonical review-request notification below. Successful relay
   acceptance is recorded in the workflow event chain; failure leaves the
   document in `draft` and offers a retry.
3. The local-app deep link is a CAP-0020 permalink URI. It identifies the
   workspace and stable document ID, plus the review-request target ID, without
   embedding document content, draft file name, version label, or an absolute
   filesystem path. On a host where the app is installed and the workspace is
   registered and accessible, activating the URI opens that document's review
   request and its decision UI (and focuses the matching activity tab). If no
   eligible local workspace is available, the app reports that condition and
   does not open an arbitrary path or record a workflow decision.
4. When the transport is `mailto`, the desktop app opens the host's default
   mail handler with the canonical review-request subject and body below in a
   pre-filled `mailto:` URI. The lifecycle state does not advance to `in_review`
   until the operator explicitly confirms in the app that the message was sent.
5. After an `approved`, `rejected`, or `changed_requested` decision is recorded,
   the app notifies the requester's snapshotted email address. The notification
   contains the relative path, decision outcome, confidentiality label, and a
   local-app deep link to the review detail; it does not include document
   content or the decision comment. Failure to send or confirm this outcome
   notification records a retryable delivery attempt and never reverses the
   recorded decision.
6. After a successful minor-version release, the app notifies the effective
   approver snapshotted for that release that their assigned document has a new
   minor publication. The notification is sent only after atomic export commits
   the release; SMTP failure or unconfirmed `mailto:` send records a retryable
   delivery attempt and never reverses the committed release. Minor releases do
   not send a review request before release.
7. A workspace may switch transport at any time. Switching from `mailto` to
   `smtp` requires a relay configuration; switching from `smtp` to `mailto`
   clears the relay settings but keeps Microsoft Entra workflow-role bindings.
8. The workflow history records each review-request, decision-outcome, and
   minor-publication notification with its recipient, transport, delivery status,
   SMTP response code (or `mailto`-sent confirmation), and timestamp. An
   operator-visible report can filter by transport and delivery status.
9. If the host has no registered mail handler and the transport is `mailto`,
   the action surfaces a clear message naming the missing handler; the
   workflow does not silently fall back to SMTP.

## Review-request notification (contract)

Every review-request notification uses this exact UTF-8 plain-text template for
the SMTP message and the pre-filled `mailto:` draft. The subject and body have
the following literal labels and field order:

```text
Subject: [<confidentiality-label>] DMS review requested — <document-title> — <target-version>

A review decision is requested.

Action: Review and decide
Title: <document-title>
Document: <edit-root-relative-source-path>
Requested by: <requester-display-name>
Target version: <target-version>
Confidentiality: <confidentiality-label>

Open review task:
<review-permalink>
```

- `<document-title>` is the DMS-managed `title` control field, not the source
  file name, Office document properties, or Markdown front matter.
- `<edit-root-relative-source-path>` is the current filesystem-derived source
  path relative to the edit root.
- `<requester-display-name>` is the display name snapshotted with the review
  request; the requester’s email address is not included in the message body.
- `<target-version>` is the candidate version snapshotted with the review
  request (for example, `V1.4`). It remains review evidence and is not a
  reserved or released version.
- `<review-permalink>` is the CAP-0020 review-target URI. It is the only
  document link in the notification; the email contains no source-file URL,
  released-PDF URL, public web URL, attachment, document content, or decision
  control.

## Minor-publication notification (contract)

Every successful minor-version release sends this UTF-8 plain-text template to
the effective approver snapshotted with that release. It has no review action or
decision control:

```text
Subject: [<confidentiality-label>] DMS minor version released — <document-title> — <released-version>

A new minor version of your assigned document has been released.

Title: <document-title>
Document: <edit-root-relative-source-path>
Released by: <requester-display-name>
Released version: <released-version>
Confidentiality: <confidentiality-label>

Open document:
<document-permalink>
```

- `<released-version>` is the committed minor release label, not an uncommitted
  candidate.
- `<document-permalink>` is the CAP-0020 document URI without a review target.
- The notification contains no document content, source/PDF URL, attachment, or
  approval control.

## Non-goals

- Multi-recipient review requests in v1 (single configured approver per request)
- Calendar or meeting invitations
- Server-issued read receipts (the operator confirms `mailto` send manually)
- Embedding or attaching document content

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0010-notification-transport.html`](../wireframes/html/CAP-0010-notification-transport.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0010-notification-transport.png`](../wireframes/exports/CAP-0010-notification-transport.png)

- Lifecycle: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Permalinks: [`CAP-0020-document-permalinks.md`](CAP-0020-document-permalinks.md)
- Workflow-role routing: [`CAP-0019-inherited-workflow-role-routing.md`](CAP-0019-inherited-workflow-role-routing.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0009, ADR-0012, ADR-0020: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
