# CHG-0037 — Library switcher and lock-owner feedback

DMS Desktop will let an operator choose an existing or newly initialized library while another library is active, silently replace the prior library session only after the destination lock succeeds, and name the local OS user holding a destination library lock when that switch is blocked.

**Plan ID:** CHG-0037-library-switcher-and-lock-owner-feedback
**Execution slot:** P0600
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/architecture.md` (Runtime shape, Trust and control boundary); `docs/privacy.md` (Data classes, Processing principles); `docs/product/capabilities/CAP-0005-desktop-shell.md` (Outcomes 12, 17); `docs/product/capabilities/CAP-0014-workspace-integrity.md` (Outcomes 1–2); `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/integrity.rs` (`WorkspaceLock`, `WorkspaceLockStatus`, lock acquisition); `crates/dms-desktop/src/lib.rs` (`open_workspace`, `workspace_lock_status`, `acquire_workspace_lock`); `crates/dms-desktop/ui/app.mjs` (`setupMarkup`, `switchWorkspaceSession`, `activateWorkspace`); `crates/dms-desktop/ui/app.test.mjs`; `docs/product/wireframes/generate.mjs`
**Produces:** An active DMS Desktop session has an explicit library switcher with the same existing/open and initialize options as startup. A successful different-library selection acquires the destination lock, releases the former lock, clears the former session activities without confirmation, and opens the destination Library. A blocked target remains inactive and identifies its recorded local OS lock owner and host without releasing the current library.
**Status:** done

| Field | Value |
| --- | --- |
| ID | CHG-0037 |
| Status | done |
| External request | Direct operator request: "Allow the user to switch between multiple known libraries. If a different library is selected, the previous one is \"closed\" silently. Add also the ability to open a new library like with the starting screen so the user do not have to close DMS in order to open a different library. If a library is opened by a different user (blocked) show the user who keeps the library open" |
| Affected CAPs | CAP-0005, CAP-0014 |
| Decision records | ADR-0001 and ADR-0014 remain applicable; no new ADR is required because this reuses the existing single-active-workspace and advisory-lock contracts. |

## Current state

- Startup has the required existing-workspace form, initialize form, native directory pickers, and a per-user list of at most ten recent edit roots. An active session exposes **Open library…**, which presents those same controls without unlocking or replacing the current library until a different destination lock succeeds.
- A successful switch already acquires the destination lock before releasing the old owner-matched lock, rolls back the newly acquired lock if old-lock release fails, then reconstructs frontend session state and opens Library (`crates/dms-desktop/ui/app.mjs`). Opening the same workspace focuses its singleton Library activity and performs no lock handoff.
- A destination that refuses because a current or stale advisory lock exists queries read-only `workspace_lock_status` and names the recorded OS user, hostname, lock state, and acquired time. A status-query failure keeps the original refusal without an owner claim. The process ID is not shown.
- CAP-0005, CAP-0014, `docs/privacy.md`, and `crates/dms-desktop/AGENTS.md` describe active-session **Open library…**, silent prior-session close, and bounded OS-user/host lock feedback. The CAP-0005 wireframe shows the switcher and a current-lock refusal.

## Risk call-out

A switch must never release the active library lock or discard its activities until the destination lock acquisition has succeeded. A blocked, malformed, inaccessible, or stale-target failure leaves the current library active and unchanged. If the destination status read after a refusal is unavailable or races to a different state, show the truthful generic refusal rather than inventing an owner.

The owner label is advisory local-process evidence, not an Entra identity, access-control decision, or proof that the named user is currently editing. Show the recorded OS user and hostname needed to identify the holder; do not expose the process ID in the switcher or add lock metadata to preferences, workflow evidence, or outbound notifications. Stale takeover and override-any-lock retain their existing explicit warnings and confirmations.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Implement active-session library switching and blocked-owner feedback | done (`node --test crates/dms-desktop/ui/app.test.mjs` — 50 passed) | `node --test crates/dms-desktop/ui/app.test.mjs` exits 0 with active-to-different switching, same-library reuse, blocked current/stale owner display, status-query failure, and destination/old-lock rollback coverage. |
| 2 | Publish shell and advisory-lock contracts, wireframe, and full verification | done (`node docs/product/wireframes/generate.mjs`; CAP-0005 PNG 249199 bytes; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs` — 127 passed) | `node docs/product/wireframes/generate.mjs`, the CAP-0005 PNG render command, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP/CHG indexes and DOX contracts agree. |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Implement active-session library switching and blocked-owner feedback

**Goal:** An operator can use the startup-equivalent picker while a library remains active; only a successfully locked different destination replaces that session, and an advisory-lock refusal names its recorded holder.

Steps:

1. Extract the reusable existing/open and initialize-library controls from `setupMarkup` into a switcher surface that can be explicitly opened from active shell chrome. Reuse the current recent-library list, `Browse…` controls, validation, stale-takeover, and override-any-lock choices; include both opening existing metadata and confirmed dual-root initialization so the active-session path matches startup.
2. Add an accessible **Open library…** control that opens the switcher without closing, unlocking, or replacing the current library. Dismissing the switcher restores the unchanged active activity. Opening the same workspace focuses its singleton Library activity and performs no lock handoff or activity reset.
3. Route both switcher forms through the current `open_workspace` / `initialize_workspace` and `activateWorkspace` boundary. For a different destination, preserve the existing acquire-destination → release-owner-matched-prior → reset-session sequence; after success, discard prior activities silently, retain recent-library preferences, and open the destination root Library. Do not create simultaneous workspace sessions, a close confirmation, or a saved multi-library tab list.
4. When destination acquisition rejects because a lock exists, obtain the target's read-only `workspace_lock_status` and retain it as switcher-local feedback. For a current or stale status with a lock, state the recorded OS user and hostname plus the lock state/acquired time; keep the old workspace, old lock, activities, and preferences intact. If that status query cannot produce a current lock snapshot, preserve the original refusal without an owner claim. Do not display the process ID.
5. Add focused frontend tests for opening/dismissing the active switcher, recent/manual existing opens, confirmed initialization, same-workspace reuse, successful different-workspace handoff/reset, current and stale lock-owner feedback, lock-status-query failure, stale takeover/override forwarding, and release-failure rollback. Keep existing startup and permalink paths unchanged.

Verification gate: `node --test crates/dms-desktop/ui/app.test.mjs` exits 0 with active-to-different switching, same-library reuse, blocked current/stale owner display, status-query failure, and destination/old-lock rollback coverage.

## Phase 2 — Publish shell and advisory-lock contracts, wireframe, and full verification

**Goal:** Current-state product records and the CAP-0005 review screen explain switching, silent prior-session closure, and bounded owner feedback, with full workspace evidence proving the implementation.

Steps:

1. Amend CAP-0005 to make the active-session **Open library…** switcher explicit: it presents startup-equivalent existing/open and initialize controls plus recent libraries, preserves the source session until a different destination lock succeeds, silently ends that former session after success, and retains only one active workspace/activity set. Keep startup recent-library behaviour intact.
2. Amend CAP-0014 to state that a blocked open identifies the recorded advisory lock's OS user and hostname, distinguishes current from stale, and retains explicit takeover/override semantics. Amend `docs/privacy.md` to classify that visible local owner feedback accurately; it is local process metadata, not Entra identity or authority.
3. Update `crates/dms-desktop/AGENTS.md` with the durable switch transaction and owner-feedback contract. Leave parent AGENTS files unchanged unless the DOX pass finds a parent ownership/index change.
4. Update CAP-0005's definition in `docs/product/wireframes/generate.mjs` with synthetic active-library chrome and an **Open library…** switcher state showing recent/manual/initialize options plus a current-lock owner refusal. Regenerate HTML, `index.html`, `manifest.json`, and the CAP-0005 PNG; visually inspect the result. Do not add a new CAP screen.
5. Run the full workspace checks. Record passing evidence, mark phases done, move this CHG to `docs/changes/archive/`, and update `docs/changes/README.md` only when implementation is complete; do not alter unrelated active records.

Verification gate: `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0005-desktop-shell.png "file://$PWD/html/CAP-0005-desktop-shell.html" && test -s exports/CAP-0005-desktop-shell.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree.

## Out of scope

- Multiple simultaneously active libraries, multiple windows, cross-library activity tabs, or restoring a closed library's activities after a switch.
- Replacing the advisory lock with shared editing, network coordination, filesystem ACL enforcement, or Entra/Graph authorization.
- Automatically taking over a stale/current lock, exposing the lock process ID, or notifying the named lock owner.
- Persisting a chosen active library, switcher form state, lock-owner feedback, or closed activities beyond existing recent-library preferences.
