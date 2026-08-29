# CHG-0035 — Entra session required for a group-bound library

A DMS Desktop user activating a library with a Microsoft Entra ID group binding will complete a valid delegated Entra session that resolves `/me` and confirms the signed-in enabled user is a direct member of that library's bound group; all new DMS mutations and workflow evidence for that session identify that Entra tenant/object ID rather than the local OS user.

**Plan ID:** CHG-0035-entra-session-required-for-group-bound-library
**Execution slot:** P0400
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/architecture.md` (Runtime shape, Trust and control boundary); `docs/privacy.md` (Data classes, Processing principles); `docs/design-decisions.md` (ADR-0013, ADR-0021, ADR-0024, ADR-0028, ADR-0029); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcomes 1, 3); `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md` (Operational details, Outcomes 5, 6, 8, 9); `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/lib.rs` (`default_author`, `Workspace::update_control`); `crates/dms-core/src/lifecycle.rs` (`AuthenticatedActor`, `WorkflowEventBody`, `GraphClient`, event appenders); `crates/dms-core/src/library.rs`; `crates/dms-core/src/maintenance.rs`; `crates/dms-core/src/audit.rs`; `crates/dms-desktop/src/graph.rs` (`MicrosoftGraphClient::authenticated_actor`, `direct_user_members_with_token`, token/device-flow primitives); `crates/dms-desktop/src/lib.rs` (`open_workspace`, `WorkspaceSummary`, `update_document_control_with`, startup authorization); `crates/dms-desktop/ui/app.mjs` (`switchWorkspaceSession`, startup-authorization card); `crates/dms-desktop/ui/configuration.mjs`; `crates/dms-desktop/ui/library.mjs`; `docs/product/wireframes/AGENTS.md`; `docs/product/wireframes/generate.mjs`
**Produces:** A group-bound desktop workspace cannot activate until its signed-in Entra actor is freshly confirmed as an enabled direct member of its bound group. New metadata, note, lifecycle, report, and workflow evidence records that actor's tenant/object ID without a local-OS-user principal; unbound libraries retain the existing local-operator behavior.
**Status:** done — Phase 5 published the current-state contracts and session-required screen.

| Field | Value |
| --- | --- |
| ID | CHG-0035 |
| Status | done |
| External request | Direct operator request: "If an library is connected to a Entry ID Microsoft 365 Group, the user using DMS need a valid Entra ID login/session in order to be identified as the Entra ID user (not the local user)" |
| Affected CAPs | CAP-0011, CAP-0021 |
| Decision records | Add ADR-0031. ADR-0013, ADR-0021, ADR-0024, ADR-0028, and ADR-0029 remain applicable. |

## Current state

- Phase 1 is committed at `a1a7a75`. Schema v18 persists a bound tenant ID; a v17 group-only binding migrates as unverified and fails closed. `dms-core` requires `MutationPrincipal` on event-producing mutations; bound records serialize `authenticated_actor` and omit `local_os_user`. ADR-0031 is current (`docs/design-decisions.md` heading `ADR-0031`).
- Phase 2 caches a process-only `group_bound_sessions` map keyed by canonical edit root. Bound `open_workspace` revalidates tenant, `/me`, and fresh enabled direct membership before returning a summary; `switchWorkspaceSession` re-enters that gate before `acquire_workspace_lock`. Failed validation does not insert a session and does not acquire a destination lock. `configure_global_entra` and a successful `apply_identity_source` clear the cache.
- Phase 3 routes bound desktop mutations through that cached actor. Missing, replaced, and tenant-mismatched sessions reject before metadata changes. Bound `WorkspaceSummary.change_author` presents the cached display snapshot plus tenant/object ID. Review decisions keep the one-time interactive approver actor. The CLI remains local-principal-only and fails closed through `dms-core`. `crates/dms-desktop/AGENTS.md` records the session-before-lock and mutation-routing contract.
- Phase 4 adds one process-only per-library device-flow challenge. Missing or refresh-rejected credentials block the selected library with a sanitized code, expiry, host-mediated sign-in page, and terminal-only reissue control. Successful polling revalidates the actor/member and atomically acquires the destination lock before the shell activates; the direct lock IPC rejects group-bound libraries.
- CAP-0011 and CAP-0021 describe the verified active principal and the group-bound activation boundary. Architecture, privacy, and the CAP-0021 wireframe show the process-only session, blocking challenge, and fail-closed outcomes.

## Risk call-out

This changes who may activate and mutate a bound library. A cached display name, a local OS username, or a cached group membership must never substitute for the live tenant/object-ID and enabled-direct-member checks. Start no browser automatically, expose no token or `device_code` across IPC, and do not persist the active actor in `.dms`; the OS credential store remains the token location.

The event body is hash-chained evidence. Changing `local_os_user` from required to optional must preserve verification of historical event JSON and hashes exactly. The recovery path for a failed migration or verification regression is the existing versioned workspace backup; do not rewrite, backfill, or rehash historical events. A failed, expired, tenant-mismatched, inaccessible-group, disabled-account, or non-member session must leave the target workspace inactive and must release any newly acquired advisory lock.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Define the group-bound session and evidence principal | done (`cargo fmt --all -- --check`; `cargo test -p dms-core`; `cargo test -p dms-desktop --lib group_bound_session`; `cargo check --workspace`; `git diff --check`) | `cargo test -p dms-core --test lifecycle --test workspace` and `cargo test -p dms-desktop --lib group_bound_session` exit 0, proving legacy hash verification plus actor/tenant/member validation cases |
| 2 | Cache a verified actor before bound-library activation | done (`cargo test -p dms-desktop --lib group_bound_session`; `cargo check -p dms-desktop`) | `cargo test -p dms-desktop --lib group_bound_session` and `cargo check -p dms-desktop` exit 0, proving unbound activation preservation plus cached-token member, non-member, disabled, tenant-mismatch, and unavailable-session outcomes |
| 3 | Route bound adapter mutations through the cached actor | done (`cargo test -p dms-desktop --lib`) | `cargo test -p dms-desktop --lib` exits 0, proving every principal-aware command records the cached Entra actor and rejects a missing, replaced, or tenant-mismatched session before mutation |
| 4 | Add the library-session device-flow challenge and blocking shell | done (`cargo fmt --all -- --check`; `cargo test -p dms-desktop`; `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/library.test.mjs`; `git diff --check`) | `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/library.test.mjs` exit 0, including pending, expiry, reissue, sign-in, switch, no-session, non-member, and direct-IPC cases |
| 5 | Publish current-state contracts and the session-required screen | done (`node docs/product/wireframes/generate.mjs`; CAP-0021 PNG render; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check`) | `node docs/product/wireframes/generate.mjs`, the CAP-0021 PNG render command, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP/ADR/CHG indexes agree |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Define the group-bound session and evidence principal

**Goal:** The core and desktop adapter share one explicit actor contract: a bound-library mutation receives a matching authenticated Entra actor, while a library without an identity-source binding retains the local-operator contract.

Steps:

1. Add ADR-0031 to `docs/design-decisions.md`. It must fix these boundaries: an `EntraIdentitySource` makes desktop activation session-required; valid means an effective configured tenant, a cached or refreshed delegated credential, `/me` resolving to that tenant/object ID, and a fresh Graph direct-member response containing that enabled actor; a bound group may be either a security group or a Microsoft 365 group. Session identity is authorization truth only for the active desktop session and is never persisted in `.dms`.
2. Extend the persisted identity source with the bound tenant ID. Schema v18 migrates an existing group-only binding to an explicitly unverified binding that fails closed until an operator explicitly reapplies the source; it must not infer a tenant from mutable OS-user configuration, the display cache, or historical evidence. New/replaced bindings persist the effective tenant ID.
3. Introduce a Tauri-independent mutation-principal input in `dms-core` rather than importing Graph or a desktop session into the core. It must distinguish local OS actor from authenticated Entra actor, require the latter when `Workspace::identity_source()` exists, reject an Entra actor whose tenant differs from the binding, and reject an unverified legacy binding. Preserve the unbound local actor behavior.
4. Update the canonical event-body contract so new group-bound records carry `authenticated_actor` and omit `local_os_user`; new unbound records retain `local_os_user`. Deserialize and reserialize historical records without changing their canonical JSON or hashes. Do not invent Entra identities for historic local events, and do not rewrite existing event chains.
5. Thread the explicit principal through every core mutation that currently reaches `default_author()`: document-control updates, note writes, candidate/review/release and local lifecycle operations, periodic-review events, source reassociation, and report generation. Audit public core mutation entry points so a future caller cannot silently select `default_author()` for a bound workspace.
6. Add core tests for unbound local events, bound Entra events, missing/mismatched actor rejection without mutation, legacy-binding rejection, and successful verification of pre-change hash fixtures. Add fake-backed Graph/desktop tests for a valid cached token, refresh success, missing/expired credential, tenant mismatch, disabled account, and signed-in non-member.

Implementation finding: schema v17's group-only identity binding cannot validate the required tenant/object-ID pair. Tenant binding and its fail-closed legacy migration are required in this phase because every group-bound mutation depends on that comparison.

Implementation audit finding: lifecycle and periodic-review entry points still
constructed event bodies with `default_author()` after the first principal-aware
paths landed. This is current-phase work, not a later adapter concern: their
public core APIs must require and serialize the explicit principal so the CLI
cannot bypass a bound-library session before Phase 2 routes desktop commands.
This remains one atomic Phase 1 change despite its cross-module signature
updates: split entry points would leave a compiled mutation bypass or evidence
with an incorrect principal.

Verification gate: `cargo test -p dms-core --test lifecycle --test workspace` and `cargo test -p dms-desktop --lib group_bound_session` exit 0, including unchanged historic-hash verification and all actor/tenant/member validation cases.

## Phase 2 — Cache a verified actor before bound-library activation

**Goal:** Opening, recent-library switching, permalink resolution, and refresh preserve unbound behaviour but validate a bound library's effective tenant, cached or refreshed delegated credential, `/me`, and fresh enabled direct-member response before its advisory lock is acquired or its shell activates.

Steps:

1. Keep the uncommitted process-only `group_bound_sessions` map keyed by canonical edit root. A successful validation stores only `binding_id` and `AuthenticatedActor` (tenant/object ID live on the actor). Revalidate on every bound `open_workspace`; do not add a token-generation counter in this phase.
2. Finish the cached-token gate in `activate_group_bound_session` using `authenticated_actor`, `direct_user_members`, and `verify_group_bound_actor`. Missing, expired, refresh-rejected, tenant-mismatched, inaccessible-group, disabled-account, and non-member outcomes must not insert a session and must not acquire a destination lock.
3. Call that gate before `acquire_workspace_lock` on explicit open, recent-library open, refresh, and permalink activation. `switchWorkspaceSession` must not lock a bound library from a pre-resolved summary that skipped `open_workspace`. On a failed bound open, release any newly acquired destination lock.
4. Clear `group_bound_sessions` in `configure_global_entra` and after a successful `apply_identity_source`. A people display cache is never authorization truth.
5. Repair the Windows shortcut-refresh unit test so it no longer calls `open_workspace` without adapter state. Add focused tests for unbound preservation, valid member cache, tenant mismatch, disabled account, non-member, unavailable Graph, and cache-clear on tenant reconfiguration and identity-source replacement.

Verification gate: `cargo test -p dms-desktop --lib group_bound_session` and `cargo check -p dms-desktop` exit 0.

## Phase 3 — Route bound adapter mutations through the cached actor

**Goal:** Every bound workspace-scoped desktop mutation obtains the current verified session principal or fails before changing metadata; every new event records that actor and no local OS user.

Steps:

1. Centralize principal lookup on the canonical workspace/binding session and pass it to reassociation, document control, notes, candidate/review/release, local lifecycle, periodic-review, report, maintenance, configuration, notification-confirmation, and source-open mutation boundaries.
2. Preserve read-only queries and the separate one-time approver-decision actor. The CLI remains local-principal-only and fails closed through `dms-core`.
3. Replace bound `WorkspaceSummary.change_author` presentation with the cached display snapshot plus immutable tenant/object ID; preserve historical local-user evidence.
4. Test missing, replaced, and tenant-mismatched sessions reject before mutation; test every covered bound event serializes `authenticated_actor` and omits `local_os_user`.

Verification gate: `cargo test -p dms-desktop --lib` exits 0.

## Phase 4 — Add the library-session device-flow challenge and blocking shell

**Goal:** A missing or refresh-rejected bound-library credential produces an explicit per-library device-flow challenge, never an automatic browser launch or a startup-authorization fallback.

Steps:

1. Add a distinct library-session device-login purpose and challenge/status/reissue IPC. Expose user code, expiry, verification URI, and sanitized terminal states only; never expose tokens or `device_code`.
2. Add a blocking shell state naming the selected library/group with **Open sign-in page** and terminal-only **Reissue code**. Poll only at the provider interval and do not load workspace destinations while pending or failed.
3. Revalidate and activate exactly once after successful sign-in; safely release/acquire locks across recent-library switches and permalink activation. Route bound lock acquisition through that activation boundary so direct IPC cannot acquire a bound-library advisory lock without a verified session.
4. Add Rust and frontend coverage for pending, expiry, reissue, valid member, non-member, disabled, mismatch, unavailable service, direct IPC bypass, and actor presentation.

Verification gate: `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/library.test.mjs` exit 0.

Implementation finding: `acquire_workspace_lock` directly opened and locked any workspace, while `switchWorkspaceSession` performed the Entra validation only in a preceding frontend call. A direct IPC caller could therefore acquire a bound-library advisory lock without the activation gate. This is current-phase work because the device-flow completion must own the one verified activation and lock acquisition; the public lock command must reject bound workspaces outside that boundary.

## Phase 5 — Publish current-state contracts and the session-required screen

**Goal:** The product records and CAP-0021 wireframe explain that a bound library requires a verified Entra session and that new events use the verified Entra actor, not a local OS principal.

Steps:

1. Amend CAP-0021 with the workspace-activation session requirement, exact validity checks, direct-enabled-member condition, session invalidation conditions, process-session-only storage, and explicit failure/reissue state. Preserve the distinction between workflow identity and filesystem/SharePoint/OneDrive access control.
2. Amend CAP-0011 so current canonical events use exactly one principal for new records: local OS user for unbound libraries or authenticated Entra tenant/object identity for group-bound libraries. State that historic evidence remains unchanged and review decisions still require their snapshotted approver.
3. Update `docs/architecture.md` and `docs/privacy.md` for the new active-session boundary and Entra actor processing. State that group membership is freshly queried for activation, display cache is non-authoritative, tokens remain in the OS credential store, and `.dms` does not retain an active user session or backfilled identities.
4. Update CAP-0021's existing screen definition in `docs/product/wireframes/generate.mjs` with synthetic normal, sign-in-required, pending, non-member, and unavailable states for activating a group-bound library. Regenerate the HTML/index/manifest, render the matching existing CAP-0021 PNG, and visually inspect it. Do not add a pending-only screen to the product manifest.
5. Run the full workspace gates. After every gate passes, record evidence, move this CHG to `docs/changes/archive/`, and update `docs/changes/README.md` from Active to Archive without changing the unrelated active records.

Verification gate: `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0021-microsoft-entra-workflow-identity.png "file://$PWD/html/CAP-0021-microsoft-entra-workflow-identity.html" && test -s exports/CAP-0021-microsoft-entra-workflow-identity.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP/ADR/CHG indexes agree.

Implementation finding: the Phase 3 cached-session parameter made two desktop mutation helpers exceed the workspace clippy argument limit. Phase 5 must package their adapter-only dependencies before closeout so the required full clippy gate proves this CHG rather than carrying a known lint failure into the archive.

Implementation finding: the full workspace gate exposed a stale CLI integration invocation that omitted the tenant required by the Phase 1 identity-source contract. Update that fixture in this phase so the executable workspace proof exercises the current fail-closed CLI boundary instead of an obsolete command shape.

## Out of scope

- Changing filesystem, SharePoint, OneDrive, Microsoft 365 group, or Entra tenant access control; the session proves the DMS actor only.
- A browser approval portal, web SSO redirect flow, application-managed user roster, group/membership management, or background directory synchronization.
- Persisting the active Entra actor, a token, a refresh token, `device_code`, or a local-OS-to-Entra identity mapping in `.dms`.
- Backfilling, rewriting, or rehashing historical workflow events that contain local OS-user evidence.
- Adding Entra authentication to the headless CLI; it must fail closed for group-bound mutations rather than bypass the desktop session requirement.
