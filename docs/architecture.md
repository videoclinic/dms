# Architecture

## Purpose

Desktop application for operator-maintained document versioning and approval
aligned with ISO 27001 control needs for documented information. Required
runtime targets are **Windows, macOS, and Linux**. Drafts remain Microsoft Office or
Markdown (`.md`) files under an **edit root**; released artifacts are versioned
PDFs under a **publish root**, with integrity checksums.

## Runtime shape

| Layer | Responsibility |
| --- | --- |
| `dms-core` Rust library | Tauri-independent workspace metadata, library registry, document control data, notes, path validation, and future lifecycle rules shared by every application surface |
| `dms` CLI (Rust) | Headless explicit workspace operations and standalone Markdown conversion through `dms-export`; no WebView, Tauri runtime, Entra flow, or mail transport |
| `dms-export` Rust adapter | Shared controlled OOXML placeholder fill and installed-Word automation for CLI conversion and desktop release; no Tauri dependency |
| Tauri 2 shell (Rust) | Windowing, WebView IPC, format-specific PDF export orchestration, OS integration, registered local-app URI handler (workspace and document permalinks), Windows edit-root `Open in DMS.lnk` helper, and an adapter over `dms-core` |
| Frontend (web UI in WebView) | Foldable left menu with hamburger when collapsed; open-activity panes/tabs as quicklinks; one Configuration workspace with persistent Workspace, Document defaults, Workflow, and Notifications routes plus contextual secondary setup; folder-dominant Library workspace with a persistent edit-root-relative tree, Windows Explorer-like Back/Forward/Up and breadcrumb navigation, current-folder child folders + exact source-file names + controlled-document data, and a selection pane that separates filesystem-derived Source file identity from CAP-0015 DMS-managed document control data and single/batch actions; add/remove control, lifecycle, change commentary, approval, release/verify, confidentiality and workflow-role policies, audit export, publish history, copy/resolve document permalinks |
| Microsoft Office (host-installed) | PDF export engine for Office drafts and Word documents assembled from Markdown plus the workspace export template; standalone conversion retains the saved final DOCX |
| Workspace Word template | One reusable `.docx` asset under the edit root supplies styles, page setup, headers, footers, media, and controlled-field locations for every Markdown release; it is configuration, not a controlled document |
| Claude Desktop (optional host app) | Operator-mediated, consented handoff for advisory target-version mode and changelog wording; not a callable local model or lifecycle authority |
| Microsoft Entra ID + Microsoft Graph | App-global public-client and tenant configuration plus a per-workspace group supplies eligible workflow people; delegated interactive sign-in verifies review decisions; never reads or synchronizes document content. When both effective identifiers are process-environment values, startup validates the cached delegated session or starts one non-blocking OAuth device-authorization challenge; Windows policy and saved-settings-only configuration do not |
| App-global Entra configuration | Windows machine policy at `HKLM\SOFTWARE\Policies\Videoclinic\DMS` when present, otherwise OS-user `global-settings.json`; both hold only the non-secret public-client ID and tenant ID shared by local libraries |
| `<edit-root>/.dms/` | Roots config, library registry, active Markdown export-template identity/relative locator/validation digest, DMS-managed document control data, Entra group binding + read-only display cache, folder confidentiality and workflow-role policies, notes, approval/release history, evidence hashes, checksums, advisory lock |
| OS-user `global-settings.json` | Non-secret public-client/tenant IDs plus notification transport/SMTP relay settings shared by every local library opened by that DMS user |
| Edit root tree | Operator-edited Microsoft Office and Markdown source drafts (library members are a subset); on Windows, `<edit-root>/Open in DMS.lnk` is a DMS helper that carries only the workspace permalink |
| Publish root tree | Versioned released PDFs in a directory tree mirrored from edit-relative paths |

No application database server and no mandatory git repository.

## Dual-root path model

```
edit root                          publish root
─────────                          ────────────
policies/HR/Handbook.docx   →      policies/HR/Handbook_V1.0_restricted.pdf
policies/HR/Handbook.docx   →      policies/HR/Handbook_V2.0_restricted.pdf   (later substantive release)
procedures/Onboarding.md    →      procedures/Onboarding_V1.0_internal.pdf
```

- `.dms` stores absolute edit root + absolute publish root once per workspace.
- Each library document has a **stable document ID**; the draft **relative path**
  under the edit root is the current locator used for open/export path mapping.
- Source filename and relative path are filesystem-derived locator facts.
  Renaming or reassociating the draft updates that locator only; DMS-managed
  document control data and history remain keyed to the stable document ID.
- First registration of a Markdown member may seed selected flat frontmatter
  values into DMS document control, review schedule, workflow overrides, and an
  uncommitted first-release target. This is a one-time import boundary; later
  source edits, re-addition, and reassociation never update `.dms`.
- On release, the app: snapshots the effective confidentiality type ID, confirms
  the approved target version label → ensures the relative parent path exists under the
  publish root → builds one export chrome map from `.dms` → exports Office
  drafts through installed Microsoft Office, or validates Markdown frontmatter
  after projecting controlled keys from `.dms`, assembles a temporary DOCX from
  the configured workspace template and CommonMark body (frontmatter stripped
  from body; optional non-controlled frontmatter `{KEY}` variables filled),
  fills controlled fields from the release snapshot, and
  exports that DOCX through installed Word → writes
  `<stem>_VMAJOR.MINOR_<confidentiality-type-id>.pdf` → checksums the result.
- Application-managed version history consists of immutable release PDFs and
  their evidence. First Office import may separately retain at most three
  source-derived person-and-date observations bound to the exact imported draft
  SHA-256; this is unverified provenance, not version/release/workflow history
  or a source-version store. Source drafts remain mutable working copies; draft
  recovery comes from workspace backups rather than an embedded source-version
  store.
- The publish root is a storage location, not an additional lifecycle stage:
  successful release creates the released PDF. Its location is a pure function
  of configured roots + relative path + version label, preventing “edited here,
  released who-knows-where” drift.

## Trust and control boundary

- Operator chooses edit root and publish root (local or mapped filesystem).
- Standalone `dms convert markdown` is not a release operation. Its explicit
  external input tree and new output tree are disjoint from edit/publish trees
  after canonical path checks. Core reads only schema 18, 19 or current export
  configuration fields, checks the native-host config location and template
  digest/contract, and never opens, migrates, saves or locks the workspace.
  External flat frontmatter supplies title, numeric version and an enabled
  confidentiality ID solely for conversion chrome; it grants no approval or
  registered-document authority. No identity or notification adapter runs.
- Conversion discovers immediate `.md` children, or recursive children only
  with `--recursive`, excluding `.dms` and `assets` subtrees. Referenced images
  may reside anywhere within the canonical input root. A new output root with
  an existing parent is required; there is no overwrite or resume mode.
  Case-insensitive portable path collisions fail before document processing.
- Conversion runs sequentially and retains `<stem>.docx` / `<stem>.pdf` pairs
  plus `manifest.json`, all marked `conversion_only`. The shared Word adapter
  fills controlled fields, updates fields/TOC/pagination, saves DOCX and exports
  that same document. Core reads back ZIP/XML and parses PDF text before
  recording final artifact hashes. Consumed Markdown, template and embedded
  asset bytes determine input digests, even if originals subsequently change.
  Document-specific failures continue and yield a nonzero command result;
  incomplete owned artifacts are removed or named as retained diagnostics.
  `--dry-run` assembles privately without calling Word or creating output.
- Template TOC is retained by default; `--without-toc` removes only the template
  TOC structured-document tag, never a Markdown-authored contents section.
  Generated tables of every supported column count span printable body width.
  Link labels remain visible text; Markdown-relative links are not rewritten
  into working PDF navigation. Automated doubles prove boundaries, not real
  Word pagination; full-corpus Windows and macOS rendering remain host gates.
- The app reads/writes within those roots and `.dms`, except explicit
  standalone conversion inputs/outputs and user-chosen import paths that must
  still resolve under the edit root for library membership.
- A workspace Markdown export template is selected from a `.docx` under the edit
  root, has a stable template ID and relative locator in `.dms`, and is excluded
  from document lifecycle, notes, release history, and permalinks.
- Approval and release are operator actions; the app records process state and
  enforces naming/path rules. `V1.0` and every later candidate that increases
  the major component require Entra-verified approval. A minor candidate releases
  directly after validation and then notifies the effective approver of the new
  publication; delivery failure never reverses the committed release.
- An approver uses the app against the same workspace (for example, through a
  shared or mapped edit root). An approval email is a notification with a
  CAP-0020 permalink deep link (workspace ID + document ID + review target) to
  that local workspace; it is not a web approval portal. The same permalink
  scheme opens a document selection without a review target and remains valid
  across draft renames and version bumps. A workspace-only permalink opens
  `Library · /` with no document selected. On Windows, `<edit-root>/Open in
  DMS.lnk` holds that workspace URI and never a filesystem root. Resolution
  uses only registered accessible recent libraries.
- Workflow roles select individual, direct user members of the workspace's
  configured Microsoft Entra group. `.dms` records the binding tenant and group
  object IDs, group label, display cache, and role references to immutable Entra
  user object IDs. The public-client ID and app-global tenant configuration are
  outside workspace metadata. On Windows, a complete machine policy pair takes
  precedence over process environment overrides and OS-user settings; policy
  values remain read-only and are never copied to either persisted store. It does not keep an
  application-managed user roster. A group may be a Microsoft 365 group when
  its membership is exactly the intended workflow population.
- Owner, editor, and approver authority is keyed only by the tenant-scoped
  Entra object ID under the current group binding. Names and email addresses
  are refreshable display and notification data and never participate in
  identity equality. The literal `<owner>` and `<editor>` placeholders are not
  Entra identities; they appear only for routing when a successful direct
  member response contains zero eligible users, and they cannot authorize
  review, decision, notification, or release.
- The app refreshes Microsoft Graph membership when assigning a role and before
  workflow authority is applied. Cached display information is presentation
  data only. A missing, disabled, or no-longer-eligible identity leaves the
  policy unresolved and blocks new review work until rerouted.
- The desktop public client uses delegated device authorization for identity
  source setup and approver sign-in. `DMS_ENTRA_CLIENT_ID` and
  `DMS_ENTRA_TENANT_ID`, when non-empty, override the corresponding stored
  app-global value for that process and are read-only in Configuration. Invalid
  non-empty overrides fail closed. Graph access and refresh tokens live only in
  the OS credential store; the workspace persists no token, client ID, active
  actor, or client secret, and retains its tenant ID only as part of the
  group-binding authorization boundary.
- A group-bound library activation requires a cached or refreshed delegated
  credential, `/me`, and a fresh enabled direct-member result for its persisted
  tenant/group binding before its advisory lock is acquired. Missing or
  refresh-rejected credentials produce one process-only library-session device
  challenge; it exposes no token or `device_code`, does not open a browser
  automatically, and leaves the library inactive on expiry, decline, tenant
  mismatch, disabled/non-member, inaccessible-group, or service failure.
- Recording an approval decision requires interactive Microsoft Entra sign-in.
  The signed-in tenant/object ID must match the review's snapshotted effective
  approver and still be eligible in the bound group. This verifies the decision
  actor; it does not grant source-file access or turn the app into a web portal.
- The application sends notification email through the DMS user's configured SMTP
  relay or host mail handler. The non-secret transport and relay settings live in
  OS-user app configuration, while the Microsoft 365 app password is a write-only
  input held in the OS credential store. Neither is stored in `.dms`, frontend
  state, or an IPC response.
- When SMTP is not configured, the desktop app opens the host's default email
  handler with a pre-filled `mailto:` URI as a fallback notification path; this
  is an outbound mail draft, not a server-issued message, and the lifecycle
  state does not enter `in_review` until the operator confirms submission.
- A library document opened from the app launches the host-registered editor:
  Office for Office drafts and the default text editor for Markdown. The app
  releases the OS file handle before returning; it does not hold a long-running
  lock on the draft.
- An advisory lock file in `<edit-root>/.dms/lock` serialises metadata writes
  from this app only. It is not a multi-writer lock and never blocks read-only
  filesystem access from other tools.
- ISO 27001 support is traceable library membership, lifecycle, notes, and
  released-PDF integrity — not organizational certification.

## Document data domains

Each library document persists data across four durable domains. Mixing them
would either let a later profile edit silently rewrite a recorded release or
treat a refreshable display string as workflow authority, so the boundaries
below are part of the contract every adapter honours.

| Domain | What it stores | Mutability | Notes |
| --- | --- | --- | --- |
| Mutable document profile | Title, document number, document type, current owner reference, review interval/exemption inputs | Editable in **Edit document control data**; profile edits append canonical before/after evidence and invalidate stale candidates | The owner reference is a tenant-scoped Entra object ID under the current group binding; the cached display name/email is refreshable, never authoritative. Free-text owner labels from earlier schema versions are kept in `legacy_owner_label` as display-only unresolved state and are never converted into workflow authority by label or email matching |
| Immutable candidate/release snapshot | Profile (title, optional number/type, owner tenant/object ID + display snapshot), required effective date, confidentiality snapshot, workflow people (editor, approver, requester), and approval/review chain head | Snapshotted at candidate creation; never rewritten by later profile edits, lifecycle moves, or display-cache refreshes | New releases schedule the next review from the snapshot's effective date. Pre-v12 releases without a stored effective date render an explicit **unrecorded** state rather than substituting the current mutable profile |
| Mutable review schedule | Per-document review interval, exemption reason, next-review-due date | Editable through CAP-0015/CAP-0017; changes append evidence but do not invalidate open candidates unless the interval, exemption, or its reason actually changes | The next-review-due date is derived from the current release's stored effective date plus the resolved interval; an exemption suppresses the due date and is auditable |
| First-import Office source history | Original source SHA-256, format, sanitized scan outcome, and at most three normalized source person/date/kind/count observations | Captured only while a new Office document record is created; never rescanned or rewritten on re-registration | Unverified source provenance only: no source content, raw XML, Office properties, PowerPoint client IDs, inferred Entra identity, canonical workflow event, candidate, or release data |

## Out of scope (current architecture)

- Cloud database or multi-tenant backend
- Mandatory git-based version control
- Bundling Microsoft Office inside the app binary
- Cloud/server-side PDF conversion services
- SharePoint/OneDrive document-content synchronization or using their file and
  site permissions as the workflow-person directory
- Remote/browser approval or digital signing
- Confidentiality labels as a replacement for filesystem access control
- Auto-adding every file under the edit root without operator library action
- Centralised anti-virus, DLP, or e-signature services

## ADR-0023 — Shared Rust core with a first-class headless CLI

- **Decision:** The repository is a Cargo workspace with a Tauri-independent
  `dms-core` library, separate `dms` executable, and Tauri desktop shell.
  `dms-export` owns shared installed-Word and placeholder adapters, consumed
  by both executables. The CLI is not a Tauri app, plugin or bundled sidecar.
- **Why:** Local metadata and document-control rules must be testable and usable
  in scripted operator workflows without creating a WebView or making the CLI
  depend on desktop runtime capabilities.
- **Consequences:** `dms-core` may not depend on Tauri, frontend packages or
  Office automation. Its conversion port is independent of the release port.
  The CLI performs explicit local operations and non-release conversion; it
  cannot use conversion metadata to bypass lifecycle or identity boundaries.
  Windows/macOS installed Word is required for conversion; Linux has no
  renderer or automatic WSL bridge. Desktop release retains its approved
  snapshot and core-owned atomic PDF/evidence transaction.

## Related

- Privacy: [`privacy.md`](privacy.md)
- Decisions: [`design-decisions.md`](design-decisions.md)
- Capabilities: [`product/README.md`](product/README.md)
- Membership vs obsolescence: [`library-membership-and-obsolescence.md`](library-membership-and-obsolescence.md)
