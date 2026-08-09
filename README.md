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
  drafts; Markdown renders locally through a CommonMark HTML print shell and
  native WebView PDF APIs, with header/footer chrome from the release context.
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

## Repository guide

- [Architecture](docs/architecture.md) — runtime shape, roots, trust boundary,
  and non-goals.
- [Design decisions](docs/design-decisions.md) — durable implementation choices.
- [Privacy](docs/privacy.md) — data classes and local-processing constraints.
- [Product capabilities](docs/product/README.md) — current capability contracts
  and wireframe index.
- [Active change record](docs/changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
  — implementation scope, phases, and verification gates.

## Development status

Install Rust 1.88 or newer. The desktop build also needs the platform prerequisites
listed by [Tauri](https://v2.tauri.app/start/prerequisites/). Then run:

```sh
cargo test --workspace
cargo run -p dms-cli -- --help
node --test crates/dms-desktop/ui/app.test.mjs
cargo run -p dms-desktop
```

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
