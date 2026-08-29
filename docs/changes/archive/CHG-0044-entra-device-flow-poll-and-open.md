# CHG-0044 — Entra device-flow poll, reissue, and Open library

Microsoft Entra device-authorization surfaces poll at the provider interval, expose host-mediated Open sign-in page, continue the task when sign-in succeeds, and offer Reissue code after expiry, decline, or failure. After a successful identity-source apply from a blocked library, the operator opens that library with an explicit Open control.

**Plan ID:** CHG-0044-entra-device-flow-poll-and-open
**Created:** 2026-08-29
**Depends on:** CHG-0043
**Entry checkpoint:** none
**Produces:** Identity-source and approver device-flow match startup/library-session poll and reissue. Blocked-library reapply ends with Open {library}, not an automatic open.
**Status:** done — Phase 3 published the shared device-flow rule, Open library control, and workspace gate.

| Field | Value |
| --- | --- |
| ID | CHG-0044 |
| Status | done |
| External request | Direct operator request: (1) Add polling for the identity source; also a reissue of the login should be possible; this is a general design for the whole application not only for this screen (2) The message is "DMS will then try to open this library." What have to user now to do? Would it not be better to offer also a button: "Open <what ever>" to continue? |
| Affected CAPs | CAP-0005, CAP-0021 |
| Decision records | none |

## Current state

- Startup, library-session, identity-source, and approver device-flow poll at the provider interval, expose **Open sign-in page**, continue without a complete-click, and offer **Reissue code** after expiry, decline, or failure.
- After identity-source apply from blocked-library recovery, the shell offers **Open {library}** and does not open automatically.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Identity-source poll/reissue and Open library | done (`node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/library.test.mjs`; `cargo test -p dms-desktop --lib`) | `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/library.test.mjs` and `cargo test -p dms-desktop --lib` exit 0 |
| 2 | Approver device-flow poll/reissue | done (`node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs`; `cargo test -p dms-desktop --lib`) | `node --test crates/dms-desktop/ui/library.test.mjs crates/dms-desktop/ui/app.test.mjs` and `cargo test -p dms-desktop --lib` exit 0 |
| 3 | CAP, wireframe, workspace gate | done (`node generate.mjs`; CAP-0021 PNG; `cargo fmt --all -- --check`; clippy; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check`) | generate.mjs, CAP-0021 PNG, `cargo fmt --all -- --check`, clippy, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, `git diff --check` |

## Phase 1 — Identity-source poll/reissue and Open library

Poll identity-source device-flow at the provider interval; on success show the preview without a manual signed-in control; on expiry/decline/failure offer Reissue code. After apply from a blocked library, show **Open {library}**.

## Phase 2 — Approver device-flow poll/reissue

Approver sign-in uses the same poll, Open sign-in page, success-without-complete-click, and Reissue code pattern.

## Phase 3 — CAP, wireframe, workspace gate

CAP-0005 and CAP-0021 state the shared device-flow rule and the Open library control. Wireframe CAP-0021 shows identity-source sign-in and applied Open states.
