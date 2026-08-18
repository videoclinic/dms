# DMS

Concept and implementation record for a local-first desktop document-management
system (DMS) for operator-maintained ISO 27001 document control.

## Current state

This repository has a runnable headless core CLI and an initial Tauri 2 desktop
shell. Product records and wireframes define the later workflow slices.

| Surface | Current state |
| --- | --- |
| Product contract | 22 capability records; `CAP-0022` is implemented, while full desktop and later workflow CAPs remain pending |
| Implementation plan | `CHG-0001` phase 1 provides the shared core, CLI, and desktop shell; later domain phases remain |
| Architecture | Rust workspace with a standalone CLI and Tauri 2 desktop adapter; no application database or required Git workflow |
| Core automation | `dms` CLI for local workspace initialization, document registration/control data, and notes |
| Operator UI | Initial Tauri shell for workspace open, foldable navigation, session panes, and per-user saved views; static wireframes remain design references for later capabilities |

CAP-0022 is proven by executable tests. The remaining CAPs describe intended
desktop and workflow behaviour, not released functionality.

## Intended product

DMS will keep editable Microsoft Office and Markdown (`.md`) source drafts under
an operator-managed **edit root** and write immutable, versioned PDFs under a
separate **publish root**. The app will mirror edit-relative directories on
release and store workspace metadata in `<edit-root>/.dms/`.

Planned control model:

- Tauri 2 desktop application for Windows and macOS.
- Tauri-independent `dms-core` Rust library and a standalone `dms` CLI for the
  initial local metadata core; the desktop shell calls the same library.
- Folder-dominant, Windows Explorer-like controlled-library workspace with
  persistent tree navigation, breadcrumbs, Back/Forward/Up, and a source-file
  identity distinct from DMS-managed document-control data.
- Application-driven PDF release: host-installed Microsoft Office exports Office
  drafts; Markdown is assembled locally into a temporary DOCX from the workspace
  Word template, then exported through installed Word with controlled fields
  from the release context.
  First release is `V1.0`. For every later release, the editor records a required
  changelog and proposes the next minor, the next major, or a validated manual
  target version. `V1.0` and major-version candidates require approval; a minor
  candidate releases directly after validation and notifies its effective
  approver after publication. A candidate becomes a released version only after
  required approval and atomic PDF export; unsuccessful reviews keep their
  evidence but do not occupy that target version.
- Released PDFs use
  `<stem>_V<major>.<minor>_<confidentiality-type-id>.pdf` and receive SHA-256
  integrity checksums.
- Local approval workflow with revision-bound evidence, tamper-evident event
  hashes, inherited editor/approver routing from a Microsoft Entra workspace
  group, interactive Entra identity verification for decisions, and SMTP or
  `mailto:` notifications that open the local app through stable document
  permalinks.
- Local-only workspace metadata, backups, restore support, confidentiality
  policies, periodic review, audit export, and optional consented Claude Desktop
  assistance for advisory changelog wording and target-version mode.

## Deliberate boundaries

The current architecture excludes a cloud database, multi-tenant backend,
mandatory Git-based version control, SharePoint/OneDrive document-content
synchronization, bundled Office, cloud PDF conversion, browser-based approval,
and digital signatures. Microsoft Graph is limited to Microsoft Entra workflow
identity resolution and verification; filesystem permissions remain the
source-file access-control boundary.

### Installing the Windows app

The signed NSIS installer is published as a GitHub Release on every
`v*` tag push.

1. Open the [releases page](https://github.com/videoclinic/dms/releases)
   and pick the latest tag (e.g. `v0.1.0`).
2. Download `DMS_Desktop_<version>_x64-setup.exe` **and** the matching
   `*.sha256` sidecar.
3. Verify the SHA-256 of the installer matches the sidecar:
   ```powershell
   Get-FileHash .\DMS_Desktop_<version>_x64-setup.exe -Algorithm SHA256
   Get-Content .\DMS_Desktop_<version>_x64-setup.exe.sha256
   ```
4. Double-click the installer and follow the standard NSIS prompts.
   The installer writes the `dms://` URL Protocol keys under
   `HKCU\Software\Classes\dms` (per-user) so document permalinks from
   notification emails open back into the installed app.
5. Launch **DMS Desktop** from the Start Menu, then open an existing
   workspace through the setup screen.

The Windows SmartScreen prompt is expected on first launch until the
Authenticode certificate accrues enough installs; an EV code-signing
certificate removes it entirely. See
[CHG-0024](docs/changes/active/CHG-0024-windows-nsis-installer-release.md)
for the full distribution contract.

The "Windows setup" section above is the **developer** path (build from
source with the MSVC toolchain); this section is the **end-user** path
(download and run the signed installer).

## Repository guide

- [Architecture](docs/architecture.md) — runtime shape, roots, trust boundary,
  and non-goals.
- [Design decisions](docs/design-decisions.md) — durable implementation choices.
- [Privacy](docs/privacy.md) — data classes and local-processing constraints.
- [Microsoft Entra application setup](docs/entra-client-setup.md) — create the
  DMS public-client registration, grant delegated Graph consent, and configure
  DMS with its client, tenant, and group IDs.
- [Product capabilities](docs/product/README.md) — current capability contracts
  and wireframe index.
- [Library membership and obsolescence](docs/library-membership-and-obsolescence.md)
  — why **Mark obsolete** and **Unregister** are different actions.
- [Bootstrap implementation receipt](docs/changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
  — completed implementation scope, phases, and verification evidence.

## Development status

The committed [`rust-toolchain.toml`](rust-toolchain.toml) selects Rust
**1.88.0** and supplies `clippy` and `rustfmt`; `Cargo.toml` retains Rust 1.88
as the minimum supported version. Use a native Windows toolchain for Windows
desktop builds and tests. Git Bash/MSYS does not provide the required MSVC Rust
environment.

### Windows setup

1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   and select **Desktop development with C++**. Keep the MSVC x64/x86 build
   tools and a Windows 10 or 11 SDK selected.
2. Install the [Microsoft Edge WebView2 Evergreen Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).
3. In PowerShell, install Rustup and Node.js LTS, then restart the terminal so
   `%USERPROFILE%\\.cargo\\bin` is on `PATH`:

   ```powershell
   winget install --id Rustlang.Rustup
   winget install --id OpenJS.NodeJS.LTS
   ```

   In the repository, Rustup automatically installs/selects the committed
   `1.88.0-x86_64-pc-windows-msvc` toolchain. Confirm the required commands
   resolve before testing:

   ```powershell
   cargo --version
   rustc --version
   rustup component list --installed
   node --version
   ```

4. For the external Office-release smoke only, install and license the
   appropriate Microsoft Office desktop applications. Office is not needed for
   the ordinary Rust and frontend test suites.

Then run the local checks from a native Windows terminal:

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p dms-cli -- --help
node --test crates/dms-desktop/ui/app.test.mjs
cargo run -p dms-desktop
```

WSL can run the headless Rust and frontend checks when its own Linux Rust
toolchain is installed, but it is not a substitute for native Windows WebView2,
Windows packaging, or Office integration validation. Linux development has
separate [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#linux).

### Ubuntu on WSL2 setup

Use an Ubuntu **WSL2** distribution and keep the checkout in its Linux
filesystem (for example, `/home/<user>/src/dms`), not under `/mnt/c`. From an
elevated Windows PowerShell, install or confirm WSL2 and Ubuntu if needed:

```powershell
wsl --install -d Ubuntu
wsl --set-default-version 2
```

Open Ubuntu (`wsl -d Ubuntu`) and install the compiler, Tauri development
libraries, Git, and Node.js. The Ubuntu packages provide a supported Node LTS
line; the repository has no separate npm dependency installation step.

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  file \
  git \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  nodejs \
  npm \
  pkg-config \
  wget
```

Install Rust through Rustup rather than Ubuntu's `rustc`/`cargo` packages, then
restart the Ubuntu shell. When you enter this checkout, the committed
`rust-toolchain.toml` selects Rust 1.88.0 with `clippy` and `rustfmt`.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
exit
```

Reopen Ubuntu, clone or enter the checkout, and verify the local toolchain:

```bash
cd ~/src/dms
cargo --version
rustc --version
rustup component list --installed
node --version
npm --version
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
node --test crates/dms-desktop/ui/app.test.mjs
```

`cargo run -p dms-desktop` can be used only when WSLg is available and the
Linux desktop prerequisites above are installed. It validates the Linux Tauri
adapter, not the native Windows application. Run native Windows desktop,
WebView2, packaging, and Office integration checks from the Windows setup
above.

Initialize an explicit workspace and register a source draft:

```sh
cargo run -p dms-cli -- workspace init \
  --edit-root /path/to/edit-root --publish-root /path/to/publish-root --confirm
cargo run -p dms-cli -- document add \
  --edit-root /path/to/edit-root --path /path/to/edit-root/Policy.md
```

Use `--json` for structured command results. The desktop shell opens an existing
workspace through `dms-core`; release lifecycle, export, approval, and workflow
features remain pending in `CHG-0001`.

## License

MIT © 2026 Videoclinic. See [`LICENSE`](LICENSE).
