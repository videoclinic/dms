# CHG-0043 — Blocked-library recovery navigation

When DMS Desktop cannot open a selected library, the blocking surface names that destination, explains the cause in operator language, and always offers an explicit return to Set up workspace. When the cause is an unverified Microsoft Entra identity source, the same surface explains that reapply means signing in, previewing the library group, and applying it so DMS records the verified tenant, then offers that recovery without activating the library.

**Plan ID:** CHG-0043-blocked-library-recovery-navigation
**Execution slot:** P0800
**Created:** 2026-08-29
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/design-decisions.md` (ADR-0031); `docs/product/capabilities/CAP-0005-desktop-shell.md`; `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md`; `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-desktop/src/graph.rs` (`bound_tenant_id`, `verify_group_bound_actor`); `crates/dms-desktop/src/lib.rs` (`begin_library_session_authorization`, `apply_identity_source`); `crates/dms-desktop/ui/app.mjs` (`librarySessionAuthorizationMarkup`, `render`); `crates/dms-desktop/ui/configuration.mjs` (`identitySourceMarkup`)
**Produces:** A blocked library-open surface that always returns to Set up workspace, and an unverified-identity-source recovery that reapplies the binding against the selected edit root without acquiring the library lock or opening Library/Configuration destinations.
**Status:** done — Phase 3 published the navigation rule, recovery surface, and blocked-open screen.

| Field | Value |
| --- | --- |
| ID | CHG-0043 |
| Status | done |
| External request | Direct operator request: If the "Cannot open edit" while opening a existing DMS library appears, the user do not get any navigation support, how to go back and open or fix the issue. We need a general rule for navigation. Explain more detailed what "reapply it before signing in" does it mean because the existing DMS library can not be opened |
| Affected CAPs | CAP-0005, CAP-0021 |
| Decision records | ADR-0031 remains applicable. No new ADR. |

## Current state

- Opening a group-bound library with a schema-v17 group-only binding fails closed: `tenant_id` is null, `evaluate_library_session_credential` returns unavailable, and the shell replaces Set up workspace with a card that has no return or repair action (`crates/dms-desktop/ui/app.mjs` `librarySessionAuthorizationMarkup`; `crates/dms-desktop/src/graph.rs` `bound_tenant_id`).
- "Reapply it before signing in" names Configuration → Workflow → Manage identity source, which is unavailable until a workspace is active. Identity-source sign-in and apply already use the app-global tenant and an edit-root path without a group-bound session (`begin_identity_source_sign_in`, `apply_identity_source`).
- Pending library-session challenges offer Open sign-in page; expired/declined/failed offer Reissue code. Unavailable offers neither a back path nor recovery.

## Risk call-out

Recovery must not activate Library, acquire the advisory lock, or insert a group-bound session. Identity-source apply already mutates `.dms` without a lock; do not add lock/session machinery for this path. After a successful apply, retry the normal library-session gate. Do not infer a tenant from OS-user configuration, the people cache, or historic evidence.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Blocking-surface navigation and unverified-source recovery | done (`node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs`; `cargo test -p dms-desktop --lib`) | `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs` and `cargo test -p dms-desktop --lib` exit 0, proving return navigation, unverified copy, recovery without workspace activation, and unchanged pending/reissue behaviour |
| 2 | Publish CAP contracts and the blocked-open wireframe | done (`node docs/product/wireframes/generate.mjs`; CAP-0021 PNG render) | `node docs/product/wireframes/generate.mjs` and the CAP-0021 PNG render exit 0; CAP-0005 and CAP-0021 describe the navigation rule and reapply recovery |
| 3 | Workspace gate and close the change record | done (`cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check`) | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; CAP/CHG indexes agree and CHG-0043 is archived |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Blocking-surface navigation and unverified-source recovery

**Goal:** Every library-session block returns to Set up workspace. An unverified identity source explains reapply and offers that recovery against the selected edit root without opening the library.

Steps:

1. Give every blocking library-session kind an explicit **Choose another library** control that dismisses the overlay, restores Set up workspace (recent libraries and the open/initialize forms), and keeps the failed edit root in the open form.
2. Replace the unverified-source one-liner with operator copy: the library already has a group binding, DMS has not recorded a verified tenant, sign-in cannot start, and reapply means sign in / preview group / apply so DMS stores that tenant. Expose a structured `reapply_identity_source` recovery on the IPC status.
3. **Reapply identity source** loads workspace configuration for that edit root without activating the library and renders the identity-source surface with the existing group ID prefilled. Apply uses the existing identity-source IPC. Success retries library-session activation. Failure stays on the recovery surface.
4. Do not set `appState.workspace` from a configuration snapshot during recovery. Destinations stay unavailable until the normal activation gate succeeds.

Verification gate: `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs` and `cargo test -p dms-desktop --lib` exit 0.

## Phase 2 — Publish CAP contracts and the blocked-open wireframe

**Goal:** CAP-0005 states the blocked-destination navigation rule. CAP-0021 states unverified-source recovery and matching wireframe evidence.

Steps:

1. Add CAP-0005 outcome: a destination that cannot be entered keeps a named return to the previous recoverable surface and, when remediable without activating that destination, a named action to the remediating surface.
2. Add CAP-0021 operational detail for unverified tenant bindings and the recovery that reapplies the identity source without opening the library.
3. Update the CAP-0021 wireframe blocked states with **Choose another library** and the unverified **Reapply identity source** explanation. Regenerate HTML and the CAP-0021 PNG.

Verification gate: `node docs/product/wireframes/generate.mjs` and the CAP-0021 PNG render exit 0.

## Phase 3 — Workspace gate and close the change record

**Goal:** Workspace checks pass and the change is archived.

Verification gate: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and `git diff --check` exit 0; move CHG-0043 to `archive/` and refresh the changes index.
