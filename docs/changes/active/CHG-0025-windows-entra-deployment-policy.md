# CHG-0025 — Windows Entra deployment policy and startup authorization

Produce Windows deployment support that lets an administrator configure the DMS Desktop Microsoft Entra public-client ID and tenant ID either manually or through a machine-scoped ADMX policy, and make a process configured with both `DMS_ENTRA_CLIENT_ID` and `DMS_ENTRA_TENANT_ID` validate its delegated sign-in at launch or begin and poll a new device-authorization code without blocking the UI.

**Plan ID:** CHG-0025-windows-entra-deployment-policy
**Execution slot:** P0100
**Created:** 2026-08-19
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `docs/entra-client-setup.md` (Configure DMS, Environment-managed configuration); `docs/architecture.md` (Runtime shape, Trust and control boundary); `docs/privacy.md` (Data classes); `docs/design-decisions.md` (ADR-0021, ADR-0024); `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md` (Operational details, Outcomes); `crates/dms-desktop/src/lib.rs` (`DesktopIntegrations`, `GlobalSettings`, `effective_global_entra_configuration*`, `runtime_entra_configuration`, `run`); `crates/dms-desktop/src/graph.rs` (`begin_delegated_sign_in`, `wait_for_device_token`, `TokenStore`); `crates/dms-desktop/ui/configuration.mjs` (Application Entra configuration); `crates/dms-desktop/ui/app.mjs` (`handleSubmit`); `crates/dms-desktop/ui/configuration.test.mjs`; `.github/workflows/desktop-platform-smoke.yml`; `.github/workflows/release-windows.yml`
**Produces:** A released DMS Desktop build resolves a valid computer policy from `HKLM\SOFTWARE\Policies\Videoclinic\DMS` before environment or saved configuration, exposes policy ownership read-only in Configuration, and, only when both process `DMS_ENTRA_*` values are present, validates the tenant's cached delegated token or starts one non-blocking device-authorization challenge with polling and explicit reissue. It also ships a validated `DMSDesktop.admx` plus `en-US\DMSDesktop.adml` and operator documentation for manual, GPO, and Intune deployment.
**Status:** in-progress — Phase 1 committed as `c2f4915`; Phase 2 is pending.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0100` is the execution order authority for this CHG and no conflicting active execution slot exists.

| Field | Value |
| --- | --- |
| ID | CHG-0025 |
| Status | in-progress — Phase 1 committed as `c2f4915`; Phase 2 is pending |
| External request | Direct operator request: when both `DMS_ENTRA_CLIENT_ID` and `DMS_ENTRA_TENANT_ID` are set, DMS Desktop must automatically validate the current Entra sign-in or begin and poll a device-authorization code, then let the user explicitly reissue a code after failure or expiry. |
| Affected CAPs | CAP-0021 |
| Decision records | New ADR for Windows machine-policy precedence and startup device authorization; ADR-0021 and ADR-0024 remain applicable |

## Current state

- DMS resolves a Windows machine-policy pair through a target-specific `winreg` dependency before process environment or OS-user configuration. There is still no ADMX, ADML, GPO/Intune deployment artifact, or deployment runbook.
- `crates/dms-desktop/src/lib.rs` rejects a partial or invalid machine-policy pair before fallback and returns explicit saved, environment, or Windows-policy ownership for each effective identifier. It still has no startup authorization status.
- `crates/dms-desktop/src/graph.rs` already obtains a device code and preserves the service-provided poll interval, but `wait_for_device_token` is a synchronous loop that starts only after a Configuration user submits **I have signed in — preview group**. It is not an application-lifetime poller and has no startup path.
- `run` constructs `MicrosoftGraphClient` from the effective configuration before the UI opens, but it performs no token-cache validity check and does not start a device-authorization challenge. The Configuration UI renders the code only in the identity-source setup flow, and its existing **Sign in again** button starts a new group-preview challenge after a failed manual flow.
- CAP-0021, ADR-0028, `docs/architecture.md`, and `docs/privacy.md` place the client/tenant pair outside workspace metadata and record Windows-policy precedence. The per-library Entra group binding, first editor/approver selection, and device authorization remain explicit interactive operations.
- `docs/entra-client-setup.md` documents manual Configuration entry and an environment-managed launch but does not distinguish developer/process overrides from managed Windows deployment or automatic device authorization.
- The Windows smoke job already runs the complete Rust test suite and NSIS packaging. The tag release workflow currently publishes only the installer, checksum, and stable-release winget bundle.
- Microsoft documents custom ADMX/ADML import for Intune as public preview; it accepts one `en-US` ADML per template. Standard Group Policy uses language-neutral ADMX plus language-specific ADML from a Central Store. Sources: <https://learn.microsoft.com/en-us/intune/device-configuration/settings-catalog/import-custom-admx-templates> and <https://learn.microsoft.com/en-us/troubleshoot/windows-client/group-policy/create-and-manage-central-store>.

## Risk call-out

A device policy can force DMS to use the wrong tenant and block every Graph operation across all Windows users. The policy must therefore be an all-or-nothing pair: if either registry value exists, both values must exist and be UUIDs; an incomplete or invalid policy fails closed and never falls back to environment or saved values. The policy controls only two public identifiers. It must not deploy an Entra app registration, store a client secret/token, set the library group binding, select workflow people, or bypass device authorization and explicit library application.

The requested "device registration" is OAuth 2.0 device authorization, not Microsoft Entra device join/registration. Starting an interactive code whenever a process has both environment values is intentional, but DMS must not open a browser without a user action, spin in a blocking loop, discard a previously cached credential on a transient network failure, or silently reissue codes. At most one pending code exists for the configured tenant; the app displays it in a persistent shell status, polls at the provider interval, and offers **Reissue code** only after expiry, decline, or an explicit user request.

Recovery is to set the policy **Not Configured** (or remove both values), refresh policy, and restart DMS. This leaves the saved OS-user configuration and workspace `.dms` metadata untouched, restoring the existing environment-then-saved resolution path. The runbook must include this recovery and `reg.exe query` evidence before a broad rollout.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Define and implement the machine-policy configuration source | done (`cargo test -p dms-desktop --lib entra_policy`; `cargo clippy -p dms-desktop --all-targets -- -D warnings`; Configuration UI test) | `cargo test -p dms-desktop --lib entra_policy` exits 0; `cargo clippy -p dms-desktop --all-targets -- -D warnings` exits 0 |
| 2 | Implement automatic process-environment device authorization | pending — Phase 1 checkpoint `c2f4915` complete | `cargo test -p dms-desktop --lib startup_device_authorization` exits 0; `node --test crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/app.test.mjs` exits 0 |
| 3 | Ship ADMX assets and manual/GPO/Intune deployment documentation | pending | `python3 scripts/validate_admx.py docs/deployment/windows/admx/DMSDesktop.admx docs/deployment/windows/admx/en-US/DMSDesktop.adml` exits 0; every relative link in `docs/windows-entra-deployment.md` resolves |
| 4 | Validate the Windows deployment path and close records | pending | Windows evidence shows the configured process presents or validates exactly one device-authorization state, `reg.exe query HKLM\SOFTWARE\Policies\Videoclinic\DMS` returns the two expected UUID values, and `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and the Windows `Desktop platform smoke` job exit/pass |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Define and implement the machine-policy configuration source

**Goal:** DMS Desktop resolves a valid device policy before process environment and OS-user settings, visibly identifies policy-managed fields, and cannot silently use a partial policy.

Steps:

1. Add an ADR after ADR-0027 that fixes the policy contract: Windows only; `HKLM\SOFTWARE\Policies\Videoclinic\DMS`; string values `EntraClientId` and `EntraTenantId`; and precedence `machine policy → DMS_ENTRA_* process override → global-settings.json`. Explain why this is a computer policy: the Entra app/tenant is organization identity shared by all users of a managed device, unlike local settings and library metadata.
2. Amend the Architecture, Privacy, and CAP-0021 contracts in the same slice. State that policy values are non-secret, are read-only in Configuration, are not copied into `.dms` or `global-settings.json`, and do not configure the library group or delegated credential cache.
3. Add a small desktop policy module instead of scattering Windows registry calls through `lib.rs`. Its Windows implementation reads only the two named values from the exact `HKLM\SOFTWARE\Policies\Videoclinic\DMS` key; non-Windows builds return no policy. Use a target-specific Windows registry dependency so Linux/macOS builds do not acquire a Windows runtime dependency.
4. Represent the result as an explicit source/ownership state, not two new booleans that can drift. Merge that state into `effective_global_entra_configuration` before `runtime_entra_configuration`; if one policy value exists, require and validate both UUIDs before any fallback. Preserve the existing environment and saved-settings behavior only when the policy key has neither value.
5. Extend the Configuration IPC shape and `ui/configuration.mjs` so policy-managed identifiers are read-only and labelled **Managed by Windows policy**. Keep the existing environment-managed label truthful. Saving globally configured IDs must not overwrite a policy value or clear a policy-managed state.
6. Add focused library tests for: absent policy preserves current behavior; valid policy wins over environment and saved values; incomplete policy fails closed; invalid policy UUID fails closed; policy values never persist to `global-settings.json`; and the UI renders the policy source. Keep policy reader tests injectable so Linux CI covers precedence without a live Windows registry.

Verification gate: `cargo test -p dms-desktop --lib entra_policy` exits 0; `cargo clippy -p dms-desktop --all-targets -- -D warnings` exits 0.

## Phase 2 — Implement automatic process-environment device authorization

**Entry condition:** Phase 1 checkpoint `c2f4915` exists; push it too when an operator requests a remote checkpoint.

**Goal:** When—and only when—both `DMS_ENTRA_CLIENT_ID` and `DMS_ENTRA_TENANT_ID` are the effective process-environment source rather than superseded by machine policy, desktop startup validates the tenant's cached delegated credential or starts one visible device-authorization challenge that the UI polls without blocking.

Steps:

1. Add an ADR after ADR-0027 that distinguishes OAuth device authorization from Entra device registration and fixes the startup trigger: both effective process-environment values must be non-empty and valid; an ADMX-controlled (including one with conflicting environment values) or saved-settings-only configuration does not initiate interactive authorization at launch. A valid cached access token, or an expired token successfully refreshed with its OS-credential-store refresh token, is **valid**. A missing cache, malformed cache, or refresh rejection starts a challenge; a credential-store or transient network failure reports an error and does not reissue or erase a code/token.
2. Refactor `graph.rs` so a pending login can be polled once. Preserve the provider interval and `slow_down` adjustment, but replace the synchronous `wait_for_device_token` loop with a result model that returns `pending(next_poll_after)`, `authorized`, `declined`, `expired`, or a terminal error after exactly one token request. Keep existing identity-source and approver flows working through the same primitive.
3. Extend `DesktopIntegrations` with app-global startup authorization state, not workspace or `.dms` state. In `run`, after resolving the effective runtime configuration, detect that both identifiers are environment-managed rather than policy-managed, validate/load/refresh the tenant credential, and create at most one pending challenge. Register narrow IPC commands to read status, poll a pending challenge, and explicitly reissue it. The commands must never return the `device_code`, access token, or refresh token to JavaScript.
4. Add an app-shell authorization status card available before a workspace is opened and while any route is active. It presents the user code, expiry, and **Open sign-in page** button; schedules the next single poll from the server-advertised interval; marks a valid cached session without a code; and replaces expired/declined/terminal states with an explicit **Reissue code** action. Do not auto-open the external browser, do not poll after a terminal result, and cancel UI timers on rerender/unload so one challenge never has two polls.
5. Add graph tests with a fake HTTP client for valid cached access token, refresh success, refresh rejection, missing cache, `authorization_pending`, `slow_down`, authorization success, expiry, decline, and duplicate reissue prevention. Add `configuration.test.mjs` and `app.test.mjs` coverage for app-shell visibility, interval scheduling semantics, terminal state, and explicit reissue; retain the manual group-preview tests unchanged.

Verification gate: `cargo test -p dms-desktop --lib startup_device_authorization` exits 0; `node --test crates/dms-desktop/ui/configuration.test.mjs crates/dms-desktop/ui/app.test.mjs` exits 0.

## Phase 3 — Ship ADMX assets and manual/GPO/Intune deployment documentation

**Goal:** An administrator can obtain one versioned ADMX package and follow a precise supported path for local manual setup, Active Directory Group Policy, or Intune without mistaking policy configuration for Entra app-registration deployment.

Steps:

1. Create `docs/deployment/AGENTS.md` for the durable deployment-asset boundary and add it to `docs/AGENTS.md`'s Child DOX Index. It owns deployment runbooks and policy assets; its verification names the ADMX validator and link checks.
2. Add `docs/deployment/windows/admx/DMSDesktop.admx` and `docs/deployment/windows/admx/en-US/DMSDesktop.adml`. Define one Computer Configuration policy under **Videoclinic/DMS Desktop** with two required text inputs, writing only `EntraClientId` and `EntraTenantId` under the ADR-fixed HKLM policy key. Use an independent vendor namespace and no dependency on Windows ADMX. Do not add `de-DE`: Intune custom-template import permits a single `en-US` ADML.
3. Add `scripts/validate_admx.py`, using only Python's standard library. It must parse both XML documents; prove the ADMX target namespace, Computer policy class, exact registry key/value names, and required ADML string references; and reject an unpaired or unused policy element. Add a lightweight test/CI invocation so a malformed asset cannot ship unnoticed.
4. Add `docs/windows-entra-deployment.md` as the operator guide. It must separate:
   - Entra control-plane prerequisites: create the public client, enable device flow, assign only the existing delegated Graph permissions and consent. Policy does not create this registration.
   - Manual local setup: use the existing Configuration flow, then configure and preview the library-specific group and roles. Describe `DMS_ENTRA_*` only as a deliberate process-launch override, not as a persistent enterprise deployment mechanism; when both values are the effective source, DMS starts its non-blocking device-authorization status at launch rather than creating an Entra device object.
   - Domain GPO: copy the ADMX and `en-US` ADML to the Central Store, enable the Computer Configuration policy, enter both UUIDs, assign/filter the GPO, run `gpupdate /force`, query the exact registry key, and restart DMS.
   - Intune: import the custom ADMX and `en-US` ADML, create an **Imported Administrative templates (Preview)** Windows 10 and later profile, enable the DMS policy, assign it to the intended device group, monitor policy delivery, query the registry, and restart DMS. State the documented preview/one-language limitation and link Microsoft guidance.
   - Rollback/troubleshooting: policy Not Configured removes the enforced pair; incomplete/malformed policy blocks Graph intentionally; policy does not replace user device authorization or workspace identity-source application.
5. Update `docs/entra-client-setup.md`, README deployment links, and `docs/AGENTS.md` ownership so the existing setup guide links to the deployment guide rather than duplicating policy instructions.
6. Package the exact ADMX tree as `dms-desktop-admx.zip` in `release-windows.yml` and attach it to stable and prerelease GitHub Releases beside the installer. Do not put it in the NSIS installer: Central Store/Intune administrators consume the package out-of-band, and installing templates on every endpoint is not required for the registry policy to apply.

Verification gate: `python3 scripts/validate_admx.py docs/deployment/windows/admx/DMSDesktop.admx docs/deployment/windows/admx/en-US/DMSDesktop.adml` exits 0; every relative link in `docs/windows-entra-deployment.md` resolves.

## Phase 4 — Validate the Windows deployment path and close records

**Goal:** One Windows device proves that the deployed policy controls the DMS Entra configuration, while a process with both DMS environment values validates or completes one device-authorization flow without exposing secrets, mutating library metadata, or disabling the normal identity-source safeguards.

Steps:

1. On a real Windows 10/11 managed test device, install the signed NSIS build, apply the GPO path or assigned Intune profile, and record non-secret evidence: `gpresult /r` or Intune policy state; `reg.exe query HKLM\SOFTWARE\Policies\Videoclinic\DMS`; and the app Configuration state showing both identifiers are policy-managed.
2. Launch DMS with both valid `DMS_ENTRA_*` process values, no machine policy, and no delegated credential. Confirm it displays one new device-authorization code without automatically opening a browser, polls after the displayed/provider interval, stores authorization after a completed sign-in, and next launch reports a valid cached/refreshable session without a new code. Exercise expired and declined codes, confirm polling stops, and confirm only the user-triggered **Reissue code** starts a replacement.
3. Verify device policy wins over intentionally conflicting `DMS_ENTRA_*` process values and pre-existing `global-settings.json` values. Verify an incomplete policy blocks Graph with the documented error, then remove it through Not Configured and confirm saved/environment resolution returns after restart. Verify policy-only and policy-overridden environment configurations do not start a device-authorization challenge; the automatic trigger remains the effective process-environment pair.
4. Complete normal DMS identity-source setup: use delegated device authorization, preview the configured group, and explicitly apply the library binding. Confirm that policy deployment and startup credential validation did not create or modify the group, `.dms` client/tenant fields, or the delegated token cache beyond a completed device-authorization token.
5. Run the repository gate: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs`. Confirm the Windows entry in `Desktop platform smoke` passes and a tagged release contains `dms-desktop-admx.zip` with the expected three files.
6. Update CAP-0021, Architecture, Privacy, the deployment guide, and this CHG with actual Windows evidence. Mark all phases `done (<evidence>)`, move the CHG to `docs/changes/archive/`, and refresh `docs/changes/README.md` in the same change.

Verification gate: Windows evidence shows the configured process presents or validates exactly one device-authorization state, `reg.exe query HKLM\SOFTWARE\Policies\Videoclinic\DMS` returns the two expected UUID values, DMS renders both fields policy-managed, and `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, and the Windows `Desktop platform smoke` job exit/pass.

## Out of scope

- Provisioning, modifying, or deleting the Microsoft Entra application registration, Graph permissions, admin consent, groups, memberships, or workspace role assignments through ADMX, Intune, or DMS.
- Storing a client secret, certificate, OAuth token, user identity, group ID, edit root, publish root, or SMTP credentials in the policy registry.
- Microsoft Entra device join/registration, device-object lifecycle, compliance, or Conditional Access management. This CHG implements only OAuth device authorization for the desktop user's delegated Graph session.
- Replacing the signed NSIS installer with Intune Win32 app packaging. The policy configures an installed client; application deployment is a separate endpoint-management concern.
- User-scoped `HKCU` policy, macOS profiles, Linux configuration management, or Windows Home local-policy workarounds.

## Risks & open questions

- Intune custom ADMX import is documented as public preview and supports only one `en-US` ADML. The runbook must make Group Policy the stable enterprise-template path and mark Intune import as preview rather than promising equivalent lifecycle guarantees.
- Phase 4 needs a Windows managed test device or tenant. GitHub Actions can prove compile/tests and package contents but cannot prove a real GPO/Intune assignment or interactive device authorization.
