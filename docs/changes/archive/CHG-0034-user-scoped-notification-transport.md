# CHG-0034 — User-scoped notification transport

Move review and release email transport configuration from portable library
metadata to DMS's OS-user configuration. Each DMS user chooses their own
SMTP relay or host-mail fallback while workflow evidence continues to record
which transport delivered each attempt.

**Plan ID:** CHG-0034-user-scoped-notification-transport
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** Direct operator request defines the notification scope.
**Context sources:** `docs/product/capabilities/CAP-0010-notification-transport.md`; `docs/architecture.md`; `docs/privacy.md`; `docs/design-decisions.md` (ADR-0009, ADR-0012, ADR-0024); `crates/dms-core/src/lifecycle.rs`; `crates/dms-desktop/src/lib.rs`; `crates/dms-desktop/src/notify.rs`; `crates/dms-desktop/ui/configuration.mjs`.
**Produces:** Per-OS-user notification transport settings and SMTP credential, workspace-schema migration that removes relay metadata, lifecycle adapters that inject the current user setting, and CAP-0010 wireframe/test evidence.
**Status:** done — schema migration, OS-user configuration, lifecycle injection,
documentation, wireframe, and all workspace gates passed.

| Field | Value |
| --- | --- |
| ID | CHG-0034 |
| Status | done |
| External request | Direct operator request: "the configuration of the email relay is per user (using DMS) not per library" |
| Affected CAPs | CAP-0001, CAP-0002, CAP-0010, CAP-0017 |
| Decision records | ADR-0009, ADR-0012, ADR-0024 — notification transport scope changes from workspace to OS user. |

## Risk call-out

A relay configured by one user must never be copied with a library or make a
second user's workflow actions use the first user's credential. The migration
must preserve workflow evidence while removing old relay settings from
workspace metadata. The app password remains write-only and never crosses IPC,
preferences, or metadata.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Move the non-secret transport configuration and credential key to OS-user scope; migrate `.dms` metadata | done (`cargo test -p dms-core`; schema-v15 migration test) | Focused core and desktop Rust tests pass |
| 2 | Inject the current user setting into every notification-producing desktop lifecycle action | done (`cargo test -p dms-desktop --lib`) | Candidate, decision, release, retry, and periodic-reminder coverage passes |
| 3 | Update CAP/architecture/privacy/ADRs and CAP-0010 wireframe | done (`node docs/product/wireframes/generate.mjs`; CAP-0010 PNG inspected) | Generated CAP-0010 HTML/PNG reflects user scope |
| 4 | Run workspace gates and close the record | done (`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check`) | Format, lint, Rust/frontend tests, records and wireframe checks pass |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 1 — User-owned storage boundary

1. Store the selected transport and non-secret SMTP relay fields in
   `global-settings.json` under the OS-user app-config directory.
2. Store the app password in one OS credential-store entry per DMS OS user,
   rather than one entry per workspace ID.
3. Add a schema migration that removes `notification_settings` from `.dms`
   while retaining the standard source backup. Do not import an old workspace
   relay into a user profile implicitly.
4. Keep `dms-core` free of desktop user settings and credentials.

Verification gate: focused `dms-core` and `dms-desktop` tests exit 0.

## Phase 2 — Explicit lifecycle injection

1. Require desktop notification-producing actions to load the current OS-user
   setting and pass it to the core notification port.
2. Preserve delivery-attempt evidence, explicit `mailto:` confirmation, and the
   current failure semantics.
3. Make the SMTP test use the same user setting and credential as the runtime
   notifier.

Verification gate: candidate, decision, release, retry, and periodic-reminder
coverage exits 0.

## Phase 3 — Current behaviour evidence

1. Amend CAP-0010 and related architecture/privacy/decision records so they
   name OS-user scope and remove all workspace-scoped relay claims.
2. Update the CAP-0010 wireframe generator, regenerate HTML/index/manifest,
   and render its PNG with synthetic values.

Verification gate: `node docs/product/wireframes/generate.mjs` exits 0 and the
CAP-0010 review assets show the user-scoped configuration.

## Phase 4 — Verify and close

1. Run the workspace format, lint, Rust-test, frontend-test, and record checks.
2. Complete the DOX pass; archive this CHG and refresh the change index only
   after every gate passes.

## Out of scope

- A shared SMTP relay registry or administrator-managed mail service.
- Migrating a portable workspace's previous relay into any user's profile.
- Changing notification templates, deep-link identity, or delivery evidence.
- Persisting mail passwords or SMTP relay configuration in `.dms`.
