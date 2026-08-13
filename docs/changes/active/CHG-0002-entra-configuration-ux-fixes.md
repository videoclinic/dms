# CHG-0002 — Entra configuration UX fixes

| Field | Value |
| --- | --- |
| ID | CHG-0002 |
| Status | in-progress |
| External request | Direct operator request: (1) Clicking "Save application configuration" the view is left and started with the Library view. I would expect to see a confirmation that the settings are set. (2) Clicking the "Sign-in page" link/url does not open the standard browser -- my assumption is that Tauri is a "browser" itself the "Sign-in page" is not opned. (3) Tried to use the device id and failed; I'm missing a regeneration of a new device code because the "previous" one is not accepted by Entra ID anymore. (4) Windows Credential Manager rejects a delegated Entra token when its UTF-16 password representation exceeds 2,560 characters. (5) Microsoft Graph returns a group user object ID without a display name. |
| Affected CAPs | CAP-0021 |
| Decision records | (none — UX corrections to phase 9k.1, no cross-cutting fork) |

## Goal

Eliminate five operator-facing defects introduced by phase 9k.1 so the Entra
configuration flow matches the operator contract in
`crates/dms-desktop/AGENTS.md` lines 95-103:

1. `Configure → Workflow → Manage identity source → Save application configuration`
   must show an in-place success notice on the same screen; it must not navigate
   the user to any other view.
2. The "Sign-in page" link produced by Microsoft Entra device authorization
   must open in the host's default browser via Tauri, not navigate the in-app
   WebView.
3. When an Entra device code has expired or the previous sign-in attempt
   failed, the operator must be able to regenerate a new challenge from the
   same surface without leaving the workflow.
4. A valid delegated Microsoft Entra token must persist in the OS credential
   store even when its serialized value exceeds the Windows Credential Manager
   2,560-byte UTF-16 password-blob limit.
5. DMS must request permission to read each direct user's profile and enabled
   state; Graph's limited-information response must report the missing
   `User.Read.All` consent rather than misdiagnose a missing display name.

## Current state

- `crates/dms-desktop/ui/app.mjs:1648-1654` writes `appState.workspace =
  result.workspace` unconditionally after every configuration mutation, but
  `configure_global_entra` (`crates/dms-desktop/src/lib.rs:351-375`) returns a
  flat `GlobalEntraConfiguration` without a nested `workspace` field, so
  `result.workspace === undefined` clobbers `appState.workspace` and `render()`
  falls back to `setupMarkup`.
- `crates/dms-desktop/ui/configuration.mjs:186` renders the device-flow
  `verification_uri` as `<a target="_blank" rel="noreferrer">`, and
  `crates/dms-desktop/ui/library.mjs:448` does the same for approver sign-in.
  Tauri 2 WebView does not forward `target="_blank"` to the OS browser.
- `crates/dms-desktop/src/graph.rs:278-322` exposes `begin_identity_source_setup`
  as a one-shot: `pending.remove(&challenge_id)` consumes the entry on
  `complete_identity_source_setup` (line 328). After expiry the frontend sees
  the stale challenge (kept in `state.configuration.identity_setup.challenge`)
  with no regeneration affordance — `configuration.mjs:185-189` only renders the
  input form when no challenge is present.
- `crates/dms-desktop/src/graph.rs:249-300` splits the serialized delegated
  token, but its 2,048-unit chunk limit is not safe for the installed
  `windows-native-keyring-store` backend. That backend encodes passwords as
  UTF-16 and enforces Windows' 2,560-**byte** credential-blob limit, so a
  2,048-unit ASCII fragment becomes a 4,096-byte blob. The Entra response is
  valid; the failure is at the desktop credential-store boundary.
- `crates/dms-desktop/src/graph.rs:673-720` requests display name, mail,
  principal name, and `accountEnabled` for direct user members, but the delegated
  scope previously contained only `User.Read` and `GroupMember.Read.All`.
  Microsoft Graph's directory-object relationship contract therefore returns
  inaccessible user objects with limited information (ID and object type while
  profile fields are null). `User.ReadBasic.All` would expose basic profile data
  but not the `accountEnabled` state DMS uses to exclude disabled accounts;
  delegated `User.Read.All` with tenant-admin consent is required.
- No shell-opener dependency is registered. `tauri = "2.11.5"` is in
  `Cargo.toml:25`; Tauri 2 exposes `AppHandle::shell().open(...)` without an
  additional plugin. The closest precedent in this workspace is
  `crates/dms-desktop/src/lib.rs:2181` (deep-link `on_open_url`).
- `crates/dms-desktop/AGENTS.md:96-103` explicitly states: "Configuration
  remains one session activity across Workspace, Document defaults, Workflow,
  and Notifications routes." A `Configure → … → Save` that leaves the activity
  is a contract violation.

## Context sources

The fresh-session contract for executing these phases loads only this CHG, its
AGENTS chain, and the files named below. Phase-specific file:line references
are repeated inside each phase; the list here is the loading checklist.

- `crates/dms-desktop/ui/configuration.mjs` (`identitySourceMarkup`,
  `configurationMutationRequest` `global-entra` / `identity-source-start` /
  `identity-source-complete` branches; `applyConfigurationSnapshot` helper)
- `crates/dms-desktop/ui/app.mjs` (`handleSubmit` configuration branch lines
  1607-1664; approver sign-in branch lines 1106-1145)
- `crates/dms-desktop/ui/library.mjs` (`externalLifecycleMarkup` lines 441-460 —
  approver sign-in button)
- `crates/dms-desktop/src/lib.rs` (`configure_global_entra` lines 351-375;
  `begin_identity_source_sign_in` / `complete_identity_source_sign_in` lines
  484-508; `invoke_handler` registration around line 2207; existing test module
  around lines 2540-2620)
- `crates/dms-desktop/src/graph.rs` (`begin_identity_source_setup` lines
  278-283; `complete_identity_source_setup` lines 324-348; `begin_delegated_sign_in`
  lines 292-322; `OsTokenStore` lines 139-177)
- `crates/dms-desktop/Cargo.toml` (Tauri 2.11.5 workspace dep, no
  `tauri-plugin-opener`)
- `crates/dms-desktop/ui/configuration.test.mjs` and
  `crates/dms-desktop/ui/library.test.mjs` (frontend tests; create the latter
  if absent)
- `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md`
  (`Implemented subset` items 1, 4, 5)
- `crates/dms-desktop/AGENTS.md` (Configuration route contract lines 95-103)
- Related CHG for context only (not modified):
  `docs/changes/active/CHG-0001-tauri-local-dms-bootstrap.md` Phase 9k.1 (line
  174) and phase 9l (line 175)

## Scope

Close three operator-facing contract gaps that surfaced during live setup of the
runtime Entra configuration shipped in phase 9k.1 of CHG-0001:

1. The **Application Entra configuration** card on `Configure → Workflow →
   Manage identity source` shows an in-place success notice after
   **Save application configuration** and never changes the active Configuration
   activity. CAP-0021's "Configuration remains one session activity across
   Workspace, Document defaults, Workflow, and Notifications routes"
   (`crates/dms-desktop/AGENTS.md:95-103`) is currently violated by this form
   because `handleSubmit` writes `appState.workspace = result.workspace` even
   though `configure_global_entra` returns a flat `GlobalEntraConfiguration`
   shape (`crates/dms-desktop/ui/app.mjs:1648-1654`,
   `crates/dms-desktop/src/lib.rs:351-375`).
2. The device-flow sign-in page produced by Microsoft Entra (`verification_uri`
   on the `DeviceLoginChallenge`) opens in the host's default browser through a
   validated native host-launcher IPC, not by navigating the in-app WebView. Today the
   same URL is rendered as `<a target="_blank" rel="noreferrer">` in
   `crates/dms-desktop/ui/configuration.mjs:186` and
   `crates/dms-desktop/ui/library.mjs:448`; Tauri 2 WebView does not forward
   `target="_blank"` to the OS browser.
3. When a device code has expired or the previous sign-in attempt failed, the
   operator can regenerate a new challenge on the same surface with a single
   **Sign in again** control that re-uses the last entered library group ID
   (Configuration) or simply re-invokes the approver sign-in (Library). The
   current markup keeps the stale challenge in
   `state.configuration.identity_setup.challenge` and offers no escape
   (`crates/dms-desktop/ui/configuration.mjs:185-189`,
   `crates/dms-desktop/src/graph.rs:278-322`).

The slice is an Entra configuration UX correction plus required Graph-adapter
remediations discovered during live device-flow sign-in: accept documented
optional fields in a successful token response, keep oversized delegated tokens
in OS-credential-store-only chunk entries, and request the delegated
`User.Read.All` permission required to read direct members' profile and
`accountEnabled` fields. It does not change the runtime Entra configuration
shape, privacy posture, or Office export pipeline.
CHG-0001 phase 9l (Windows external smokes + CAP promotion) remains untouched
and continues to own the Office/release evidence gate.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Fix global-entra save to stay on the configuration screen | done (`node --test crates/dms-desktop/ui/*.test.mjs` — 61 passed; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace`; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`) | `node --test crates/dms-desktop/ui/configuration.test.mjs` exits 0 with a new test asserting the success notice equals `"Application Entra configuration saved."` and that the `GlobalEntraConfiguration` payload is never fed to `applyConfigurationSnapshot`; all existing configuration tests still pass |
| 2 | Accept documented optional fields in a successful device-flow token response | done (`CARGO_INCREMENTAL=0 cargo test -p dms-desktop device_flow_token_response_accepts_documented_optional_fields`; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace`; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs` — 61 passed) | `CARGO_INCREMENTAL=0 cargo test -p dms-desktop device_flow_token_response_accepts_documented_optional_fields` exits 0 after a RED run where the current strict response parser rejects `token_type`, `scope`, and `id_token`; `CARGO_INCREMENTAL=0 cargo test --workspace`, `cargo fmt --all -- --check`, and `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings` exit 0 |
| 3 | Open device-flow verification_uri in the host browser (Configuration + Library) | done (`CARGO_INCREMENTAL=0 cargo test -p dms-desktop external_url_validation_allows_only_browser_safe_urls`; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace`; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs` — 63 passed; `DMS_DESKTOP_SMOKE=1 CARGO_INCREMENTAL=0 cargo run -p dms-desktop`) | `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs`; a new Rust unit test for `validate_external_url` covering `https://example.com` → `Ok`, `file:///etc/passwd` → `Err`, `javascript:alert(1)` → `Err`, empty → `Err`, `http://localhost:1234` → `Ok`, `http://example.com` → `Err`; frontend markup emits `data-open-external="https://…"` and never `target="_blank"` for the device-flow URI |
| 4 | Persist oversized delegated Entra tokens within the OS credential store | done (Windows-native RED reproduced `TooLong("password encoded as UTF-16", 2560)` at 2,048 units; GREEN native ASCII and supplementary-plane write/read/delete passed at 1,024 UTF-16 units; the production credential adapter rejects larger fragments before `keyring`; oversized round-trip, legacy-load, and Unicode-boundary regressions passed; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace` — 43 desktop tests; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs` — 63 passed) | A Windows-native synthetic credential write accepts ASCII and supplementary-plane passwords at the shared 1,024-UTF-16-unit chunk limit and deletes them afterward; the OS credential adapter rejects larger fragments; an oversized delegated token round-trips through chunked credentials; legacy single-entry JSON still loads; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace`; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings` |
| 5 | Request permission to read direct users' profile and enabled state | done (focused RED reproduced the absent `User.Read.All` scope and the false display-name error; GREEN scope and ID-only limited-information regressions passed; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace` — 42 desktop tests and all workspace suites passed; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs` — 63 passed; `DMS_DESKTOP_SMOKE=1 CARGO_INCREMENTAL=0 cargo run -p dms-desktop` — exit 0 with the known Chromium class-unregister cleanup warning; operator guide, ADR-0021, and CAP-0021 updated) | Focused RED/GREEN tests prove `GRAPH_SCOPE` contains `User.Read.All` and an ID-only Graph member response reports the required delegated permission/admin-consent recovery; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace`; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`; operator setup, ADR-0021, and CAP-0021 name the permission contract |
| 6 | Allow regenerating an expired or failed device-code challenge | done (focused frontend RED/GREEN tests prove failed Configuration and Library challenges expose same-surface **Sign in again** controls while active challenges do not; focused Rust RED/GREEN proves beginning sign-in discards expired pending challenges; `cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace` — 44 desktop tests and all workspace suites passed; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs` — 65 passed; `DMS_DESKTOP_SMOKE=1 CARGO_INCREMENTAL=0 cargo run -p dms-desktop` — exit 0 with the known Chromium class-unregister cleanup warning) | Failed Configuration sign-in renders an `identity-source-start` restart form with the transient last group ID; failed Library approver sign-in reuses `data-library-approver-sign-in`; neither appears for an active challenge; no new mutation kind or IPC command exists; issuing a challenge sweeps expired pending entries; full Rust, Clippy, frontend, formatting, and desktop smoke gates pass |
| 7 | CAP-0021 amendment + DOX closeout | pending | CAP-0021 `Implemented subset` lists four present-tense bullets for in-place save confirmation, host-browser opener, OS credential-store token chunking, and expired-challenge regenerate control; `docs/changes/README.md` still lists both CHG-0001 and CHG-0002 as active; `docs/changes/active/` contains both files; `crates/dms-desktop/AGENTS.md` Configuration contract unchanged (the fix conforms to it); `git diff --check` clean; conventional commit lands with explicit verification evidence |

**Current phase:** 7 — CAP-0021 amendment and DOX closeout. Each phase below carries the steps,
verification gate, and recovery path the executor must follow.

Mark a phase `in-progress` only while it is being executed, `done
(<evidence>)` only after its gate passes, and `pending` otherwise.

### Phase 1 — Fix global-entra save to stay on the configuration screen

**Goal:** `app.mjs:handleSubmit` treats `global-entra` as a mutation that
returns a `GlobalEntraConfiguration` (not a `WorkspaceConfiguration`); it
merges the result into `state.snapshot.global_entra_configuration`, surfaces
the notice, and never writes to `appState.workspace`.

**Steps:**

1. Read `crates/dms-desktop/ui/app.mjs:1607-1664` and the
   `applyConfigurationSnapshot` helper at
   `crates/dms-desktop/ui/configuration.mjs:33-43`.
2. Add a new branch in the `handleSubmit` configuration path that matches
   `configurationMutation === "global-entra"` before the generic success
   branch on line 1637. The branch must:
   - call `await invokeCommand(request.command, request.arguments)` (no
     `editRoot` prefix — `configure_global_entra` takes `clientId` / `tenantId`
     only, see `lib.rs:351-375`),
   - produce a synthetic snapshot
     `{ …state.configuration.snapshot, global_entra_configuration: result }`,
   - emit `state.notice = "Application Entra configuration saved."`,
   - clear `state.error`, leave `identity_setup` untouched,
   - **not** assign `workspace:` and **not** call `applyConfigurationSnapshot`
     with the raw result.
3. Update the `notices` map (line 1637) to include
   `global-entra: "Application Entra configuration saved."` so a future
   refactor that lifts the branch back into the generic path still produces
   the right notice.
4. Add a test in `crates/dms-desktop/ui/configuration.test.mjs`:
   - construct a snapshot that contains `global_entra_configuration` and a
     `workspace` summary,
   - simulate the post-submit state via
     `applyConfigurationSnapshot(state, { ...snapshot, global_entra_configuration: { ... } }, "Application Entra configuration saved.")`
     and assert the notice + the unchanged workspace fields,
   - assert that calling
     `applyConfigurationSnapshot(state, { client_id: "x", tenant_id: "y", client_id_environment_managed: false, tenant_id_environment_managed: false }, "…")`
     (i.e. the raw `GlobalEntraConfiguration` shape) does **not** produce a
     `state.error` and does not produce a `notice` — proving the frontend
     branch should never feed the raw payload to `applyConfigurationSnapshot`.
5. Run `node --test crates/dms-desktop/ui/configuration.test.mjs`. The new test
   must pass; all existing tests must still pass.

**Verification gate:** the new test in
`crates/dms-desktop/ui/configuration.test.mjs` passes;
`node --test crates/dms-desktop/ui/*.test.mjs` exits 0;
`cargo fmt --all -- --check`, `cargo test --workspace`, and
`cargo clippy --workspace --all-targets -- -D warnings` all exit 0.

### Phase 2 — Accept documented optional fields in a successful device-flow token response

**Goal:** The Graph adapter parses Microsoft Entra's documented successful
device-flow token response while retaining strict deserialization for fields it
uses to persist the delegated token. In particular, a response containing
`token_type`, `scope`, and `id_token` alongside `access_token`,
`refresh_token`, and `expires_in` must not fail before the preview can begin.

**Steps:**

1. In `crates/dms-desktop/src/graph.rs`, add a focused unit test named
   `device_flow_token_response_accepts_documented_optional_fields` that passes
   a successful token response with `token_type`, `scope`, and `id_token` to
   `parse_success::<OAuthTokenResponse>` and asserts the persisted fields.
2. Run `CARGO_INCREMENTAL=0 cargo test -p dms-desktop
   device_flow_token_response_accepts_documented_optional_fields`; it must fail
   because `OAuthTokenResponse` currently uses `#[serde(deny_unknown_fields)]`.
3. Remove `deny_unknown_fields` from `OAuthTokenResponse` only. Do not add or
   store Microsoft token fields DMS does not use; Serde ignores them.
4. Re-run the focused test, then the phase gate.

**Verification gate:** the focused test completes a Microsoft-shaped response
containing documented optional fields; `cargo fmt --all -- --check`,
`CARGO_INCREMENTAL=0 cargo test --workspace`, and
`CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`
all exit 0.

### Phase 3 — Open device-flow verification_uri in the host browser (Configuration + Library)

**Goal:** The device-flow verification URL opens the host's default browser
through DMS's existing platform-native launcher. The stale
`<a target="_blank">` markup is gone. DMS does not add Tauri shell/opener
plugins for this: the pinned dependency graph contains neither plugin, while
`open_host_path` already owns the native Windows/macOS/Linux launch boundary.

**Steps:**

1. In `crates/dms-desktop/src/lib.rs`, add a
   `#[tauri::command] open_external_url(url: String) -> Result<(), String>`
   that:
   - rejects empty URLs,
   - parses with `url::Url::parse(&url)` (the workspace already pulls `url`
     transitively via Tauri — verify, otherwise add `url.workspace = true` to
     `Cargo.toml`),
   - rejects schemes other than `https` (allow `http` only when the host is
     `localhost` or `127.0.0.1`, for local-dev convenience — emit an error
     string that names the rejected scheme so the frontend can surface it),
   - calls a small `open_host_url(&url)` helper that starts `explorer.exe` on
     Windows, `open` on macOS, or `xdg-open` on other Unix hosts, matching the
     established `open_host_path` platform boundary,
   - returns `Ok(())` on success or a descriptive `Err(String)` on validation
     / open failure.
2. Register the new command in the existing `invoke_handler` builder near
   `lib.rs:2207` (`configure_global_entra,` is the closest neighbour).
3. Add a Rust unit test in `crates/dms-desktop/src/lib.rs` (under the existing
   test module — see lines 2540-2620 for the established pattern) that
   exercises the validation function directly:
   `https://example.com` → `Ok`,
   `file:///etc/passwd` → `Err`,
   `javascript:alert(1)` → `Err`,
   empty string → `Err`,
   `http://localhost:1234/foo` → `Ok` (only because local dev),
   `http://example.com` → `Err`.
4. Refactor the validator into a pure helper
   (`fn validate_external_url(url: &str) -> Result<url::Url, String>`) so the
   unit test does not need an `AppHandle`.
5. In `crates/dms-desktop/ui/configuration.mjs:185-189`, replace the
   `<a target="_blank">` link with a button:
   `<button type="button" data-open-external="${escapeHtml(setup.challenge.verification_uri)}">Open sign-in page</button>`.
   Remove the `target="_blank"` attribute entirely.
6. In `crates/dms-desktop/ui/library.mjs:448`, replace the inline URL text +
   Complete-button pair with a button form whose first control opens the
   external URL (same `data-open-external` attribute), then the existing
   Complete button.
7. Add a delegated click handler in `crates/dms-desktop/ui/app.mjs` that
   resolves any `data-open-external` ancestor and calls
   `invokeCommand("open_external_url", { url })`. Add the handler alongside
   the existing delegated handlers (see `data-configuration-secondary` around
   `app.mjs:1485` and `data-library-approver-sign-in` at `app.mjs:1106` for
   the established pattern). The handler must render an error state on
   rejection but never navigate the WebView.
8. Extend `crates/dms-desktop/ui/configuration.test.mjs`:
   - render the challenge card from a synthetic
     `state.configuration.identity_setup.challenge`,
   - assert `data-open-external="https://…"` is present,
   - assert the markup does **not** contain `target="_blank"`.
9. Extend `crates/dms-desktop/ui/library.test.mjs` (or create it if it
   doesn't exist — verify with `ls crates/dms-desktop/ui/library.test.mjs`):
   - if missing, create it mirroring the configuration test layout,
   - assert `data-open-external` is present in the approver sign-in branch.

**Verification gate:** the new frontend tests pass; the new Rust unit test
passes; `cargo fmt --all -- --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`node --test crates/dms-desktop/ui/*.test.mjs` all exit 0.

**Recovery path:** add the existing locked `url` crate as an explicit workspace
dependency if direct use proves unavailable; do not silently fall back to
substring checks. Do not add Tauri shell/opener plugins for this URL-only use
case.

### Phase 4 — Persist oversized delegated Entra tokens within the OS credential store

**Goal:** A delegated token whose serialized UTF-16 representation exceeds the
Windows Credential Manager password limit remains entirely inside the OS
credential store. The primary existing entry becomes a small versioned manifest;
freshly generated credential entries hold UTF-16-safe fragments. No token,
fragment, encryption key, or recovery material enters `.dms`, frontend state,
or a local file.

**Steps:**

1. In `crates/dms-desktop/src/graph.rs`, separate the token-format logic from
   `OsTokenStore` behind a small private credential-entry interface so unit tests
   can use an in-memory map and production alone calls `keyring::Entry`.
2. Serialize `DelegatedToken` exactly as today, but split the resulting string
   only at Unicode scalar boundaries. Each fragment must contain at most 1,024
   UTF-16 code units. Windows Credential Manager limits the UTF-16 password
   blob to 2,560 bytes, not 2,560 code units; 1,024 units leaves margin below
   that byte limit. Do not split by byte index or Rust `String` length.
3. Store fragments under a UUID generation beneath the existing tenant/purpose
   namespace. Write all new fragments first, then replace the primary entry with
   a versioned manifest containing only the generation and fragment count. This
   preserves the previously committed manifest if a fragment write fails.
4. Read a manifest by loading every referenced fragment in order before parsing
   the reassembled JSON. Bound the fragment count and map absent/invalid
   fragments to the existing generic invalid-cache recovery message. Continue to
   parse a non-manifest primary entry as the legacy single-entry JSON shape so
   currently stored short tokens remain usable after upgrade.
5. After a successful manifest switch, best-effort delete only the prior
   manifest generation. Failed cleanup must not report a successful saved token
   as failed; the new manifest remains authoritative and later saves retry
   cleanup for their own prior generation.
6. Add `oversized_delegated_token_round_trips_through_chunked_credentials` in
   `graph.rs` with a synthetic oversized token. Assert the serialized input
   exceeds 2,560 UTF-16 units, every test-backend password is at most 1,024
   units, and the loaded value equals the input. Add a separate legacy
   single-entry load regression. Add a Windows-only native credential-store test
   that writes and reads a synthetic password at the shared fragment limit, then
   deletes the temporary entry. It must exercise `keyring::Entry`, not the
   in-memory credential backend.
7. Run the native focused test RED before reducing the limit, then run it GREEN
   and the phase gate.

**Verification gate:** `CARGO_INCREMENTAL=0 cargo test -p dms-desktop
oversized_delegated_token_round_trips_through_chunked_credentials`, the
legacy-load regression, and the Windows-native fragment write regression pass;
`cargo fmt --all -- --check`,
`CARGO_INCREMENTAL=0 cargo test --workspace`, and
`CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`
all exit 0.

**Recovery path:** If a Windows-native write rejects a 1,024-UTF-16-unit
fragment, reduce the shared fragment limit and re-run the native and synthetic
regressions. Do not bypass the OS credential store with filesystem, registry,
`.dms`, or frontend persistence.

### Phase 5 — Request permission to read direct users' profile and enabled state

**Goal:** Device authorization and token refresh request delegated
`User.Read.All`, which is required for the direct-member query's profile and
`accountEnabled` fields. A token lacking this permission produces an actionable
admin-consent/sign-in-again error when Graph returns an ID-only limited user
object; DMS does not fabricate a name or treat unknown account state as enabled.

**Steps:**

1. Add a focused test proving `GRAPH_SCOPE` contains `User.Read.All`.
2. Add a fake-backed Graph regression whose direct-user page contains only an
   object ID and assert the result names delegated `User.Read.All`, tenant-admin
   consent, and a fresh sign-in as the recovery path.
3. Run both tests RED against the prior scope and display-name validation.
4. Replace delegated `User.Read` with `User.Read.All` in `GRAPH_SCOPE`; retain
   `GroupMember.Read.All` and the OpenID/offline scopes.
5. Detect only the all-null limited-information shape before field-specific
   validation. Report both permission checks documented by Microsoft Graph:
   delegated `User.Read.All` tenant-admin consent and the signed-in user's
   authorization to read group members. Preserve the existing display-name,
   email, and account-status errors for partially populated malformed objects.
6. Update `docs/entra-client-setup.md`, ADR-0021 in
   `docs/design-decisions.md`, and CAP-0021 with the two delegated Graph
   permissions and why `User.ReadBasic.All` is insufficient.

**Verification gate:** focused scope and limited-information regressions pass;
`cargo fmt --all -- --check`, `CARGO_INCREMENTAL=0 cargo test --workspace`, and
`CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`
exit 0; the operator guide, ADR-0021, and CAP-0021 consistently require
delegated `User.Read.All` and `GroupMember.Read.All` with tenant-admin consent.
The limited-information error also names the signed-in user's group-member-read
authorization check; it does not claim that app consent is the only cause.

**Recovery path:** if Microsoft documents a narrower delegated permission that
exposes `accountEnabled` for other group members, replace `User.Read.All` with
that permission and rerun the focused tests. Do not weaken the enabled-account
eligibility contract or substitute object IDs for human identity fields.

### Phase 6 — Allow regenerating an expired or failed device-code challenge

**Goal:** When `state.configuration.identity_setup.challenge` exists and
either has expired server-side or was just failed by
`complete_identity_source_sign_in`, the operator sees an explicit "Sign in
again" control on the same surface that re-submits the original group ID.
The same affordance exists for the Library approver sign-in path.

**Steps:**

1. Re-calling `begin_identity_source_setup` already allocates a new
   `challenge_id`, but the Graph adapter has no pending cleanup. Before issuing
   a replacement challenge, retain only pending entries whose expiry is still
   in the future. Add a focused Rust regression proving an expired entry is
   discarded while the newly issued challenge remains pending.
2. In `crates/dms-desktop/ui/configuration.mjs`:
   - capture the `groupId` from the most recent `identity-source-start`
     submission into `state.configuration.identity_setup.last_group_id` (a
     transient field, not persisted — it is the operator's last intent),
   - when `setup.challenge` and `state.error` are present, render an additional
     `data-configuration-form="identity-source-start"` form above the existing
     Complete form, with the transient `last_group_id` in a hidden field:
     `<button class="button secondary" type="submit">Sign in again</button>`,
   - prefix the existing `Complete …` heading with
     `"Previous sign-in failed — "`.
3. In `crates/dms-desktop/ui/app.mjs:handleSubmit`, keep using the existing
   `identity-source-start` branch for both initial and replacement challenges.
   Capture the submitted group ID into the replacement `identity_setup` as
   `{ challenge: result, last_group_id: request.arguments.groupId }`; a
   successful replacement clears the existing configuration error.
4. Do not add a new configuration mutation kind or IPC command. The restart
   form maps through the existing `identity-source-start` request.
5. Mirror the same affordance for the Library approver sign-in path
   (`crates/dms-desktop/ui/app.mjs:1106-1145` and
   `crates/dms-desktop/ui/library.mjs:447-451`):
   - when `library.approver_sign_in?.challenge` is set and
     `library.detail_error` (or the relevant error channel — verify against
     `app.mjs:1118-1142`) is non-empty, render a
     `data-library-approver-sign-in-restart` button that re-calls
     `begin_approver_sign_in`,
   - clear `library.detail_error` on restart.
6. Extend `crates/dms-desktop/ui/configuration.test.mjs`:
   - construct
     `state.configuration.error = "Microsoft Entra sign-in challenge is no longer available; start again"`,
   - construct
     `state.configuration.identity_setup = { challenge: { … }, last_group_id: "00000000-0000-0000-0000-000000000000" }`,
   - render the markup,
   - assert `data-configuration-form="identity-source-restart"` is present and
     the "Previous sign-in failed" prefix appears,
   - render the markup *without* `state.configuration.error`,
   - assert the restart form is **not** present (the stale challenge is still
     actionable, no restart needed yet).
7. Extend `crates/dms-desktop/ui/library.test.mjs` (or create it per
   Phase 2) with the same assertions for the approver sign-in branch.
8. Run `cargo test --workspace` and
   `node --test crates/dms-desktop/ui/*.test.mjs`. Both must pass.

**Verification gate:** the new frontend tests pass; the existing `phase 9j`
Graph client fake tests still pass (no change to the Graph public surface
in this phase); `cargo fmt --all -- --check`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`node --test crates/dms-desktop/ui/*.test.mjs` all exit 0.

**Recovery path:** if the Graph `pending` map grows unbounded under repeated
restarts, add a `pending.retain(|_, p| p.expires_at > Instant::now())` sweep
inside `begin_delegated_sign_in` (`graph.rs:292-322`) in the same change —
do not split into a separate CHG phase, and document the sweep in the CHG
row's evidence line.

### Phase 7 — CAP-0021 amendment + DOX closeout

**Goal:** The active CHG records this slice; CAP-0021 reflects the new
operator-visible behaviour; the DOX chain stays consistent.

**Steps:**

1. Mark each completed phase row in this file as `done (<evidence>)` using
   the gate output for that phase. Update the `**Current phase:**` line to
   point at the next pending row, or remove the line if every phase is done.
2. Amend `docs/product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md`
   `Implemented subset` to add four present-tense bullets immediately after
   item 1:

   ```
   - The **Application Entra configuration** card confirms a successful save with an in-place status notice and remains on the same Configuration secondary surface; the active Configuration activity never changes as a side effect of saving global Entra configuration.
   - The device-flow sign-in page opens in the host's default browser through a Tauri shell opener that accepts only `https:` URLs (and `http://localhost`/`http://127.0.0.1` for local development); the in-app WebView never navigates to the verification URI.
   - Delegated Microsoft Entra tokens remain exclusively in the OS credential store. When their serialized UTF-16 representation exceeds the Windows Credential Manager per-password limit, DMS writes UTF-16-safe fragments under a versioned manifest and continues to read the previous single-entry cache format.
   - An expired or failed Microsoft Entra device-flow challenge surfaces an explicit **Sign in again** control on the same surface that re-issues a fresh challenge with the operator's last group ID; the previous pending challenge is discarded by the Graph adapter.
   ```
3. Replace the `## Links → ## Progress` line at the bottom of CAP-0021 so it
   lists both CHG-0001 (phase 9k.1 / 9l context) and CHG-0002 (the
   operator-visible UX corrections).
4. Confirm `docs/changes/README.md` `## Active` table still lists exactly two
   active CHGs (CHG-0001 and CHG-0002). The `## Rules` section permits this:
   "Exactly one active CHG progress authority per material request" — each
   CHG is the sole authority for its own request.
5. Re-read `crates/dms-desktop/AGENTS.md` lines 95-103 — the fix conforms to
   the Configuration contract; no AGENTS.md edit required.
6. Run `git diff --check` and any records-link validator if one exists
   (`pnpm records:check` or equivalent — `docs/AGENTS.md:42-44` states none
   exists yet).
7. Stage the diff and commit with a conventional commit message generated
   via `github/dev-git-commit-message`: scope `dms-desktop`, summary
   `fix(configuration): confirm Entra save, open device flow in host browser, persist oversized delegated tokens, regenerate expired challenge`.
   The commit must include this CHG file, the README update, the CAP-0021
   amendment, the code changes, and the new tests. It lands locally; pushing
   requires separate explicit operator authorization.

**Verification gate:** `git diff --check` clean; every CHG-0002 phase row
carries `done (<evidence>)`; CAP-0021 `Implemented subset` contains the four
new bullets; `docs/changes/README.md` still lists exactly two active CHGs and
zero archived CHGs; commit message follows conventional format; the engine
(this CHG's eventual executor) confirms the commit lands locally without
pushing.

## Risk call-out

- The shell-opener IPC must validate the URL scheme against an explicit
  allowlist (`https:` only, plus `http://localhost` / `http://127.0.0.1` for
  local dev) before calling `AppHandle::shell().open(...)`. An unvalidated
  opener is a primitive for arbitrary URI launches from a tampered frontend
  payload; the existing `target="_blank"` was inert, so this slice *adds* that
  surface area. The validator must live in `crates/dms-desktop/src/lib.rs`
  next to the existing `configure_global_entra` command and be unit-tested
  directly.
- The **Sign in again** control must re-use the existing
  `begin_identity_source_sign_in` IPC plus the existing
  `identity-source-start` mutation kind. It must not introduce a new pending
  challenge state in the frontend, a new IPC command, or a new Graph lifecycle.
  The `last_group_id` is transient frontend session state; it must not be
  persisted to `<edit-root>/.dms` or `global-settings.json`, since that would
  leak the operator's group choice across workspaces and is not part of the
  CAP-0021 contract.
- Phase 9k.1's risk call-out in CHG-0001 (lines 185-208) still applies in
  full: no Entra credentials, tokens, group object IDs, tenant IDs, client
  IDs, or device codes cross the IPC boundary, the persisted frontend state,
  the `.dms` store, an error message, a test fixture, or this CHG file.
  The new `open_external_url` IPC carries only the bare `verification_uri`
  that Microsoft Graph already returns to the desktop adapter, validated
  against the same allowlist that protects every other outbound launch in
  this codebase.
- Token fragments and their manifest remain OS credential-store entries. A new
  generation is written before the manifest switch so a failed fragment write
  cannot corrupt the previously committed cache; cleanup of superseded fragments
  is best effort and never changes a successful save into a reported failure.
- The Windows + macOS shell-opener behaviour must be exercised by the local
  Linux smoke (`DMS_DESKTOP_SMOKE=1 cargo run -p dms-desktop`) before the
  CI/platform smoke is claimed; if the local smoke fails to launch the host
  browser (e.g. missing `xdg-open`), the Rust unit test on the validator is
  the gate that still passes, and the CHG evidence line records that the
  local smoke was attempted with its observable outcome.

## Out of scope

- Adding `tauri-plugin-opener` as a new dependency. The in-process
  `AppHandle::shell().open(...)` covers the requirement and keeps the
  dependency surface unchanged; revisit only if a future slice needs richer
  open-with semantics.
- Persisting `last_group_id` in `<edit-root>/.dms` or `global-settings.json`.
- Re-issuing any CHG-0001 phase's evidence line. This slice does not
  retroactively re-prove 9j / 9k / 9k.1 — only its own phases.

## Links

- Capability contract: [`../product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md`](../product/capabilities/CAP-0021-microsoft-entra-workflow-identity.md)
- Related CHG: [`CHG-0001`](CHG-0001-tauri-local-dms-bootstrap.md) phase 9k.1 (runtime Entra configuration) and phase 9l (Windows external smokes, pending)
- AGENTS contract: [`../../crates/dms-desktop/AGENTS.md`](../../crates/dms-desktop/AGENTS.md) Configuration route (lines 95-103)