# CHG-0024 — Windows NSIS installer + signed GitHub Release

**Plan ID:** CHG-0024-windows-nsis-installer-release
**Created:** 2026-08-18
**Depends on:** none (CAP-0005 Phase 5 packaging slice)
**Context sources:** `docs/product/capabilities/CAP-0005-desktop-shell.md`, `crates/dms-desktop/tauri.conf.json`, `.github/workflows/desktop-platform-smoke.yml`, `docs/changes/archive/CHG-0023-os-level-dms-uri-registration.md`, `docs/design-decisions.md` (ADR-0002 platform targets), `docs/architecture.md`
**Produces:** On `v*` tag push, a `windows-latest` GitHub Actions run builds the NSIS installer (`crates/dms-desktop/tauri build --bundles nsis`), signs it with the Authenticode cert, uploads it to a GitHub Release alongside its SHA-256, and records the run id + size + hash in this CHG. The README's Windows install story is now "download the signed installer, run it".
**Status:** in-progress

| Field | Value |
| --- | --- |
| ID | CHG-0024 |
| Status | in-progress |
| External request | Direct operator request: "proceed as recommended with NSIS for Windows -- if I got it right. The release should be published on GitHub so using GitHub Actions is prefered" |
| Affected CAPs | CAP-0005 |
| Decision records | ADR-0027 (Windows installer distribution = signed NSIS on GitHub Releases, tag-driven workflow) |

## Current state

- `tauri.conf.json` declares `productName: "DMS Desktop"`, `version: "0.1.0"`, `identifier: "de.videoclinic.dms"`, `plugins.deep-link.desktop.schemes: ["dms"]`, and `bundle.active: false`. The bundler template already maps `plugins.deep-link.desktop` to the NSIS template's `deep_link_protocols` loop (`tauri-bundler/src/bundle/windows/nsis/installer.nsi:671-672, 806-807` against tauri v2.11.5), so installing the NSIS build writes `Software\Classes\dms` `URL Protocol` and `shell\open\command` and removes them on uninstall — CHG-0023 already verified the scheme-injection mechanism.
- The existing `desktop-platform-smoke.yml` matrix already runs `tauri build --bundles nsis` on `windows-latest` and uploads the artifact via `tauri-apps/tauri-action@v1` with `uploadWorkflowArtifacts: true`. What it does NOT do: trigger on tag push, create a GitHub Release, attach the installer to that release, sign the binary, or publish a SHA-256 next to it. Run id 32131091296 (4.86 MB `windows-x64-nsis`) is the most recent evidence the smoke build itself works.
- `bundle.active: false` is intentional: dev `cargo run -p dms-desktop` should not trigger bundling. The release workflow passes `--bundles nsis` explicitly on the build step, mirroring what the smoke already does — so no `tauri.conf.json` change is needed.
- tauri 2.11.5 reads Windows signing from `tauri.conf.json` (`bundle.windows.certificateThumbprint`, `bundle.windows.timestampUrl`, `bundle.windows.signCommand`). There is no env-var override for these (`crates/tauri-bundler/src/bundle/settings.rs:566-630` has no `env::var` path for them), so a signed build needs the thumbprint committed or supplied through a config merge. `tauri build --config <path>` (the CLI's `Options.config` field, `crates/tauri-cli/src/build.rs:50-58`) accepts a JSON/TOML/JSON5 file that is merged into the default config, which is the supported way to inject a thumbprint from a CI secret without touching the repo file.
- `actions/runner-images` `windows-latest` does NOT include the MSVC C++ workload by default; the `Microsoft.VisualStudio.2022.BuildTools` `OneCore` workload is enough for the Tauri build but not for Authenticode. `signtool` is available in the Windows SDK that ships with VS Build Tools 2022, and Tauri's `tauri sign` is a separate binary that wraps it. The workflow installs the cert into the CurrentUser\My store on the runner via `Import-PfxCertificate`, then calls `signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /sha1 <thumbprint> /f <pfx>` on the installer exe.
- No GitHub Release workflow exists yet. `softprops/action-gh-release@v2` is the standard pick: creates a release from a tag, attaches files, supports draft releases and prerelease flags. The repo is `videoclinic/dms` (origin `git@github.com:videoclinic/dms.git`, public — `gh repo view` reports `isPrivate: false`).
- ADR-0002 lists Windows, macOS, and Linux as supported targets. macOS DMG is a parallel concern (out of scope for this CHG; existing smoke matrix already covers it as a packaging smoke only — not as a release workflow).
- Let's Encrypt and ZeroSSL issue TLS certificates (EKU serverAuth/clientAuth, browser trust store) and cannot be used here: Authenticode requires an EKU `codeSigning` certificate chained to a root in Windows' code-signing trust list. Production options: OV code-signing cert (DigiCert/Sectigo/GlobalSign, SmartScreen warns until installs accrue) or EV code-signing cert (removes SmartScreen).
- Until the production cert is ordered, the signing pipeline is exercised with a **self-signed test code-signing CA** (730-day validity, EKU codeSigning, keyUsage keyCertSign+cRLSign, CN "DMS CI Code Signing (self-signed test)"). The workflow imports the PFX into `Cert:\CurrentUser\My` *and* `Cert:\CurrentUser\Root` on the ephemeral runner so `signtool verify /pa` can validate the chain — that trust import is a no-op for a real CA. Generated at `/tmp/dms-cert` (CA + leaf are the same self-signed cert; PFX password `test-password`); the base64 PFX and SHA-1 thumbprint `771d2d9847ca850f62431e36754a6786060ef5a1` go into the `WINDOWS_CERT_*` secrets as a test configuration. Installers signed with it verify in CI but are untrusted on end-user machines — expected for a pipeline test; the production cert replaces it by swapping the three secrets only.
- `git config user.name` / `user.email` are repo-local and the commits land under `videoclinic` identity (Raphael Bossek <raphael.bossek@videoclinic.de>).

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | `release-windows.yml` tag-driven workflow + first tagged release | in-progress | Workflow file committed; tag `v0.1.0-installer-preview.1` pushed (2026-08-18) → release run 32156557691 executing with the self-signed test cert (`WINDOWS_CERT_*` synced via gh-vault); gate: green run + draft release with signed installer + `.sha256` sidecar, then operator cert swap for the production release |
| 2 | README install story + ADR-0027 + CAP-0005 update | done (2026-08-18) | README Windows install section present; ADR-0027 in `docs/design-decisions.md`; CAP-0005 outcome 5 references the workflow; `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/app.test.mjs` all green |
| 3 | Smoke integration: release workflow runs do not regress the existing matrix | done (2026-08-18) | Non-tag push `f5ecc8e` triggered `Desktop platform smoke` runs 32138172649 + 32138171263, both `success`; zero `Release Windows installer` runs from the same push; `gh workflow list` shows both workflows active; `desktop-platform-smoke.yml` unmodified by this commit |
| 4 | Records closeout: archive this CHG, refresh `docs/changes/README.md` | pending | CHG moved to `archive/`, status `done`, README active index updated, archive entry present |

Mark a phase `in-progress` while running it, `done` once its gate passes (record evidence), `pending` otherwise.

Evidence (phase 1, partial):

- `release-windows.yml` committed; `gh act workflow_dispatch --dryrun` resolves the workflow and lists all 11 steps. Local Linux `gh act` cannot host `windows-latest`, so the CI run is the source of truth.
- `gh-vault workflow check --env-file <empty>` reports exactly the three `WINDOWS_CERT_*` references and no other issues; the repo declares them in `.env` (2 secrets, 1 variable) and `gh-vault secret check` passes after sync.
- Pre-commit gates green on the landing commit: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/app.test.mjs`.
- First tag run 32156557691 (`v0.1.0-installer-preview.1`, 2026-08-18) **failed at Compute SHA-256** with two root causes, both fixed in the follow-up commit:
  - The thumbprint is a GitHub **variable** (`.env` declares it `# gh-vault: variable`), but the workflow read `secrets.WINDOWS_CERT_THUMBPRINT` → empty → cert-import warned and exited 0, sign step skipped by its `if:`. Fix: `vars.WINDOWS_CERT_THUMBPRINT`.
  - The NSIS bundle lands in the **workspace-root** `target/release/bundle/nsis` (confirmed in the build log: `D:\a\dms\dms\target\release\bundle\nsis\DMS_Desktop_0.1.0_x64-setup.exe`), not a per-crate `target/`. Fix: all three path references (sign, hash, release upload).
- Re-run 32157572691 (workflow_dispatch, same tag) **failed at cert import**: `Import-Certificate` has no `-Certificate` parameter in any PowerShell version (PS 5.1 *and* 7). Fix: trust the cert via the .NET `X509Store` API, which works in both.
- Re-run 32157868239 **failed at `$store.Add($cert)`**: `Import-PfxCertificate` returns an *array* (`System.Object[]`) because the first PFX build contained the cert bag twice (`-in` + `-certfile`). Two fixes: regenerate the PFX with a single cert bag, and make the import robust — pick the cert by the pinned thumbprint from whatever the PFX carries (production bundles include the full chain) and only self-trust when the cert is actually self-signed (`Issuer -eq Subject`), so a real CA leaf still verifies against the system trust store.
- Re-run 32158199796 **failed at import** with a PowerShell parser error: `-ie` is not a PowerShell operator (the case-insensitive equality operator is `-ieq`) — and the replace needs its own parentheses. Both fixed: `($_.Thumbprint -replace '\s','') -ieq ...`. All three step scripts now parse cleanly under pwsh 7 (verified locally in a `mcr.microsoft.com/powershell:latest` container before pushing).
- Runs 32159384160 + 32163986746 **hung 27–50 min in the import step** (both cancelled). Phase markers (`[1/4]`–`[4/4]`) + the cancelled-run partial logs localize the block to the `.NET X509Store('Root','CurrentUser').Add()` call. Initial diagnosis blamed the private key (CAPI keyset creation) — later disproven.
- Run 32168893884 **failed fast (51 s) at import**: with the keyless export the step ran in 2.7 s but threw at keyless-cert construction — `New-Object X509Certificate2 <byte[]>` deconstructs the byte array into 952 scalar arguments ("Cannot find an overload ... argument count: 952"). Fix: `[...X509Certificate2]::new($bytes)` (single `byte[]` argument).
- Run 32176773256 **hung the full 35-min cap again** (cancelled by timeout at 20:03:30) — the `certutil -addstore` partial log shows `[4/4] self-signed cert: trusting … via certutil` at 19:29:08, then 34 min of silence. That **disproves the certutil path too** and the earlier "keyed cert" theory alike. The invariant across runs 5/6/8/9: every mechanism that writes to `Cert:\CurrentUser\Root` (X509Store keyed, X509Store keyless, certutil) blocks indefinitely on headless `windows-latest`, while every other call in the step is sub-second.
- **Final fix: no Root-store write at all.** The import step ends at cert selection (`[1/3]`–`[3/3]`). The self-trust only existed to satisfy `signtool verify /pa`'s strict chain for the *self-signed test* cert, so verification moves to **`Get-AuthenticodeSignature`**: reads the embedded signature, asserts the signer thumbprint equals the pinned one, and accepts `Status` `Valid` (future CA-issued cert) or `NotTrusted` (self-signed test cert). No trust store is consulted, so the step works for the test cert and a production cert alike. Job `timeout-minutes` dropped 60 → 35 so any future hang fails with its partial log instead of burning an hour.
- Run 32180394018 **first run past the import step in ~8 min**: import succeeded in 2 s (20:07:18 → 20:07:20), the NSIS build + tauri-action completed, `signtool sign` succeeded — then the new verification threw `Authenticode status UnknownError for A certificate chain processed, but terminated in a root certificate which is not trusted by the trust provider.` Root cause: pwsh 7 / .NET 8 classifies an untrusted self-signed root as `UnknownError`, not the `NotTrusted` bucket Windows PowerShell 5.1 uses (which the check was written for). The signature itself was valid and from the pinned cert — only the status-bucket assumption was wrong. Fix: accept `UnknownError` *only when* `StatusMessage` matches the untrusted-root wording, keeping `Valid`/`NotTrusted` as the normal paths so a genuinely broken signature (bad hash, revoked, wrong signer) still fails.

## Phase 1 — `release-windows.yml` + first tagged release

**Goal:** A `v*` tag push builds, signs, and publishes the NSIS installer to a GitHub Release; the operator can install DMS on Windows by downloading and running that signed exe.

Steps:

1. Add `.github/workflows/release-windows.yml`. Triggers: `push: tags: ['v*']` and `workflow_dispatch` (manual release of an existing tag). Permissions: `contents: write` (for `softprops/action-gh-release`). Concurrency: `group: release-${{ github.ref_name }}`, `cancel-in-progress: false` (a release must finish even if a second tag lands). Job-level `env` binds the three `WINDOWS_CERT_*` secrets so step-level `if:` conditions can test them.
2. Job `release-windows` on `windows-latest`, `timeout-minutes: 35` (~3x observed cold-build time, so a hung step fails with its partial log instead of burning an hour — see evidence below). Steps, in order:
   - Resolve the tag (push ref or the dispatch input), then `actions/checkout@v4` with `fetch-depth: 0`, then `git rev-parse --verify refs/tags/<tag>` so a typo fails fast.
   - `dtolnay/rust-toolchain@1.88.0` with `components: clippy, rustfmt`, `actions/setup-node@v4` with `node-version: 24`, `Swatinem/rust-cache@v2` — the same setup as the smoke matrix.
   - Import Authenticode certificate: decode `WINDOWS_CERT_PFX` (base64) to `$env:RUNNER_TEMP\dms-cert.pfx`, `Import-PfxCertificate` into `Cert:\CurrentUser\My` with `Exportable:$false`, delete the pfx, and select the cert by the pinned SHA-1 thumbprint (production PFX bundles carry the full chain). **No Root-store self-trust**: writing to `Cert:\CurrentUser\Root` blocks indefinitely on headless runners (evidence below). If the secrets are missing, the step warns and continues unsigned (dry-run path).
   - `tauri-apps/tauri-action@v1` in **build-only mode** (`projectPath: crates/dms-desktop`, `args: --bundles nsis`, `uploadWorkflowArtifacts: true`, no `tagName`/`releaseName`/`releaseId` — the action's documented way to build without touching the release) so the unsigned exe never reaches GitHub.
   - Sign installer with `signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /sha1 <thumbprint>` (Windows SDK on the runner; searched under both Kits roots), then verify via `Get-AuthenticodeSignature` (signer thumbprint must match the pinned one; accepted chain statuses `Valid` — CA-issued cert — and the untrusted-root buckets, which differ by runtime: `NotTrusted` on Windows PowerShell 5.1, `UnknownError` with the untrusted-root status message on pwsh 7 / .NET 8. `signtool verify /pa` is avoided because it chains against the trust store).
   - Compute the `<installer>.sha256` sidecar with `Get-FileHash` (lowercase hex, two spaces, filename).
   - `softprops/action-gh-release@v2` creates the release for the tag and uploads the signed `.exe` plus `.sha256` (`fail_on_unmatched_files: true`, `draft: true`, `prerelease` when the tag contains `-`), so the operator reviews and publishes the draft in the GitHub UI.
3. Signing is deliberately done with `signtool` directly rather than `tauri sign` / `bundle.windows.signCommand`: tauri 2.11.5 reads Windows signing config only from `tauri.conf.json` (no env-var override, verified in `crates/tauri-bundler/src/bundle/settings.rs`), so a signCommand would need a committed config fragment; direct `signtool` keeps the thumbprint in CI secrets only.
4. Required CI secrets (one-time operator action, recorded in this CHG): `WINDOWS_CERT_PFX` (base64), `WINDOWS_CERT_PFX_PASSWORD`, `WINDOWS_CERT_THUMBPRINT` (SHA-1, no spaces). Without them the workflow still publishes, but with an unsigned installer and a warning — the first run is the operator setting them up.
5. The first release tag is `v0.1.0-installer-preview.1` (matches `v*`, triggers `prerelease: true`). The release body links this CHG; the release stays a draft until the operator publishes it in the GitHub UI.
6. Recovery: a failed run leaves a draft release with missing assets. `gh release delete <tag> --cleanup` (or the GitHub UI) removes it; the next push to the same tag re-runs cleanly.

## Phase 2 — README install story + ADR-0027 + CAP-0005 update

**Goal:** The public-facing install story matches reality, and the design decision is recorded so future maintainers know why this exact distribution path was chosen.

Steps:

1. Add a "Installing the Windows app" section to `README.md` immediately after the existing "Windows setup" dev section. The new section says: download the latest release from `https://github.com/videoclinic/dms/releases`, pick the `DMS_Desktop_<version>_x64-setup.exe` asset, verify the SHA-256 against the `.sha256` sidecar, run the installer, approve the SmartScreen prompt (EV cert removes this; standard cert shows it once until enough installs accrue), launch "DMS Desktop" from the Start Menu, open an existing workspace.
2. Add ADR-0027 to `docs/design-decisions.md` (after ADR-0026): "Windows installer distribution = signed NSIS, published to GitHub Releases by a tag-driven `release-windows.yml` workflow." Justify: Tauri NSIS is the bundler's only first-party Windows installer; GitHub Releases is the public channel the operator requested; CI is the only build environment with the signing cert. Note: macOS and Linux distribution are out of scope of this ADR (future ADRs if/when they become a release).
3. Update CAP-0005 outcome 5: "Platform packaging produces installable artifacts for Windows and macOS (exact installer formats chosen at implementation)." Tighten to reference the release workflow and the NSIS artifact naming. Do NOT flip CAP-0005 to `implemented` — the other 16 outcomes are still pending; this CHG only closes the packaging/distribution sub-outcome.
4. Run `cargo test --workspace` and `node --test crates/dms-desktop/ui/app.test.mjs`; both exit 0 (the doc changes don't touch code, but the gate is cheap insurance against accidental reformat of the changelog front matter that the lefthook `docs` hook would catch anyway).

## Phase 3 — Smoke integration: no regression to the existing matrix

**Goal:** The new release workflow does not silently break the existing push/PR smoke.

Steps:

1. Confirm `.github/workflows/desktop-platform-smoke.yml` is unchanged (this CHG adds a new file, not edits the existing one).
2. Push a non-tag commit on a branch and confirm the smoke matrix is still green on all three OSes; the release workflow is NOT triggered by a branch push.
3. `gh workflow list` shows both `Desktop platform smoke` and `Release Windows installer` as active.

## Phase 4 — Records closeout

**Goal:** Move this CHG to `archive/`, status `done`, refresh the active index.

Steps:

1. Move `docs/changes/active/CHG-0024-windows-nsis-installer-release.md` to `docs/changes/archive/CHG-0024-windows-nsis-installer-release.md`.
2. Set `Status: done — closed YYYY-MM-DD` in the moved file; record the run id, tag, installer size, and SHA-256 hash under the phase 1 evidence.
3. In `docs/changes/README.md`, clear the "Active" table row (already empty before this CHG), and add an archive entry: `| [CHG-0024](archive/CHG-0024-windows-nsis-installer-release.md) | Windows NSIS installer and signed GitHub Release | done | CAP-0005 |`.

## Out of scope

- macOS `.dmg` release workflow — separate ADR when the operator wants it; the existing smoke matrix already covers DMG packaging as a smoke only.
- Linux deb/rpm/AppImage packaging and distribution — Tauri 2.11 supports it, but there is no `dms://` registration story in those formats and the operator has not asked.
- Tauri auto-updater — for an ISO 27001 tool, full-installer reinstall is the safer default. Revisit if/when release cadence justifies it.
- Code-signing timestamping via a private timestamp server (TSP/RFC 3161) — DigiCert's free public TSA is the default; swap to a private TSA only if the operator's PKI requires it.
- `bundle.active: true` in `tauri.conf.json` — the release workflow passes `--bundles nsis` explicitly, mirroring the smoke matrix. Dev `cargo run -p dms-desktop` stays bundler-free.
- Multiple Windows targets (`x86_64-pc-windows-gnu`, `aarch64-pc-windows-msvc`) — Tauri Windows desktop is x64 only for v1; revisit if/when ARM64 is requested.
