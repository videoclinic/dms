# Product capabilities

Current user-visible behaviour lives in `capabilities/CAP-*.md`.
Code and executable tests are the final proof of behaviour; a CAP without
linked tests is a coverage gap, not implemented proof.

## Rules

- One CAP file per distinct capability boundary.
- State present-tense, falsifiable outcomes only.
- Link architecture, privacy, and design decisions instead of copying them.
- Update a CAP only when a material outcome changes.
- Planned-but-unimplemented capabilities use `Status: not implemented` and do
  not claim runtime behaviour.

## Index

| ID | Title | Status |
| --- | --- | --- |
| [CAP-0001](capabilities/CAP-0001-local-folder-dms.md) | Dual-root local DMS metadata store | not implemented |
| [CAP-0002](capabilities/CAP-0002-document-lifecycle.md) | Draft → approval → versioned PDF release | not implemented |
| [CAP-0003](capabilities/CAP-0003-document-notes.md) | Document notes | not implemented |
| [CAP-0004](capabilities/CAP-0004-release-integrity.md) | Released PDF checksum integrity | not implemented |
| [CAP-0005](capabilities/CAP-0005-desktop-shell.md) | Tauri desktop shell (Windows and macOS) | not implemented |
| [CAP-0006](capabilities/CAP-0006-library-explorer.md) | Folder-first controlled library explorer | not implemented |
| [CAP-0007](capabilities/CAP-0007-draft-pdf-export.md) | Application-driven source draft → PDF export | not implemented |
| [CAP-0008](capabilities/CAP-0008-confidentiality-classification.md) | Inherited document confidentiality classification | not implemented |
| [CAP-0009](capabilities/CAP-0009-release-editor.md) | Release editor (host source editor) | not implemented |
| [CAP-0010](capabilities/CAP-0010-notification-transport.md) | Notification transport (SMTP or host mail handler) | not implemented |
| [CAP-0011](capabilities/CAP-0011-approval-evidence.md) | Approval evidence (changelog and decision comments, chain) | not implemented |
| [CAP-0012](capabilities/CAP-0012-audit-export.md) | Audit and report export | not implemented |
| [CAP-0013](capabilities/CAP-0013-library-maintenance.md) | Library maintenance beyond add/remove | not implemented |
| [CAP-0014](capabilities/CAP-0014-workspace-integrity.md) | Workspace integrity (locks, backups, restore) | not implemented |
| [CAP-0015](capabilities/CAP-0015-document-control-data.md) | Document control data and revision cycle | not implemented |
| [CAP-0016](capabilities/CAP-0016-publish-tree-maintenance.md) | Publish-tree and release-set maintenance | not implemented |
| [CAP-0017](capabilities/CAP-0017-periodic-document-review.md) | Periodic review of current released documents | not implemented |
| [CAP-0018](capabilities/CAP-0018-claude-desktop-change-assistance.md) | Optional Claude Desktop change-comment assistance | not implemented |
| [CAP-0019](capabilities/CAP-0019-inherited-workflow-role-routing.md) | Inherited editor and approver routing | not implemented |
| [CAP-0020](capabilities/CAP-0020-document-permalinks.md) | Document permalinks (stable local-app URI) | not implemented |
| [CAP-0021](capabilities/CAP-0021-microsoft-entra-workflow-identity.md) | Microsoft Entra workflow identity source | not implemented |
| [CAP-0022](capabilities/CAP-0022-headless-core-cli.md) | Headless DMS core CLI | implemented |

CAP-0022 is a Tauri-independent initial core surface. It does not promote the
pending desktop, lifecycle, export, or workflow CAPs; the future Tauri shell
uses the same `dms-core` library.

## Wireframes

Static operator-surface wireframes (shadcn-admin 2.2.0 visual base):
[`wireframes/index.html`](wireframes/index.html) ·
[`wireframes/manifest.json`](wireframes/manifest.json) ·
PNG exports under `wireframes/exports/`.
Each CAP links its HTML + PNG under **Links**.

## Related

- Progress authority: [`../changes/README.md`](../changes/README.md)
- Architecture: [`../architecture.md`](../architecture.md)
- Privacy: [`../privacy.md`](../privacy.md)
- Design decisions: [`../design-decisions.md`](../design-decisions.md)
