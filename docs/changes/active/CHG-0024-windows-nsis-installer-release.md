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
| 1 | `release-windows.yml` tag-driven workflow + first tagged release | done (2026-08-18) | Workflow file committed; tag `v0.1.0` published (run 32183590595); assets `DMS_Desktop_0.1.0_x64-setup.exe` (4.87 MB, sha256 `ed2ab70c…b91b59`) and `.sha256` sidecar; operator confirmed install + `dms://` URI on a real Windows host |
| 2 | README install story + ADR-0027 + CAP-0005 update | done (2026-08-18) | README Windows install section present; ADR-0027 in `docs/design-decisions.md`; CAP-0005 outcome 5 references the workflow; `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/app.test.mjs` all green |
| 3 | Smoke integration: release workflow runs do not regress the existing matrix | done (2026-08-18) | Non-tag push `f5ecc8e` triggered `Desktop platform smoke` runs 32138172649 + 32138171263, both `success`; zero `Release Windows installer` runs from the same push; `gh workflow list` shows both workflows active; `desktop-platform-smoke.yml` unmodified by this commit |
| 4 | Records closeout: archive this CHG, refresh `docs/changes/README.md` | pending | CHG moved to `archive/`, status `done`, README active index updated, archive entry present |
| 5 | Winget submission kit: `Build-WingetKit.ps1` + workflow step + operator runbook | in-progress | `scripts/winget/Build-WingetKit.ps1` renders header-bearing, recommended ManifestVersion 1.12.0 winget-pkgs manifests (version, installer, defaultLocale); workflow's new `Build winget submission kit` step calls the script on every stable release and uploads the bundle as a `winget-bundle` job artifact; `docs/winget-submission.md` documents the `wingetcreate submit` flow; prerelease tags skip the step. The workflow does not hold a maintainer-fork write credential, so the operator reviews and submits the generated manifest-only, one-version PR. Release source is always the resolved tag: `actions/checkout` uses `ref: ${{ steps.tag.outputs.name }}` so tag pushes and manual dispatches cannot build different commits. Gate: rerun `v0.1.0` after moving its stale tag forward to the corrected release workflow, then confirm the artifact contains the three manifests + `SUBMIT.md` and the published release remains signed. |

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
- Run 32181331321 **first fully-green run in 9 min 23 s**: import 2 s, build OK, sign OK, `Get-AuthenticodeSignature` accepted (`status: UnknownError` with the untrusted-root message — the new bucket logic), SHA-256 sidecar emitted (`cecc905b…  DMS Desktop_0.1.0_x64-setup.exe`), draft prerelease created. But the published asset is **`DMS.Desktop_0.1.0_x64-setup.exe`** (dot), not the `DMS_Desktop_<version>_x64-setup.exe` the README + ADR-0027 contract promise. Cause chain: tauri 2's NSIS bundler uses `productName` verbatim in the on-disk name (DMS Desktop → `DMS Desktop_0.1.0_x64-setup.exe`), and GitHub replaces spaces with dots at asset upload time (community #60449, gh-cli #10585). tauri issue #13893 + #13999 confirm no installer-name override exists upstream. Fix: new `Normalize installer filename` step between build and sign that renames space→underscore in the NSIS output dir, then asserts the contracted `DMS_Desktop_<version>_x64-setup.exe` exists before sign — so the sign/hash/publish steps all see one stable name, and any future tauri naming change fails the gate instead of shipping a surprise. The orphan run-11 draft (with the wrong-name asset) was deleted before run 12 so the next successful run replaces it cleanly.
- Run 32183590595 **second green run in 3 min 19 s**: the `Normalize installer filename` step renamed `DMS Desktop_0.1.0_x64-setup.exe` → `DMS_Desktop_0.1.0_x64-setup.exe`, the positive gate fired (`Installer ready: DMS_Desktop_0.1.0_x64-setup.exe`), `signtool sign` accepted the normalized path, the sidecar recorded `ed2ab70c…  DMS_Desktop_0.1.0_x64-setup.exe`. The draft prerelease carried the contracted asset name at last. The orphan preview tag `v0.1.0-installer-preview.1` was then re-pointed to `v0.1.0` via `gh release edit --tag v0.1.0 --draft=false --prerelease=false`, the preview tag was deleted, and the result is **the first published release**: `DMS Desktop v0.1.0` with `DMS_Desktop_0.1.0_x64-setup.exe` + `.sha256` (4.87 MB signed exe, sha256 `ed2ab70ce12ca43e6063ba450cf9241e334f20aea327c8a44bc0ed557bc91b59`).

## Phase 1 — `release-windows.yml` + first tagged release

**Goal:** A `v*` tag push builds, signs, and publishes the NSIS installer to a GitHub Release; the operator can install DMS on Windows by downloading and running that signed exe.

Steps:

1. Add `.github/workflows/release-windows.yml`. Triggers: `push: tags: ['v*']` and `workflow_dispatch` (manual release of an existing tag). Permissions: `contents: write` (for `softprops/action-gh-release`). Concurrency: `group: release-${{ github.ref_name }}`, `cancel-in-progress: false` (a release must finish even if a second tag lands). Job-level `env` binds the three `WINDOWS_CERT_*` secrets so step-level `if:` conditions can test them.
2. Job `release-windows` on `windows-latest`, `timeout-minutes: 35` (~3x observed cold-build time, so a hung step fails with its partial log instead of burning an hour — see evidence below). Steps, in order:
   - Resolve the tag (push ref or the dispatch input), then `actions/checkout@v4` with `fetch-depth: 0`, then `git rev-parse --verify refs/tags/<tag>` so a typo fails fast.
   - `dtolnay/rust-toolchain@1.88.0` with `components: clippy, rustfmt`, `actions/setup-node@v4` with `node-version: 24`, `Swatinem/rust-cache@v2` — the same setup as the smoke matrix.
   - Import Authenticode certificate: decode `WINDOWS_CERT_PFX` (base64) to `$env:RUNNER_TEMP\dms-cert.pfx`, `Import-PfxCertificate` into `Cert:\CurrentUser\My` with `Exportable:$false`, delete the pfx, and select the cert by the pinned SHA-1 thumbprint (production PFX bundles carry the full chain). **No Root-store self-trust**: writing to `Cert:\CurrentUser\Root` blocks indefinitely on headless runners (evidence below). If the secrets are missing, the step warns and continues unsigned (dry-run path).
   - `tauri-apps/tauri-action@v1` in **build-only mode** (`projectPath: crates/dms-desktop`, `args: --bundles nsis`, `uploadWorkflowArtifacts: true`, no `tagName`/`releaseName`/`releaseId` — the action's documented way to build without touching the release) so the unsigned exe never reaches GitHub.
   - Normalize installer filename: tauri 2 names the NSIS installer from `productName` verbatim (DMS Desktop → `DMS Desktop_0.1.0_x64-setup.exe`), and GitHub replaces spaces with dots in uploaded asset names, which would break the documented `DMS_Desktop_<version>_x64-setup.exe` contract. The step renames space→underscore in the NSIS output dir, then asserts the contracted name exists (positive gate; a future tauri naming change fails the gate instead of shipping a surprise).
   - Sign installer with `signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /sha1 <thumbprint>` (Windows SDK on the runner; searched under both Kits roots), then verify via `Get-AuthenticodeSignature` (signer thumbprint must match the pinned one; accepted chain statuses `Valid` — CA-issued cert — and the untrusted-root buckets, which differ by runtime: `NotTrusted` on Windows PowerShell 5.1, `UnknownError` with the untrusted-root status message on pwsh 7 / .NET 8. `signtool verify /pa` is avoided because it chains against the trust store).
   - Compute the `<installer>.sha256` sidecar with `Get-FileHash` (lowercase hex, two spaces, filename).
   - `softprops/action-gh-release@v2` creates the release for the tag and uploads the signed `.exe` plus `.sha256` (`fail_on_unmatched_files: true`, `draft: true`, `prerelease` when the tag contains `-`), so the operator reviews and publishes the draft in the GitHub UI.
3. Signing is deliberately done with `signtool` directly rather than `tauri sign` / `bundle.windows.signCommand`: tauri 2.11.5 reads Windows signing config only from `tauri.conf.json` (no env-var override, verified in `crates/tauri-bundler/src/bundle/settings.rs`), so a signCommand would need a committed config fragment; direct `signtool` keeps the thumbprint in CI secrets only.
4. Required CI secrets (one-time operator action, recorded in this CHG): `WINDOWS_CERT_PFX` (base64), `WINDOWS_CERT_PFX_PASSWORD`, `WINDOWS_CERT_THUMBPRINT` (SHA-1, no spaces). Without them the workflow still publishes, but with an unsigned installer and a warning — the first run is the operator setting them up.
5. The first release tag is `v0.1.0-installer-preview.1` (matches `v*`, triggers `prerelease: true`). The release body links this CHG; the release stays a draft until the operator publishes it in the GitHub UI.
6. Recovery: a failed run leaves a draft release with missing assets. `gh release delete <tag> --cleanup` (or the GitHub UI) removes it; the next push to the same tag re-runs cleanly.

## Phase 5 — Winget submission kit

**Goal:** Every stable release ships a ready-to-submit bundle of winget-pkgs manifests so the operator can publish to `winget install` in one `wingetcreate submit` call — without granting the release workflow write access to the operator's `winget-pkgs` fork.

Steps:

1. Add `scripts/winget/Build-WingetKit.ps1` — a `pwsh` script that takes `-Version`, `-InstallerUrl`, `-InstallerSha256`, `-OutputDir` (plus optional metadata overrides) and writes the three winget-pkgs multi-file manifests to `manifests/<first-letter>/<Publisher>/<PackageName>/<Version>/`:
   - `<packageId>.yaml` (version file, `ManifestType: version`)
   - `<packageId>.locale.en-US.yaml` (`ManifestType: defaultLocale`, with `PackageName: DMS Desktop`, `License: MIT`, `Moniker: dms-desktop`, `Tags`, `PublisherUrl`, `PrivacyUrl`, `LicenseUrl`, `ReleaseNotesUrl`)
   - `<packageId>.installer.yaml` (`ManifestType: installer`, `InstallerType: nullsoft`, `Platform: [Windows.Desktop]`, `MinimumOSVersion: 10.0.17763.0`, `InstallModes: [silent, silentWithProgress]`, `Installers[0].Architecture: x64`, plus the real `InstallerUrl` + `InstallerSha256`).
   - `SUBMIT.md` — the one-pager the operator follows after downloading the artifact.
   The package id is `Videoclinic.DMSDesktop` (path drops the space, `PackageName` keeps it). Each generated file starts with the required schema header and declares `ManifestVersion: 1.12.0`, the upstream PR template's recommended schema. Sanity checks in the script: publisher id matches `^[A-Za-z][A-Za-z0-9.-]{0,31}$`, sha-256 is 64 hex chars, all three required fields per file present (verified locally against the JSON schemas at `winget-cli/schemas/JSON/manifests/v1.12.0/manifest.*.json`).
2. Add a `Build winget submission kit` step to `.github/workflows/release-windows.yml` after the publish step, gated on `!contains(steps.tag.outputs.name, '-')` so prerelease tags (e.g. `v0.1.0-installer-preview.1`) skip it — winget only takes stable versions. The step:
   - reads the just-computed `*.sha256` sidecar and the version from `crates/dms-desktop/tauri.conf.json`,
   - builds the canonical `https://github.com/<repo>/releases/download/<tag>/<asset>` URL,
   - calls the script with `$env:RUNNER_TEMP/winget-bundle` as the output dir,
   - writes a `## Winget submission kit` block to `$env:GITHUB_STEP_SUMMARY` so the operator sees the package id, version, URL, and SHA-256 right on the run page.
3. Add an `Upload winget submission kit` step (`actions/upload-artifact@v4`, `name: winget-bundle`, `if-no-files-found: error`) so the bundle is downloadable as a run artifact.
4. Add `docs/winget-submission.md` — the operator runbook: package identity, why no auto-PR, three-step submit flow, post-merge verification commands.
5. No auto-PR to `microsoft/winget-pkgs`. The release workflow has no write credential for the operator's fork, and the upstream repository requires one package version with manifest-only changes per PR. Generating the bundle from the successful run's exact installer URL and SHA-256 keeps `wingetcreate submit` to a one-liner while making the operator review the public submission before it is opened.
6. The corrected v0.1.0 tag reruns the workflow and produces the `winget-bundle` Actions artifact; the same bundle is also attached to the v0.1.0 GitHub Release. Later stable tags receive a fresh artifact automatically.

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
