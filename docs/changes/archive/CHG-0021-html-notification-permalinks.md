# CHG-0021 — Clickable permalinks in HTML notification emails

**Plan ID:** CHG-0021-html-notification-permalinks
**Created:** 2026-08-17
**Depends on:** none
**Context sources:** `crates/dms-core/src/lifecycle.rs` (`NotificationMessage`, `notification_message`, `review_request_message`, `decision_message`, `minor_publication_message`), `crates/dms-core/src/maintenance.rs` (`remind_periodic_review`), `crates/dms-desktop/src/notify.rs` (`DesktopNotifier` SMTP path), `crates/dms-desktop/src/lib.rs` (`test_smtp_notification_with`), `docs/product/capabilities/CAP-0010-notification-transport.md`
**Produces:** Every DMS notification email sent over SMTP carries an HTML alternative part in which the CAP-0020 permalink is a clickable link; the plain-text part, the `mailto:` draft, and the recorded evidence are unchanged.
**Status:** done — SMTP notifications are `multipart/alternative` (plain text + HTML with the clickable CAP-0020 permalink) across all four notification kinds; the `mailto:` transport and workflow evidence are unchanged; archived after workspace gate

| Field | Value |
| --- | --- |
| ID | CHG-0021 |
| Status | done |
| External request | Direct operator request: The send emails are as text so the "dms:" uri is not clickable for the user. Send also a HTML content so the user is able to click easily |
| Affected CAPs | CAP-0010 |
| Decision records | none |

## Current state

- `dms-core` builds each notification as plain text only: `NotificationMessage { kind, recipient, subject, body, mailto_uri }`, with the `dms://` permalink embedded as bare text in `body`.
- The desktop SMTP path (lettre) sends that text as a single `text/plain` body, so recipients cannot click the permalink.
- The `mailto:` transport opens a host mail compose window from a `mailto:` URI; `mailto:` cannot carry HTML.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Shared plain/HTML body builders + `html_body` field | done (`cargo test -p dms-core`: lifecycle suite 24 passed, HTML assertions for review request, minor publication, and periodic-review reminder) | `cargo test -p dms-core lifecycle` exits 0 with new HTML assertions |
| 2 | Desktop multipart/alternative SMTP send + SMTP test literal | done (`cargo test -p dms-desktop --lib notify`: 4 passed, including multipart/alternative ordering assertions) | `cargo test -p dms-desktop notify` exits 0 with multipart assertions |
| 3 | Workspace gate, CAP/ADR record closeout, archive | done (`cargo fmt --all -- --check`; `cargo test --workspace` 139 passed; `cargo clippy --workspace --all-targets -- -D warnings`; frontend 98/98; `git diff --check`; CAP-0010 + ADR-0012 amended; CHG archived as done) | Rust/frontend/link/diff checks exit 0; CHG archived as done |

## Phase 1 — Shared plain/HTML body builders

**Goal:** core owns one builder per notification kind that yields the canonical plain-text body and a minimal HTML body with the identical visible copy and the permalink as an `<a>` element.

Steps:

1. Add `html_body: String` to `NotificationMessage` (public field, updated at every construction site: `notification_message`, `test_smtp_notification_with`).
2. Replace the inline `format!` bodies with per-kind builders (`review_request_message`, `decision_message`, `minor_publication_message`, periodic-review reminder in `maintenance.rs`) that produce `(plain, html)` from the same field values.
3. HTML rules: `text/html; charset=utf-8`; `<pre>`-wrapped paragraphs for the label lines; the permalink line renders as `Open review task: <a href="PERMALINK">PERMALINK</a>`; all field values HTML-escaped.
4. Plain text stays byte-identical to today's contract.
5. Extend core tests: assert `html_body` contains the anchor with the full `dms://` URI as `href` and anchor text, plus the escaped label lines.

**Verification gate:** `cargo test -p dms-core lifecycle` exits 0.

## Phase 2 — Desktop multipart/alternative SMTP send

**Goal:** the SMTP transport sends `multipart/alternative` with `text/plain` first and `text/html` second so rich clients click the link while plain clients keep the bare URI.

Steps:

1. In `DesktopNotifier`'s SMTP branch, build the message with lettre's `multipart` module instead of `.body(text)`.
2. Update `FakeSmtpSender` observation coverage and assert the formatted message contains both alternative parts in order.
3. Update the SMTP configuration-test message construction in `lib.rs` (literal `html_body` mirroring the body text).
4. `mailto:` branch unchanged.

**Verification gate:** `cargo test -p dms-desktop notify` exits 0.

## Phase 3 — Workspace gate and record closeout

**Goal:** all checks pass; the CAP contract and ADR-0012 describe the dual-content email; the CHG becomes an archived receipt.

Steps:

1. Run the full verification gate.
2. Amend `CAP-0010` outcomes and notification contracts: SMTP notifications are dual-content; the HTML alternative carries the clickable permalink; `mailto:` remains plain text.
3. Amend the ADR-0012 consequence for the dual-content SMTP body.
4. Move this CHG to `docs/changes/archive/` and update `docs/changes/README.md`.

**Verification gate:** the full workspace gate command exits 0; CHG moves to `archive/`.

## Out of scope

- Changing the plain-text body, subject lines, or the `mailto:` URI shape.
- Adding recipients, attachments, or document content to notifications.
- Persisting the HTML body in workflow evidence (evidence continues to record `DeliveryAttempt` fields only).
