# CHG-0035 — Entra session required for a group-bound library

A DMS Desktop user activating a library with a Microsoft Entra ID group binding will complete a valid delegated Entra session that resolves `/me` and confirms the signed-in enabled user is a direct member of that library's bound group; all new DMS mutations and workflow evidence for that session identify that Entra tenant/object ID rather than the local OS user.

**Plan ID:** CHG-0035-entra-session-required-for-group-bound-library
**Execution slot:** P0400
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/architecture.md` (Runtime shape, Trust and control boundary); `docs/privacy.md` (Data classes, Processing principles); `docs/design-decisions.md` (ADR-0013, ADR-0021, ADR-0024, ADR-0028, ADR-0029); `docs/product/capabilities/CAP-0011-approval-evidence.md` (Outcomes 1, 3); `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md` (Operational details, Outcomes 5, 6, 8, 9); `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/lib.rs` (`default_author`, `Workspace::update_control`); `crates/dms-core/src/lifecycle.rs` (`AuthenticatedActor`, `WorkflowEventBody`, `GraphClient`, event appenders); `crates/dms-core/src/library.rs`; `crates/dms-core/src/maintenance.rs`; `crates/dms-core/src/audit.rs`; `crates/dms-desktop/src/graph.rs` (`MicrosoftGraphClient::authenticated_actor`, `direct_user_members_with_token`, token/device-flow primitives); `crates/dms-desktop/src/lib.rs` (`open_workspace`, `WorkspaceSummary`, `update_document_control_with`, startup authorization); `crates/dms-desktop/ui/app.mjs` (`switchWorkspaceSession`, startup-authorization card); `crates/dms-desktop/ui/configuration.mjs`; `crates/dms-desktop/ui/library.mjs`; `docs/product/wireframes/AGENTS.md`; `docs/product/wireframes/generate.mjs`
**Produces:** A group-bound desktop workspace cannot activate until its signed-in Entra actor is freshly confirmed as an enabled direct member of its bound group. New metadata, note, lifecycle, report, and workflow evidence records that actor's tenant/object ID without a local-OS-user principal; unbound libraries retain the existing local-operator behavior.
**Status:** pending — queued after P0300; no predecessor evidence is required.

| Field | Value |
| --- | --- |
| ID | CHG-0035 |
| Status | pending |
| External request | Direct operator request: "If an library is connected to a Entry ID Microsoft 365 Group, the user using DMS need a valid Entra ID login/session in order to be identified as the Entra ID user (not the local user)" |
| Affected CAPs | CAP-0011, CAP-0021 |
| Decision records | Add ADR-0031. ADR-0013, ADR-0021, ADR-0024, ADR-0028, and ADR-0029 remain applicable. |

## Current state

- An Entra identity source already binds one tenant/group pair to a workspace, but the group is described as the source of workflow people and review-decision verification rather than as the identity gate for every desktop user (`docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md:114-155`; `docs/design-decisions.md:370-411`).
- `open_workspace` returns a summary directly from the local workspace; frontend workspace switching acquires only the advisory lock before activation. Neither path establishes a group-bound actor (`crates/dms-desktop/src/lib.rs:410-412`; `crates/dms-desktop/ui/app.mjs:685-724`).
- `MicrosoftGraphClient::authenticated_actor` obtains a token from the OS credential store and resolves `/me`; its direct-member query filters the bound group to enabled user accounts. Those primitives can validate an actor, but only existing lifecycle paths compose them (`crates/dms-desktop/src/graph.rs:879-997`).
- Process-environment startup authorization intentionally does not start for saved settings or Windows policy, and its `valid` result proves only a tenant credential rather than membership in a selected library group (`docs/design-decisions.md:556-582`; `crates/dms-desktop/src/lib.rs:2436-2460`).
- The canonical event body always serializes `local_os_user`; most mutations call `default_author()` from `USER`/`USERNAME`, while an Entra actor is currently attached only when a workflow path has explicitly authenticated one (`crates/dms-core/src/lib.rs:1210-1216`; `crates/dms-core/src/lifecycle.rs:350-388`; `crates/dms-core/src/lifecycle.rs:2028-2066`). Notes, source reassociation, periodic-review events, and audit-report events have the same local-author default (`crates/dms-core/src/library.rs:347-393`; `crates/dms-core/src/maintenance.rs:647-691`; `crates/dms-core/src/audit.rs:172-174`).
- The headless CLI deliberately has no Entra flow. It must not remain a mutation bypass for a group-bound library (`docs/architecture.md:15-17`; `crates/AGENTS.md:20-24`).

## Risk call-out

This changes who may activate and mutate a bound library. A cached display name, a local OS username, or a cached group membership must never substitute for the live tenant/object-ID and enabled-direct-member checks. Start no browser automatically, expose no token or `device_code` across IPC, and do not persist the active actor in `.dms`; the OS credential store remains the token location.

The event body is hash-chained evidence. Changing `local_os_user` from required to optional must preserve verification of historical event JSON and hashes exactly. The recovery path for a failed migration or verification regression is the existing versioned workspace backup; do not rewrite, backfill, or rehash historical events. A failed, expired, tenant-mismatched, inaccessible-group, disabled-account, or non-member session must leave the target workspace inactive and must release any newly acquired advisory lock.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Define the group-bound session and evidence principal | pending | `cargo test -p dms-core --test lifecycle --test workspace` and `cargo test -p dms-desktop --lib group_bound_session` exit 0, proving legacy hash verification plus actor/tenant/member validation cases |
| 2 | Gate desktop activation and route every bound mutation through the Entra actor | pending | `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/library.test.mjs` exit 0, including no-session, non-member, sign-in, switch, and event-principal cases |
| 3 | Publish current-state contracts and the session-required screen | pending | `node docs/product/wireframes/generate.mjs`, the CAP-0021 PNG render command, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP/ADR/CHG indexes agree |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Define the group-bound session and evidence principal

**Goal:** The core and desktop adapter share one explicit actor contract: a bound-library mutation receives a matching authenticated Entra actor, while a library without an identity-source binding retains the local-operator contract.

Steps:

1. Add ADR-0031 to `docs/design-decisions.md`. It must fix these boundaries: an `EntraIdentitySource` makes desktop activation session-required; valid means an effective configured tenant, a cached or refreshed delegated credential, `/me` resolving to that tenant/object ID, and a fresh Graph direct-member response containing that enabled actor; a bound group may be either a security group or a Microsoft 365 group. Session identity is authorization truth only for the active desktop session and is never persisted in `.dms`.
2. Introduce a Tauri-independent mutation-principal input in `dms-core` rather than importing Graph or a desktop session into the core. It must distinguish local OS actor from authenticated Entra actor, require the latter when `Workspace::identity_source()` exists, and reject an Entra actor whose tenant differs from the binding. Preserve the unbound local actor behavior.
3. Update the canonical event-body contract so new group-bound records carry `authenticated_actor` and omit `local_os_user`; new unbound records retain `local_os_user`. Deserialize and reserialize historical records without changing their canonical JSON or hashes. Do not invent Entra identities for historic local events, and do not rewrite existing event chains.
4. Thread the explicit principal through every core mutation that currently reaches `default_author()`: document-control updates, note writes, candidate/review/release and local lifecycle operations, periodic-review events, source reassociation, and report generation. Audit public core mutation entry points so a future caller cannot silently select `default_author()` for a bound workspace.
5. Add core tests for unbound local events, bound Entra events, missing/mismatched actor rejection without mutation, and successful verification of pre-change hash fixtures. Add fake-backed Graph/desktop tests for a valid cached token, refresh success, missing/expired credential, tenant mismatch, disabled account, and signed-in non-member.

Verification gate: `cargo test -p dms-core --test lifecycle --test workspace` and `cargo test -p dms-desktop --lib group_bound_session` exit 0, including unchanged historic-hash verification and all actor/tenant/member validation cases.

## Phase 2 — Gate desktop activation and route every bound mutation through the Entra actor

**Goal:** DMS Desktop exposes no active group-bound library session until the user signs in and is verified as an enabled direct member, and all adapter mutations use that verified actor instead of the local OS username.

Steps:

1. Add a desktop-only group-bound session state keyed to the active workspace/binding. On an open, recent-library switch, or permalink resolution, inspect only enough workspace metadata to determine whether a binding exists. If unbound, retain the existing lock and activation behavior. If bound, validate the effective configuration, token, `/me`, and fresh direct-member list before acquiring/finalizing the workspace lock or loading Library/Configuration data.
2. Reuse the existing device-flow primitives for a missing or refresh-rejected credential, but create a library-session challenge rather than widening the environment-only startup trigger. The blocking shell state must name the selected library/group, show the user code, expiry, and explicit **Open sign-in page** action, poll only at the provider interval, and offer **Reissue code** only after a terminal result. It must not auto-open a browser, leak `device_code`/tokens, or fall back to the startup card's tenant-only `valid` state.
3. Keep the verified actor only in desktop process session state. Revalidate on every bound-library activation and after a binding replacement; invalidate it when the token, effective tenant configuration, or binding changes. A successful membership refresh may update the existing display cache, but the cache itself must not authorize the session.
4. Centralize the adapter's workspace-opening/mutation boundary so every workspace-scoped IPC command either receives the current verified group-bound principal or fails before opening/mutating the workspace. Cover Library, Configuration, Notes, Maintenance, audit/report, source-open/reassociate, lifecycle, notification confirmation, and permalink paths; do not rely on hiding frontend controls. The CLI supplies only a local principal and therefore rejects mutations against a group-bound library rather than bypassing the requirement.
5. Replace `WorkspaceSummary.change_author` and any affected selection/history presentation with the active actor's Entra display snapshot plus immutable tenant/object identity when the workspace is bound. Do not expose tokens or local usernames as the bound-library acting principal. Preserve historical local-user evidence and the existing review-decision actor presentation.
6. Add focused Rust and frontend tests: unbound activation remains unchanged; a bound library stays inactive while sign-in is pending, expired, unavailable, mismatched, disabled, or non-member; valid membership activates exactly once; switching releases/acquires locks safely; every mutation logs the Entra actor and no local principal; and direct IPC calls cannot bypass the gate.

Verification gate: `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/library.test.mjs` exit 0, including no-session, non-member, sign-in, switch, and event-principal cases.

## Phase 3 — Publish current-state contracts and the session-required screen

**Goal:** The product records and CAP-0021 wireframe explain that a bound library requires a verified Entra session and that new events use the verified Entra actor, not a local OS principal.

Steps:

1. Amend CAP-0021 with the workspace-activation session requirement, exact validity checks, direct-enabled-member condition, session invalidation conditions, process-session-only storage, and explicit failure/reissue state. Preserve the distinction between workflow identity and filesystem/SharePoint/OneDrive access control.
2. Amend CAP-0011 so current canonical events use exactly one principal for new records: local OS user for unbound libraries or authenticated Entra tenant/object identity for group-bound libraries. State that historic evidence remains unchanged and review decisions still require their snapshotted approver.
3. Update `docs/architecture.md` and `docs/privacy.md` for the new active-session boundary and Entra actor processing. State that group membership is freshly queried for activation, display cache is non-authoritative, tokens remain in the OS credential store, and `.dms` does not retain an active user session or backfilled identities.
4. Update CAP-0021's existing screen definition in `docs/product/wireframes/generate.mjs` with synthetic normal, sign-in-required, pending, non-member, and unavailable states for activating a group-bound library. Regenerate the HTML/index/manifest, render the matching existing CAP-0021 PNG, and visually inspect it. Do not add a pending-only screen to the product manifest.
5. Run the full workspace gates. After every gate passes, record evidence, move this CHG to `docs/changes/archive/`, and update `docs/changes/README.md` from Active to Archive without changing the unrelated active records.

Verification gate: `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0021-microsoft-entra-workflow-identity.png "file://$PWD/html/CAP-0021-microsoft-entra-workflow-identity.html" && test -s exports/CAP-0021-microsoft-entra-workflow-identity.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP/ADR/CHG indexes agree.

## Out of scope

- Changing filesystem, SharePoint, OneDrive, Microsoft 365 group, or Entra tenant access control; the session proves the DMS actor only.
- A browser approval portal, web SSO redirect flow, application-managed user roster, group/membership management, or background directory synchronization.
- Persisting the active Entra actor, a token, a refresh token, `device_code`, or a local-OS-to-Entra identity mapping in `.dms`.
- Backfilling, rewriting, or rehashing historical workflow events that contain local OS-user evidence.
- Adding Entra authentication to the headless CLI; it must fail closed for group-bound mutations rather than bypass the desktop session requirement.
