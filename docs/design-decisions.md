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
  prerequisites; CI and release packaging must cover both OS; platform-specific
  Office automation adapters (ADR-0008) sit behind one export interface.

## ADR-0003 — Drafts are Office originals; releases are PDFs

- **Decision:** Draft/working documents remain Microsoft Office formats
  (e.g. `.docx`, `.xlsx`, `.pptx`) under the edit root. Only PDF artifacts enter
  the released state under the publish root. Git-style content VCS is not
  required.
- **Why:** Authors keep familiar tools; controlled released form is a stable
  PDF suitable for distribution and integrity checking.
- **Consequences:** The Office draft remains after release. PDF bytes are
  produced by the application via installed Microsoft Office (ADR-0008), then
  version-named and checksummed. The app versions released PDFs, not each Office
  save; draft rollback depends on operator workspace backups.

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

## ADR-0007 — Versioned release file names

- **Decision:** Released PDFs use the pattern `<stem>_V<major>.<minor>.pdf`
  (examples: `Handbook_V1.0.pdf`, `Handbook_V1.1.pdf`, `Handbook_V2.0.pdf`).
  First release is `V1.0`. Cosmetic, non-semantic changes increment minor;
  substantive or uncertain changes increment major and reset minor to zero.
  Each release creates a new file; existing version paths are never overwritten.
  The approved change class determines the bump.
- **Why:** Version identity is visible in ordinary file explorers and backups
  without opening the app; supports ISO controlled-document version labeling.
- **Consequences:** Stem is derived from the draft base name (without Office
  extension); version counter lives in `.dms` per library document; collisions
  with manually dropped files at the target path fail closed.

## ADR-0008 — PDF export via preinstalled Microsoft Office

- **Decision:** The application drives Office → PDF export using preinstalled
  Microsoft Office desktop apps on the host OS — **Windows and macOS**.
  Implementation may use platform-native automation (e.g. COM on Windows,
  AppleScript/Office automation on macOS) behind one export interface. Release
  does not rely on the operator manually exporting and selecting a PDF as the
  primary path.
- **Why:** Consistent release pipeline, correct versioned target path, and
  checksum binding to bytes the app just produced; matches operator expectation
  that the DMS performs export and versioning on both supported desktops.
- **Consequences:** Licensed desktop Office on the operator machine is a runtime
  dependency for release on each OS; headless CI may mock or skip live Office
  export; export quality equals installed Office; automation must handle app
  already running, file locks, and failure rollback (no successful version
  record without PDF).

## ADR-0009 — SMTP notification opens local-app approval

- **Decision:** The application sends review-request email through a configured
  SMTP relay after the operator selects an approver. Email contains no document
  content and opens the request in the desktop app through a local-app deep
  link. SMTP credentials live in the OS credential store; `.dms` stores only
  non-secret relay configuration and delivery-attempt metadata.
- **Why:** This gives approvers a direct notification without building a server
  or a browser portal.
- **Consequences:** Each approver needs the application and access to the same
  workspace. SMTP delivery acceptance is recorded, not recipient reading. A
  failed send leaves the document out of review and offers retry; it cannot
  create a silently unnotified review request.

## ADR-0010 — Workspace confidentiality catalogue with inherited folder policy

- **Decision:** A workspace defines a list of stable confidentiality type IDs
  with display labels. Folder policies use edit-root-relative paths; a document
  derives its default from the nearest configured ancestor, including the root.
  A document may retain an explicit override. Release records snapshot the
  effective type.
- **Why:** Operators can classify a folder tree once while retaining an
  exception path for individual documents.
- **Consequences:** Changing a folder policy updates only descendants without a
  nearer policy or document override. Classification is metadata for handling
  and audit; filesystem ACLs remain the access-control boundary.

## ADR-0011 — Host OS default Office application is the draft editor

- **Decision:** Opening a library document from the desktop app launches the
  host-registered Microsoft Office application for that draft format. The app
  does not embed an editor, render its own preview, or auto-save the draft.
  After Office exits, the lifecycle state of the document is unchanged; the
  next review or release re-hashes the draft bytes before recording.
- **Why:** Authors keep their familiar Office workflow; the desktop app only
  owns the controlled metadata, never the editable document.
- **Consequences:** A draft opened while a review is in flight can be modified
  outside the app. Approval is revision-bound (ADR-0004): when the draft hash
  no longer matches the review digest, the app marks the approval invalidated
  and requires a new request. Office integration never blocks the app on the
  Office process.

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
  type, predecessor event hash, ISO-8601 UTC timestamp, configured approver
  identity (when applicable), local OS user, revision digest (when applicable),
  confidentiality snapshot and approved change class (when applicable), and
  the operator comment text.
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
- **Consequences:** CAP-0015 owns master data, next-review-due, cancel-review,
  and obsolete. CAP-0016 owns publish-history navigation and orphan cleanup.

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
