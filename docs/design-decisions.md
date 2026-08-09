# Design decisions

Irreversible or cross-cutting runtime, storage, privacy, and platform forks.
Capability-local rules stay in their CAP files.

## ADR-0001 — Filesystem `.dms` store instead of a database

- **Decision:** Persist workspace config, library registry, notes,
  approval/release state, and checksums in a hidden directory under the edit
  root (default name `.dms`), not in SQLite/Postgres or another database server.
- **Why:** Operator can copy, backup, and inspect a folder tree with normal
  filesystem tools; matches “no database” product constraint.
- **Consequences:** Concurrent multi-writer access is limited; schema is file
  formats under `.dms`; migrations are filesystem format migrations.

## ADR-0002 — Tauri 2 on Windows and macOS

- **Decision:** Build a Tauri 2 application with **Windows and macOS both
  required** supported targets. Linux is not a v1 requirement.
- **Why:** Native filesystem access with a small binary footprint; WebView UI
  without shipping a full browser; operators use both Windows and Mac for ISO
  documentation work.
- **Consequences:** Rust toolchain plus WebView2 (Windows) and WKWebView/macOS
  prerequisites; CI and release packaging must cover both OS; format-specific
  PDF export adapters (ADR-0008) sit behind one export interface.

## ADR-0003 — Source drafts; releases are PDFs

- **Decision:** Draft/working documents remain Microsoft Office formats (for
  example `.docx`, `.xlsx`, `.pptx`) or Markdown (`.md`) under the edit root.
  Only PDF artifacts enter the released state under the publish root. Git-style
  content VCS is not required.
- **Why:** Authors retain familiar Office workflows or use portable plain-text
  Markdown; the controlled released form is a stable PDF suitable for
  distribution and integrity checking.
- **Consequences:** The source draft remains after release. The application uses
  format-specific local PDF export (ADR-0008), then versions and checksums the
  result. It versions released PDFs, not each source save; draft rollback depends
  on operator workspace backups.

## ADR-0004 — Local application approval with revision-bound evidence

- **Decision:** A review request and every approve, reject, or request-changes
  decision happen in the desktop application and are recorded in `.dms`.
  A request carries a SHA-256 digest of the draft bytes; a decision or release
  is valid only while the current draft has that digest. Every workflow event
  includes its predecessor hash and a SHA-256 event hash over a canonical event
  body.
- **Why:** Small teams need a clear, simple review trail without an external
  workflow engine or a database. Binding approval to the reviewed draft avoids
  releasing a document changed after it was approved.
- **Consequences:** The app records configured approver identity, local OS user,
  timestamps, required comments, revision digest, and event-chain hash. A hash
  chain detects uncoordinated metadata changes but is not non-repudiation: a
  writer able to replace `.dms` can rewrite hashes. External identity proof,
  digital signatures, and a remote audit store remain out of scope.

## ADR-0005 — SHA-256 checksums on released PDFs

- **Decision:** Every released PDF stores a SHA-256 digest of file bytes at
  release time; the app can recompute and compare later.
- **Why:** Detects accidental or unauthorized modification of released
  controlled documents; supports ISO integrity expectations for documented
  information.
- **Consequences:** Renames without byte changes keep the same digest; any
  byte change fails verification; checksums are not a substitute for access
  control or encryption.

## ADR-0006 — Dual roots with mirrored relative tree

- **Decision:** Each workspace stores an **edit root** and a **publish root**.
  Library documents are identified by path relative to the edit root. Released
  PDFs are written under the publish root using the same relative directory
  segments; missing directories are created on release.
- **Why:** Separates work-in-progress Office trees from published controlled
  PDFs while preventing path mismatch (“edited in A, published in random B”).
  Reconstructing the tree keeps human navigation consistent across roots.
- **Consequences:** Moving/renaming a draft outside the app breaks the relative
  link until repaired; changing either root is a workspace configuration
  change and must be validated; publish root must be writable.

## ADR-0007 — Versioned, classified release file names

- **Decision:** Released PDFs use the pattern
  `<stem>_V<major>.<minor>_<confidentiality-type-id>.pdf` (examples:
  `Handbook_V1.0_restricted.pdf`, `Handbook_V1.1_restricted.pdf`,
  `Handbook_V2.0_confidential.pdf`). The filename carries the effective
  confidentiality type's stable, portable ID snapshotted at release. First
  release is `V1.0`. Cosmetic, non-semantic changes increment minor;
  substantive or uncertain changes increment major and reset minor to zero.
  Each release creates a new file; existing version paths are never overwritten.
  The approved change class determines the bump.
- **Why:** Version and confidentiality are visible in ordinary file explorers
  and backups without opening the app; supports ISO controlled-document
  version labeling.
- **Consequences:** Stem is derived from the draft base name (without Office
  extension); version counter and effective confidentiality snapshot live in
  `.dms` per library document; an operator who can list the publish tree can
  see the classification ID. Collisions with manually dropped files at the
  target path fail closed.

## ADR-0008 — Format-specific local PDF export

- **Decision:** The application dispatches PDF export by source format. Office
  drafts use preinstalled Microsoft Office desktop apps on Windows and macOS.
  Markdown (`.md`) drafts are rendered locally as CommonMark HTML inside a
  shipped **print shell** (HTML/CSS/logo chrome derived from the corporate
  Vorlage layout) and printed by the native WebView PDF API. Markdown release
  does **not** convert through Word or a runtime `.docx` template. Both paths
  share one export interface, one release-context **export chrome** map
  (version label, confidentiality display label and type ID, optional title /
  document number) sourced only from `.dms`, and the same versioned target,
  validation, checksum, and atomic-commit flow. Release does not rely on the
  operator manually exporting and selecting a PDF. Office drafts may still
  contain `{CONFIDENTIALITY}` / `{VERSION}` tokens that export replaces on a
  temporary copy before invoking Office.
- **Why:** This preserves the established Office release path while letting
  Markdown release without requiring Office, a bundled browser, or a cloud
  conversion service, and still produces corporate header/footer chrome aligned
  with the Vorlage without making Office properties or Markdown front matter
  authoritative (CAP-0015).
- **Consequences:** Licensed desktop Office is a runtime dependency only for
  Office drafts. Office automation may use platform-native mechanisms (for
  example COM on Windows and AppleScript/Office automation on macOS). Markdown
  export requires supported native WebView PDF APIs on each OS and a proven
  print-shell layout (fixed header/footer and page indicators under WebView2
  and WKWebView). CAP-0002 visible-content markers remain a source-draft check;
  the Markdown print shell may repeat the canonical captions but does not
  replace that gate. CI may use test doubles, but platform integration must
  prove both adapters. No successful version record exists unless the selected
  exporter produces the valid PDF that the app validates, checksums, and
  atomically commits.

## ADR-0009 — SMTP notification opens local-app approval

- **Decision:** The application sends a review-request email through a configured
  SMTP relay after the operator submits a review to its effective approver, then
  sends the recorded decision outcome to the requester's snapshotted email
  address. Email contains
  no document content and opens the request in the desktop app through a
  CAP-0020 permalink (ADR-0020) whose target is the review request. The URI
  identifies the stable workspace, document, and review request; the receiving
  app resolves it only against an accessible registered workspace. SMTP
  credentials live in the OS credential store; `.dms` stores only non-secret
  relay configuration and delivery-attempt metadata.
- **Why:** This gives approvers a direct notification without building a server
  or a browser portal.
- **Consequences:** Each approver needs the application and access to the same
  workspace. The URI cannot open arbitrary filesystem paths or record a decision
  by itself. SMTP delivery acceptance is recorded, not recipient reading. A
  failed review-request send leaves the document out of review and offers retry;
  a failed decision-outcome notification is retryable but never reverses the
  recorded decision.

## ADR-0010 — Workspace confidentiality catalogue with inherited folder policy

- **Decision:** A workspace defines a list of stable confidentiality type IDs
  with display labels. Folder policies use edit-root-relative paths; the edit
  root always has a direct policy and a non-root folder may add, replace, or
  remove its own direct policy. A document derives its default from the nearest
  configured ancestor, including the root, and may retain an explicit override.
  Release records snapshot the effective type.
- **Why:** Operators can classify a folder tree once while retaining an
  exception path for individual documents.
- **Consequences:** Changing a folder policy, or removing a non-root policy,
  updates only descendants without a nearer policy or document override; the
  removed policy is not copied into records. Classification is metadata for
  handling and audit; filesystem ACLs remain the access-control boundary.

## ADR-0011 — Host OS editor opens source drafts

- **Decision:** Opening a library document from the desktop app launches the
  host-registered Microsoft Office application for Office formats and the host
  default text editor for Markdown. The app does not embed an editor, render a
  custom preview, or auto-save the draft. After the editor exits, lifecycle
  state is unchanged; the next review or release re-hashes draft bytes.
- **Why:** Authors retain their chosen source workflow while the desktop app
  owns only controlled metadata and release handling.
- **Consequences:** A draft opened while a review is in flight can be modified
  outside the app. Approval is revision-bound (ADR-0004): when the draft hash
  no longer matches the review digest, the app marks the approval invalidated
  and requires a new request. The app never blocks on the opened editor process.

## ADR-0012 — Configurable notification transport with mailto fallback

- **Decision:** The workspace's notification transport is selectable. The
  default is the configured SMTP relay (ADR-0009). When SMTP is absent or
  disabled, the app falls back to the host's default mail handler via a
  pre-filled `mailto:` URI. The lifecycle state never advances to `in_review`
  on `mailto:` submission alone; it requires an explicit operator confirmation
  inside the app that the message was sent.
- **Why:** The operator wants a notification path that uses the host's mail
  application when a relay is not appropriate, without forcing every workspace
  to host a relay.
- **Consequences:** The contract for "review requested" is identical for both
  transports; only the message-creation step differs. SMTP delivery acceptance
  is recorded (ADR-0009); `mailto:` fallback records the operator's send
  confirmation and a placeholder receiver address. The UI must distinguish the
  two states (sent vs queued-in-mail-handler) in the workflow history.

## ADR-0013 — Canonical event body for the workflow hash chain

- **Decision:** Every workflow event stored in `.dms` is the SHA-256 of a
  canonical event body that contains, at minimum: stable document ID, event
  type, predecessor event hash, ISO-8601 UTC timestamp, requester, effective
  approver, and responsible editor IDs (when applicable), local OS user,
  revision
  digest (when applicable), confidentiality snapshot and approved change class
  (when applicable), and the operator comment text.
- **Why:** A canonical schema is the only way the chain is verifiable later
  and the only way two installations can compare evidence.
- **Consequences:** Any reader can recompute and verify each event hash and the
  chain head. Comments are part of the canonical body; changing a comment
  invalidates the chain. The app exposes verification as a routine UI action
  and as an export format (CAP-0012).

## ADR-0014 — Operational portability of the workspace

- **Decision:** A workspace consists of the edit root, `<edit-root>/.dms/`, and
  the separate publish root. A full backup includes controlled drafts, metadata,
  and recorded release PDFs with a checksum manifest. The app does not
  introduce encryption or expiry. Restore validates the manifest and maps both
  roots only to operator-confirmed paths.
- **Why:** Operators must be able to take a workspace to another machine,
  archive it, or restore it without bespoke tooling. ADR-0001 already commits
  to filesystem-native metadata; this ADR commits the operational shape.
- **Consequences:** Corrupt `.dms` is detected on open and surfaced as a
  restore prompt pointing at the most recent backup. Cross-machine moves work
  only when both sides can resolve the absolute roots; relative paths remain
  the locator; stable document IDs remain the durable identity.

## ADR-0015 — Stable document ID plus relative-path locator

- **Decision:** At library add, each document receives a stable opaque ID
  stored in `.dms`. The draft’s edit-root-relative path is the mutable locator
  used for open, export path mapping, and human navigation. Workflow events,
  release records, and notes key on the stable ID.
- **Why:** Operators rename and move drafts inside the tree; process history
  must survive locator changes without breaking the approval chain.
- **Consequences:** Rename/move detection updates the locator only
  (CAP-0013). Path is never the sole foreign key for history. Export still
  derives publish relative segments from the current locator (ADR-0006).

## ADR-0016 — Explicit post-release revision cycle

- **Decision:** A successful release leaves the document in `released` until
  the operator chooses **Begin revision**, which returns it to `draft` for the
  next change cycle. Released PDFs are never overwritten; the next release
  creates a new versioned file. Obsolescence is a separate terminal control
  state that blocks further review/release.
- **Why:** ISO-style controlled documents need a clear “current released” vs
  “work in progress for next version” distinction and a deliberate start to
  the next cycle.
- **Consequences:** CAP-0015 owns document control data, next-review-due,
  cancel-review, and obsolete. CAP-0016 owns publish-history navigation and
  orphan cleanup.

## ADR-0017 — Versioned `.dms` schema with fail-closed migration

- **Decision:** `.dms` carries an explicit schema version. Supported older
  schemas are backed up and migrated atomically on open; an unknown newer schema
  is read-only. Migration never silently discards an unknown field or rewrites
  metadata after a failed validation.
- **Why:** Operators must maintain and reopen a workspace across application
  upgrades without corrupting document history.
- **Consequences:** Every schema change requires a migration fixture and
  old→new verification. The application retains the pre-migration metadata
  until the migrated store passes parsing and event-chain verification.

## ADR-0018 — Claude Desktop is an optional operator-mediated handoff

- **Decision:** The app may launch an installed Claude Desktop and copy a
  locally prepared change-evaluation prompt, but the operator manually pastes
  the prompt and any response. The app does not call an undocumented Claude
  Desktop interface. AI output is advisory and cannot choose version, approve,
  or write workflow state.
- **Why:** Anthropic documents Claude Desktop as an MCP client that loads local
  MCP servers/extensions, not as a local model API another desktop application
  can invoke. Manual handoff is bounded and does not add API credentials or a
  custom MCP extension to v1.
- **Consequences:** Core operation remains AI-independent. The operator must
  preview and consent to every external-processing payload; confidentiality
  policy can disable handoff. Direct automation requires a future supported
  provider contract and an ADR update.

## ADR-0019 — Inherited workflow-role routing without application access control

- **Decision:** The workspace keeps a roster of people and assigns one
  responsible editor and one approver at the edit root or any subfolder. Each
  role derives independently from the nearest ancestor policy unless an
  individual document overrides it. The effective approver receives a review
  request; the effective editor and approver are snapshotted as workflow
  evidence.
- **Why:** Operators can route responsibility across a directory tree without
  repeating the same assignments for every document, while retaining document
  exceptions.
- **Consequences:** These assignments route work and provide audit context only.
  They do not prevent a person from opening or editing a shared source file;
  filesystem ACLs remain the access-control boundary. Changing an effective
  approver invalidates an open review and requires a new request.

## ADR-0020 — Document permalinks key only on stable IDs

- **Decision:** Local-app document permalinks and notification deep links use a
  registered URI scheme whose required identity keys are the **stable workspace
  ID** and **stable document ID**. Optional target parameters may select a
  landing surface (document selection, review request, notes). Draft file name,
  relative path, version label, publish PDF name, and absolute filesystem paths
  are never identity keys and must not be required to resolve the link.
- **Why:** Operators rename drafts and release new versions routinely; shared
  links in mail and notes must keep pointing at the same controlled document
  without manual rewrite.
- **Consequences:** Resolution requires the app plus an accessible registered
  workspace that knows those IDs (CAP-0020). Path/version display is resolved
  after lookup. Missing workspace or document IDs fail closed with an operator
  message. Permalink open never records a workflow decision by itself.
