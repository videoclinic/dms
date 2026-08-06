# Architecture

## Purpose

Desktop application for operator-maintained document versioning and approval
aligned with ISO 27001 control needs for documented information. Required
runtime targets are **Windows and macOS**. Drafts remain Microsoft Office files
under an **edit root**; released artifacts are versioned PDFs under a
**publish root**, with integrity checksums.

## Runtime shape

| Layer | Responsibility |
| --- | --- |
| Tauri 2 shell (Rust) | Windowing, filesystem access, checksums, path mapping, Office export orchestration, OS integration, registered local-app URI handler (document permalinks) |
| Frontend (web UI in WebView) | Foldable left menu with hamburger when collapsed; open-activity panes/tabs as quicklinks; library directory navigator (folder tree, list metadata, selection pane with CAP-0015 master data and single/batch actions), add/remove control, lifecycle, change commentary, approval, release/verify, confidentiality and workflow-role policies, audit export, publish history, copy/resolve document permalinks |
| Microsoft Office (host-installed) | PDF export engine invoked by the app on release (Word/Excel/PowerPoint as applicable) |
| Claude Desktop (optional host app) | Operator-mediated, consented handoff for advisory change classification and changelog wording; not a callable local model or lifecycle authority |
| `<edit-root>/.dms/` | Roots config, library registry, workflow-person roster + SMTP settings (no secrets), folder confidentiality and workflow-role policies, notes, approval/release history, evidence hashes, checksums, advisory lock |
| Edit root tree | Operator-edited Microsoft Office drafts (library members are a subset) |
| Publish root tree | Versioned released PDFs in a directory tree mirrored from edit-relative paths |

No application database server and no mandatory git repository.

## Dual-root path model

```
edit root                          publish root
─────────                          ────────────
policies/HR/Handbook.docx   →      policies/HR/Handbook_V1.0_restricted.pdf
policies/HR/Handbook.docx   →      policies/HR/Handbook_V2.0_restricted.pdf   (later substantive release)
```

- `.dms` stores absolute edit root + absolute publish root once per workspace.
- Each library document has a **stable document ID**; the draft **relative path**
  under the edit root is the current locator used for open/export path mapping.
- On release, the app: snapshots the effective confidentiality type ID, assigns
  the next version label → ensures the relative parent path exists under the
  publish root → exports PDF via installed Microsoft Office into
  `<stem>_VMAJOR.MINOR_<confidentiality-type-id>.pdf` → checksums the result.
- Application-managed version history consists of immutable release PDFs and
  their evidence. Office drafts remain mutable working copies; draft recovery
  comes from workspace backups rather than an embedded source-version store.
- This prevents “edited here, published who-knows-where” drift: publish location
  is a pure function of configured roots + relative path + version label.

## Trust and control boundary

- Operator chooses edit root and publish root (local or mapped filesystem).
- The app reads/writes within those roots and `.dms`, except explicit
  user-chosen import paths that must still resolve under the edit root for
  library membership.
- Approval and release are operator actions; the app records process state and
  enforces naming/path rules.
- An approver uses the app against the same workspace (for example, through a
  shared or mapped edit root). An approval email is a notification with a
  CAP-0020 permalink deep link (workspace ID + document ID + review target) to
  that local workspace; it is not a web approval portal. The same permalink
  scheme opens a document selection without a review target and remains valid
  across draft renames and version bumps.
- The application sends notification email through a configured SMTP relay. The
  relay password is held in the OS credential store, never in `.dms`.
- When SMTP is not configured, the desktop app opens the host's default email
  handler with a pre-filled `mailto:` URI as a fallback notification path; this
  is an outbound mail draft, not a server-issued message, and the lifecycle
  state does not enter `in_review` until the operator confirms submission.
- A library document opened from the app launches the registered Office editor
  for its draft format. The app releases the OS file handle before returning;
  it does not hold a long-running lock on the draft.
- An advisory lock file in `<edit-root>/.dms/lock` serialises metadata writes
  from this app only. It is not a multi-writer lock and never blocks read-only
  filesystem access from other tools.
- ISO 27001 support is traceable library membership, lifecycle, notes, and
  released-PDF integrity — not organizational certification.

## Out of scope (current architecture)

- Cloud database or multi-tenant backend
- Mandatory git-based version control
- Bundling Microsoft Office inside the app binary
- Cloud/server-side PDF conversion services
- SharePoint/Graph sync as a required runtime dependency
- Remote/browser approval, directory-backed identity proof, or digital signing
- Confidentiality labels as a replacement for filesystem access control
- Auto-adding every file under the edit root without operator library action
- Centralised anti-virus, DLP, or e-signature services

## Related

- Privacy: [`privacy.md`](privacy.md)
- Decisions: [`design-decisions.md`](design-decisions.md)
- Capabilities: [`product/README.md`](product/README.md)
