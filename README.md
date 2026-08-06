# DMS

Concept and implementation record for a local-first desktop document-management
system (DMS) for operator-maintained ISO 27001 document control.

## Current state

This repository is at the **product-record and wireframe stage**. It contains no
application source, packages, build pipeline, or runnable desktop app yet.

| Surface | Current state |
| --- | --- |
| Product contract | 20 capability records (`CAP-0001` … `CAP-0020`), all `not implemented` |
| Implementation plan | `CHG-0001` is active; product-record and architecture bootstrap is complete, app skeleton work has not started |
| Architecture | Tauri 2 design for Windows and macOS with no application database or required Git workflow |
| Operator UI | Static HTML and PNG wireframes for every capability; design references only |

The records describe intended behaviour, not released functionality. Code and
executable tests will be the proof of implementation when development begins.

## Intended product

DMS will keep editable Microsoft Office drafts under an operator-managed **edit
root** and write immutable, versioned PDFs under a separate **publish root**.
The app will mirror edit-relative directories on release and store workspace
metadata in `<edit-root>/.dms/`.

Planned control model:

- Tauri 2 desktop application for Windows and macOS.
- Folder-dominant, Windows Explorer-like controlled-library workspace with
  persistent tree navigation, breadcrumbs, Back/Forward/Up, and a source-file
  identity distinct from DMS-managed document-control data.
- Application-driven Office-to-PDF release through host-installed Microsoft
  Office. First release is `V1.0`; cosmetic changes increment the minor version,
  while substantive or uncertain changes increment the major version.
- Released PDFs use
  `<stem>_V<major>.<minor>_<confidentiality-type-id>.pdf` and receive SHA-256
  integrity checksums.
- Local approval workflow with revision-bound evidence, tamper-evident event
  hashes, inherited editor/approver routing, and SMTP or `mailto:` notifications
  that open the local app through stable document permalinks.
- Local-only workspace metadata, backups, restore support, confidentiality
  policies, periodic review, audit export, and optional consented Claude Desktop
  assistance for advisory change wording and classification.

## Deliberate boundaries

The current architecture excludes a cloud database, multi-tenant backend,
mandatory Git-based version control, SharePoint/Graph synchronization, bundled
Office, cloud PDF conversion, browser-based approval, directory-backed identity
proof, and digital signatures. Filesystem permissions remain the access-control
boundary.

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

There is no setup or run command yet because the application skeleton has not
been created. Development resumes from `CHG-0001`, starting with the Tauri 2
skeleton and source-tree DOX contract. Do not promote a capability from `not
implemented` until executable tests prove its outcomes.
