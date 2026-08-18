# CHG-0023 — OS-level `dms://` URI handler registration

**Plan ID:** CHG-0023-os-level-dms-uri-registration
**Created:** 2026-08-18
**Depends on:** none
**Context sources:** `docs/product/capabilities/CAP-0020-document-permalinks.md`, `crates/dms-desktop/src/lib.rs` (`run()`, setup closure, lines ~2686-2760), `crates/dms-desktop/tauri.conf.json`, `crates/dms-desktop/capabilities/default.json`, `crates/dms-desktop/ui/app.mjs` (`queuePermalinks`, `registerDeepLinkHandler`), `.github/workflows/desktop-platform-smoke.yml`, `docs/changes/README.md`
**Produces:** The `dms` scheme resolves to the desktop app at OS level on Windows (NSIS installer), Linux (first-run registration at app start, verified by the smoke gate), and macOS (DMG Info.plist); CAP-0020 reports the registered-scheme outcome; CI proves Linux registration on every push.
**Status:** done — closed 2026-08-18

| Field | Value |
| --- | --- |
| ID | CHG-0023 |
| Status | done |
| External request | Direct operator request: "In order to get the dms:// URI handler working, the dms URI need to be registered on operating system level. Windows and Linux (later also macOS) should be supported" |
| Affected CAPs | CAP-0020 |
| Decision records | ADR-0002 amendment (Linux becomes a supported desktop target) |

## Current state

- The runtime half of CAP-0020 is already in: `tauri_plugin_single_instance` is the first plugin and `tauri_plugin_deep_link::init()` is registered (`crates/dms-desktop/src/lib.rs:2688-2695`); `on_open_url` focuses the main window (`src/lib.rs:2709-2711`); the frontend queues permalinks via `deepLink.onOpenUrl` + `deepLink.getCurrent` (`ui/app.mjs:2665-2669`) and resolves them through the `resolve_registered_permalink` command (`src/lib.rs:354-366`, added in `3f643f3`).
- `tauri.conf.json` declares `plugins.deep-link.desktop.schemes: ["dms"]` and `bundle.active: false`; identifier is `de.videoclinic.dms`.
- **No OS-level registration exists yet.** Verified on this host: `xdg-mime query default x-scheme-handler/dms` returns empty.
- **Windows and macOS registration is already delivered by the bundler once a package is built**: tauri-cli maps `plugins.deep-link.desktop` into `Settings.deep_link_protocols` (`tauri-cli/src/interface/rust.rs`, `get_bundler_settings`); the NSIS template writes `Software\Classes\dms` `URL Protocol` keys and removes them on uninstall (`tauri-bundler/src/bundle/windows/nsis/installer.nsi:670-676, 805-811`); the macOS bundle injects `CFBundleURLTypes`/`CFBundleURLSchemes` into Info.plist (`tauri-bundler/src/bundle/macos/app.rs:297-317`). CI already builds both packages (`.github/workflows/desktop-platform-smoke.yml` matrix: `nsis` on Windows, `dmg` on macOS) even with `bundle.active: false`, because `--bundles` supplies the targets explicitly.
- **Linux is the real gap**: tauri-bundler performs no scheme registration for Linux targets; `tauri-plugin-deep-link` provides `register`/`register_all` (writes `~/.local/share/applications/<exe>-handler.desktop` with `MimeType=x-scheme-handler/dms` — in Tauri 2.11 `app.path().data_dir()` is the XDG data dir itself, i.e. the standard applications dir — then runs `update-desktop-database` and `xdg-mime default`), but nothing in the app calls it. The plugin's own `on_open_url`/`handle_cli_arguments` already handle the inbound URL once a handler exists.
- ADR-0002 states "Linux is not a v1 requirement" — superseded by the operator request; the root `AGENTS.md` product-shape line says "target Windows and macOS".
- `capabilities/default.json` carries `deep-link:default` (allows `get-current`); `onOpenUrl` is an event and needs no permission. Registering from the Rust side (not JS) means no capability change.
- The smoke gate `DMS_DESKTOP_SMOKE=1 cargo run -p dms-desktop` exits inside `setup()` (`src/lib.rs:2712-2714`), so a registration call placed before that check is exercised by the existing local and CI smoke.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Linux runtime scheme registration in `setup()` | done (smoke 2026-08-18: `~/.local/share/applications/dms-desktop-handler.desktop` with `MimeType=x-scheme-handler/dms` written; `xdg-mime query default x-scheme-handler/dms` → `dms-desktop-handler.desktop`; `cargo test -p dms-desktop` 54 passed; `node --test` exit 0) | After `DMS_DESKTOP_SMOKE=1 cargo run -p dms-desktop`: `~/.local/share/applications/dms-desktop-handler.desktop` exists with `MimeType=x-scheme-handler/dms` and `xdg-mime query default x-scheme-handler/dms` prints `dms-desktop-handler.desktop`; `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0 |
| 2 | Linux job in the platform smoke workflow | done (run 32131091296 green on all three jobs after the `shell: bash` + keyring `NoDefaultStore` fixes, commit `dbe514b`; `ubuntu-latest` passed the registration assertions) | Pushed `desktop-platform-smoke` run is green including a new `ubuntu-latest` job that runs the workspace gates plus the phase-1 registration assertions; run id recorded here |
| 3 | Windows/macOS installer registration evidence | done (run 32131091296 green with `windows-x64-nsis` 4.86 MB and `darwin-aarch64-dmg` 6.61 MB artifacts; NSIS/Info.plist scheme injection is bundler behaviour verified against tauri-bundler sources in this record's current state) | Latest CI run green with `nsis` and `dmg` artifacts uploaded (run id recorded here); host-side probes documented in this CHG as external gates (`reg query "HKCU\Software\Classes\dms"` after NSIS install; `plutil -p Info.plist` / `mdls` CFBundleURLTypes check on the DMG app) |
| 4 | Records: ADR-0002 amendment, root + desktop AGENTS, CAP-0020 status | done (`cargo test --workspace` all green + `node --test` exit 0; CAP-0020 flipped to implemented with its outcome list confirmed against runtime; CHG archived and `docs/changes/README.md` refreshed) | `cargo test --workspace` and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP-0020 outcomes all match runtime and its Status reflects it; CHG archived as `done` and `docs/changes/README.md` index refreshed |

Mark a phase `in-progress` while running it, `done` once its gate passes (record evidence), `pending` otherwise.

## Phase 1 — Linux runtime scheme registration

**Goal:** On Linux, the app registers itself as the `dms` scheme handler on every start, using the deep-link plugin's `register_all` (driven by the `plugins.deep-link.desktop.schemes` config, so no scheme is hardcoded twice). Registration writes the standard user applications dir (`~/.local/share/applications/`), so no app-data-dir bookkeeping is needed.

Steps:

1. In the `setup()` closure of `run()` (`crates/dms-desktop/src/lib.rs`, between the `on_open_url` listener and the `DMS_DESKTOP_SMOKE` exit check), add a `#[cfg(target_os = "linux")]` call to `app.deep_link().register_all()`. Registration failures (missing `xdg-mime` / `update-desktop-database` on minimal systems) must not fail app startup: log to stderr and continue.
2. Do not touch the frontend or capabilities: `getCurrent`/`onOpenUrl` already work and JS-side `register` would need a new permission for no benefit over the Rust call.
3. Run the phase gate from a clean state (the current host has no `dms` handler — verified 2026-08-18). If a handler already exists, remove it first (`xdg-mime` entry in `~/.config/mimeapps.list` + the plugin's `.desktop` file) so the gate proves registration, not a pre-existing state.

Recovery: registration only writes `~/.local/share/applications/dms-desktop-handler.desktop` and one `mimeapps.list` line — both user-scoped and re-writable on the next start. No workspace or document data is touched.

## Phase 2 — Linux CI job

**Goal:** Every push proves the Linux registration on a clean runner, so the scheme handler cannot silently regress.

Findings (2026-08-18, while authoring the job):
- The smoke step launches a real webview window before the `DMS_DESKTOP_SMOKE` exit inside `setup()`, so it needs a display. GitHub's `ubuntu-latest` is headless — the smoke runs under `xvfb-run`.
- The plugin's `register_all` needs `xdg-mime` (`xdg-utils`) and `update-desktop-database` (`desktop-file-utils`); the apt step installs both plus the standard Tauri Linux webview libraries and `xvfb` explicitly rather than trusting the runner image.
- Linux packaging is not part of this job (`bundle: ""` skips the packaging step; Linux deb/rpm/AppImage packaging stays out of scope).

Fixes applied after the first CI run (32129034916, failed on Windows + Linux):
- **Windows `ParserError` on the launch step**: the step used a bash `if` conditional, but Windows workflow steps default to PowerShell. The step now sets `shell: bash` (bash is preinstalled on all GitHub runners).
- **Two Linux test failures** (`desktop_configuration_commands_persist_workspace_and_document_defaults`, `desktop_configuration_commands_persist_workflow_and_notifications`): both call configuration command wrappers that probe the real OS credential store; a headless runner has no default keyring store, so `keyring::Entry::new` fails with `NoDefaultStore` and the unwrap panicked. Fix in `crates/dms-desktop/src/notify.rs`: `smtp_password_exists` now maps `NoDefaultStore` to `Ok(false)` — a system without any credential store cannot hold an SMTP password, so the credential is unconfigured, not unreadable. Entry construction errors are mapped to the same user-facing string as before at the call sites. Verified locally with `env -u DBUS_SESSION_BUS_ADDRESS cargo test -p dms-desktop desktop_configuration_commands_persist` (2 passed) plus the full crate suite.

Steps:

1. Add `os: ubuntu-latest, bundle: none` to the matrix in `.github/workflows/desktop-platform-smoke.yml`.
2. On the Linux job run the existing Rust format/tests/lint, `node --test`, and the `DMS_DESKTOP_SMOKE=1 cargo run -p dms-desktop` step, then a `Verify dms scheme registration` step asserting `~/.local/share/applications/dms-desktop-handler.desktop` with `MimeType=x-scheme-handler/dms` and the `xdg-mime query default` result from phase 1's gate.
3. Skip the `tauri-action` packaging step when `bundle` is empty (Linux packaging is out of scope).
4. Ensure `xdg-utils` and `desktop-file-utils` are present on the runner; add an install step only if the first run proves them missing.

Recovery: the workflow file is plain YAML; a broken job is reverted by one `git revert` or a direct edit in the next phase's commit.

## Phase 3 — Windows/macOS installer registration evidence

**Goal:** Establish that the already-built installers carry the scheme registration and record the host-side probe steps, so "works on Windows" is evidenced, not assumed.

Steps:

1. Confirm the latest `desktop-platform-smoke` run after phase 2 is green on `windows-latest` and `macos-latest` with installer artifacts uploaded; record the run id.
2. Record the external host gates in this CHG: on a Windows host with the NSIS build installed, `reg query "HKCU\Software\Classes\dms" /v "URL Protocol"` (or `HKLM` for per-machine installs) returns the key and `shell\open\command` points at the installed exe; on a macOS host with the DMG app, `CFBundleURLTypes` in the app's `Info.plist` lists the `dms` scheme.
3. No code changes are expected. If an artifact check shows a missing scheme block, the finding belongs in this phase: inspect the `tauri-action` tauri-cli version and the `plugins.deep-link.desktop` mapping before concluding the bundler is at fault.

## Phase 4 — Records: ADR, AGENTS, CAP, closeout

**Goal:** Bring current-state documentation in line with the shipped behaviour and close the change.

Steps:

1. Amend ADR-0002 in `docs/design-decisions.md`: Linux becomes a supported desktop target (operator-requested), scheme registration on Linux happens at app start via the deep-link plugin; macOS stays a supported target with later operator rollout.
2. Update the root `AGENTS.md` product-shape line ("target Windows and macOS") and the `crates/dms-desktop/AGENTS.md` Work Guidance bullet + Verification section (the Linux smoke now also proves scheme registration).
3. Check `docs/architecture.md` platform statements and align them.
4. Re-check every CAP-0020 outcome against the runtime; flip its Status to `implemented` with the existing test links only if all outcomes hold, otherwise keep the gap explicit.
5. Move this CHG to `docs/changes/archive/`, set Status `done`, and refresh `docs/changes/README.md`.

## Out of scope

- Linux packaging (deb/rpm/AppImage) in CI — runtime registration works without it; packaging is a later request.
- macOS verification work — bundle-level registration already ships in the DMG; operator-side macOS adoption is later by request.
- Dynamic scheme registration at runtime (`deepLink.register` from JS, additional schemes).
- Uninstall/cleanup of the Linux `.desktop` entry from the app (the plugin's `unregister` stays available but unused).
- Windows per-machine vs per-user install-mode policy (NSIS default behaviour stands).
