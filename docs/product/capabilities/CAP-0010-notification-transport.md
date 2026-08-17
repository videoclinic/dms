# CAP-0010 — Notification transport (SMTP or host mail handler)

| Field | Value |
| --- | --- |
| ID | CAP-0010 |
| Status | implemented |
| Transports | Configured SMTP relay (ADR-0009) and host default mail handler via `mailto:` (ADR-0012) |
| Tests | Phases 9i, 9k, and 9k.5 fake-backed evidence plus Phase 9l configured SMTP acceptance: [core lifecycle tests](../../../crates/dms-core/tests/lifecycle.rs), [desktop adapter commands](../../../crates/dms-desktop/src/lib.rs), [notification adapter tests](../../../crates/dms-desktop/src/notify.rs), [Library request tests](../../../crates/dms-desktop/ui/library.test.mjs), and [Configuration transport/test tests](../../../crates/dms-desktop/ui/configuration.test.mjs); configured Windows SMTP review, decision, and minor-publication delivery is recorded in [CHG-0001](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md); the HTML alternative part is proven by [core notification tests](../../../crates/dms-core/tests/lifecycle.rs) and [notify multipart tests](../../../crates/dms-desktop/src/notify.rs) under [CHG-0021](../../changes/archive/CHG-0021-html-notification-permalinks.md) |

## Outcomes

1. **Configuration → Notifications** stores the selected notification transport
   (`smtp` or `mailto`) and the non-secret SMTP relay settings. SMTP stores a
   login user used only for relay authentication and a separate RFC 5322 `From`
   mailbox that may include a display name. For SMTP it also
   accepts a write-only Microsoft 365 app-password field that writes directly to
   OS credential storage after relay validation; the value is never pre-filled,
   serialized, or returned. A blank field retains an existing credential, while
   SMTP cannot be saved or used without one. Switching to `mailto` deletes the
   workspace-scoped SMTP credential. The persistent
   Configuration navigation also exposes Workspace, Document defaults, and
   Workflow so notification settings remain a discoverable peer rather than an
   isolated page. The relay password is always resolved from the OS credential
   store, never stored in `.dms`. Once saved, the UI represents credential
   presence only as `***`; it never reconstructs or returns the password.
2. When the transport is `smtp`, a review request uses the configured relay to
   send the canonical review-request notification below. The SMTP message is
   `multipart/alternative`: the plain-text part carries the template below and
   the HTML alternative part carries the same visible copy with the
   `<review-permalink>` as a clickable link (see the HTML alternative part
   contract below). Successful relay acceptance is recorded in the workflow
   event chain; failure leaves the document in `draft` and offers a retry.
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
   pre-filled `mailto:` URI (the compose window carries the plain-text subject
   and body only; `mailto:` cannot carry HTML). The lifecycle state does not
   advance to `in_review` until the operator explicitly confirms in the app
   that the message was sent.
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
10. A configured SMTP transport with a stored credential exposes one deliberate
    test action whose label names the saved `From` mailbox. It sends a fixed
    message containing no document or workflow content to the parsed address of
    that mailbox. The action accepts no arbitrary recipient and returns only a
    sanitized success/failure plus an optional numeric relay response code.

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

## HTML alternative part (contract)

Every SMTP notification — review request, decision outcome, minor publication,
and periodic-review reminder — is sent as `multipart/alternative`. The first
part is the `text/plain; charset=utf-8` body of the kind's plain-text template
above and the second part is a `text/html; charset=utf-8` alternative that
mirrors the visible copy line for line and renders only the notification's
CAP-0020 permalink as a hyperlink (`<a href>` carrying the exact same URI).
The HTML part adds no other link, image, or document content; the plain-text
part stays the canonical contract and remains what the `mailto:` transport
prefills.

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
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
