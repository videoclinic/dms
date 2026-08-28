# dms-core

## Purpose

Provide the Tauri-independent local DMS domain model and `.dms` workspace
persistence.

## Ownership

| Path | Owns |
| --- | --- |
| `src/lib.rs` | Versioned workspace store, migration boundary, document-control data, and notes API |
| `src/assistance.rs` | Optional Claude Desktop policy, released-PDF/source comparison payloads, and assistance evidence |
| `src/audit.rs` | Filtered deterministic CSV/PDF audit reports, report-file integrity, and workspace report evidence |
| `src/catalogues.rs` | Document-type catalogue primitives plus shared stable-ID and label validation |
| `src/integrity.rs` | Advisory workspace locks, backup-manifest validation, and confirmed root-safe restore |
| `src/library.rs` | Folder/file discovery, membership, search, registration state, reassociation, and permalinks |
| `src/lifecycle.rs` | Version candidates, Entra/notification/export ports, content conformance, review decisions, release commits, and hash-chained evidence |
| `src/maintenance.rs` | Release checksum verification, workspace review defaults, periodic-review scheduling and transitions, and full-workspace ZIP backup with SHA-256 manifest |
| `src/policies.rs` | Folder-policy tree, retained confidentiality type-ID migrations, Entra display binding, and workflow-role resolution |
| `src/source_history.rs` | Bounded first-import OOXML source-history capture and schema validation |
| `src/frontmatter.rs` | Strict flat Markdown parsing, controlled-key rewrite from DMS, optional template-variable map, and expected/detected comparison |
| `src/template.rs` | Reusable workspace Word-template identity, validation, CommonMark-to-OOXML assembly, and frontmatter variable fill |
| `tests/` | Domain, migration-fixture, and persistence behaviour tests |

## Local Contracts

- Stable workspace, document, and note IDs are persisted identifiers.
- Source paths are canonicalized, must resolve under the configured edit root,
  and are stored relative to it with platform-independent `/` separators in
  metadata and machine-readable output.
- Source filename/path are locator facts. Mutable document profile, immutable
  candidate/release snapshots, and mutable review schedules are separate
  metadata domains. Each actual profile edit appends canonical before/after
  workflow evidence before invalidating stale candidates.
- First registration of an Office draft may retain a source-history record bound
  to the imported bytes: at most three normalized, attributable observations or
  a sanitized no-data/malformed/unattributed outcome. It contains no source
  content, raw OOXML, Office properties, client IDs, inferred Entra identity,
  workflow evidence, candidate, or release data, and re-registration never
  rescans or rewrites it.
- Folder discovery exposes only edit-root-relative regular files and directories,
  excludes `.dms`, Office temporary sidecars, and the exact root `Open in DMS.lnk`
  helper, and never auto-registers or auto-reassociates a source. Nested or
  differently named shortcuts remain ordinary unsupported files. Every discovered
  folder carries recursive counters for draft registered documents, addable supported
  files, unsupported files, and lost-source ((re-)moved) registered documents; each
  visible file contributes to exactly one counter. Lost-source phantom rows appear in
  the folder of the stored locator.
- Unregister and reassociate preserve stable document identity and retained
  document metadata; batch mutations validate atomically before changing state.
  Unregister sets `source_state` only: it does not require an idle lifecycle,
  cancel an open content or periodic review, delete files, or append a
  workflow event. Adding the same in-root path again restores `registered` on
  the same ID and leaves lifecycle unchanged.
- `Workspace::save` is the persistence boundary for ordinary mutations; release
  export owns its save/rollback transaction so a committed PDF and metadata
  release record cannot be reported independently.
- Schema migrations retain the source metadata backup and verify the migrated
  shape before atomically replacing `workspace.json`.
- Folder policies target only existing edit-root-relative directories, exclude
  `.dms`, and retain non-removable root defaults once configured.
- Microsoft Entra workspace metadata contains tenant/group identifiers and a
  read-only person display cache, never credentials or tokens.
- Configuration queries expose only enabled cached people for role pickers and
  return folder workflow assignments with their binding-qualified references;
  workflow inheritance and notification validation remain core-owned rules.
- Lifecycle candidates snapshot the requesting person, resolved owner, effective
  editor and approver, document profile, required effective date,
  confidentiality, source digest, target, and changelog. Authority comparisons
  use binding-qualified Entra object IDs, never mutable display names or email.
  Only committed releases occupy versions. A staged real owner/editor handover
  applies atomically with a successful release export.
- Approval-required operations refresh direct-user group membership and verify
  the interactive Entra actor through injected ports; core metadata stores no
  Graph, SMTP, or authentication credentials.
- Review and release content checks require scalar Markdown frontmatter
  `version` and `confidentiality`, compare optional `title` and
  `document_number` when present, and scan visible DOCX body/header/footer text;
  unsupported formats fail closed. Overrides require a reason and remain bound
  to the checked digest, target, confidentiality, and phase in the workflow
  chain. For registered Markdown members, DMS prefills and overwrites those
  controlled frontmatter keys from document control, effective confidentiality
  type ID, and candidate target version (one-way; never imports frontmatter into
  `.dms`). Sync is skipped until a confidentiality policy exists; policy/type/label
  changes re-sync all registered Markdown members. A retained type-ID migration
  leaves current policy/override references and idle drafts unchanged, invalidates
  an affected active candidate, then projects the enabled non-cyclic replacement
  ID while preparing the next candidate. Export chrome
  `{CONFIDENTIALITY}` still uses the display label.
- One optional stable workspace Word-template asset may reference an ordinary
  in-root non-symlink `.docx`. It is configuration rather than a controlled
  document, is excluded from Library file rows and counters, and preserves all
  non-body OOXML package parts during deterministic CommonMark body assembly.
  Its strict contract includes the four `DMS_*` custom properties with one
  release-value placeholder each. Markdown release passes only a configured,
  present, unchanged, valid template path to the export adapter; rejection does
  not call the adapter or commit release evidence. Assembly strips YAML
  frontmatter from the body and fills optional non-reserved `{KEY}` tokens from
  additional flat frontmatter scalars; reserved controlled tokens remain for
  export chrome.
- Workflow evidence is append-only, SHA-256 predecessor-linked, newest-first at
  the public history boundary, and validated whenever workspace metadata opens
  or saves.
- Cancel review and mark obsolete use the core lifecycle preconditions and
  append canonical `review_cancelled` or `document_obsoleted` evidence.
  Cancellation and obsolescence require reasons. There is no Begin revision
  action: registered documents are `draft` when never released or when the
  current draft digest differs from the latest non-withdrawn release source
  digest, and `released` when that digest still matches.
- Audit reports deterministically serialize the selected control, workflow,
  periodic-review, release, and verification records without embedding source
  drafts or release PDFs. Report paths remain inside the edit root, never
  overwrite existing files, and are recorded in a separate canonical
  workspace-level `report_generated` evidence chain.
- `Workspace::verify_release` and `Workspace::verify_all_releases` re-read the
  recorded PDF and compare its SHA-256 digest; they never modify, repair, or
  delete release bytes.
- `Workspace::start_periodic_review` binds the current release ID, version,
  PDF digest, confidentiality snapshot, and approver; a mismatched or missing
  PDF blocks the request.
- `Workspace::complete_periodic_review` refreshes current Entra eligibility,
  requires the snapshotted eligible approver, records the result, and applies
  `ConfirmedCurrent` / `ChangesRequired` / `Obsolete` transitions
  deterministically.
- Periodic-review request, result, comment-required cancellation, and each
  reminder attempt are separate canonical workflow events. Cancellation leaves
  the release schedule unchanged; reminders neither duplicate the request nor
  change lifecycle state.
- Review permalinks resolve both content-approval requests and periodic-review
  requests so notification links never point at an unresolvable review ID.
- Canonical `dms://open` permalinks resolve a stable workspace UUID alone to its
  root Library target, or pair it with a stable document UUID. `review` and `notes`
  refine only document targets; workspace-only links reject both. Unknown extra
  parameters do not affect valid document-link resolution.
- `Workspace::backup_workspace` refuses to overwrite an existing archive,
  refuses symlinks and non-regular files, and writes a Zip archive containing
  metadata, every registered draft, every recorded release PDF, and a
  SHA-256 manifest entry per file.
- Advisory `.dms/lock` records contain local owner/process evidence. Ordinary
  acquisition refuses a current lock, stale-only takeover uses the
  workspace-configured threshold, and overriding any existing lock requires a
  separate explicit core operation.
- Restore validates the complete archive manifest, entry types, sizes, digests,
  workspace identity, destination lock, and confirmed replacement policy before
  writing only beneath existing operator-selected edit and publish roots. It
  holds an owner-recorded destination lock throughout file writes and rejects
  cross-platform path aliases before restoration.
- Claude assistance is disabled by default, permits only configured
  confidentiality type IDs, and verifies the current release before extracting
  comparison text. An oversized preview exposes every exact excerpt and its
  measured size; only an explicit operator-selected subset that fits the limit
  yields a digest-bound payload. No payload is silently truncated, and only
  explicit accepted-use evidence is recorded in lifecycle records.

## Work Guidance

- Keep public operations deterministic and explicit.
- Add migration support before increasing `SCHEMA_VERSION`.
- Schema v6 adds the default-disabled workspace Claude-assistance policy and
  optional candidate/release/workflow assistance evidence, with a
  `v5.json.bak` retained during migration.
- Schema v7 adds the workspace-level report evidence chain, with a
  `v6.json.bak` retained during migration.
- Schema v8 adds the positive per-workspace advisory-lock staleness threshold,
  with a `v7.json.bak` retained during migration.
- Schema v9 adds the optional document-control effective date, with a
  `v8.json.bak` retained during migration.
- Schema v12 separates mutable document profile, release-bound effective date
  and profile/owner snapshots, and mutable review schedule. Its v11 migration
  retains `v11.json.bak`, preserves legacy owner text without identity inference,
  maps a stored date only to the current non-withdrawn release and retained open
  candidates, and leaves older release snapshot omissions unrecorded.
- Schema v13 separates the SMTP authentication login user from its RFC 5322
  `From` mailbox. Its v12 migration copies the legacy sender to both fields and
  retains `v12.json.bak`; SMTP passwords remain outside core metadata.
- Schema v14 adds the optional Markdown Word-template record without assigning
  an implicit asset. Its v13 migration retains `v13.json.bak`.
- Schema v15 adds optional confidentiality replacement IDs. Its v14 migration
  writes `replacement_type_id: null` for every catalogue entry and retains
  `v14.json.bak`.
- Schema v16 removes notification transport and SMTP relay fields from portable
  workspace metadata. Desktop injects the current OS user's validated settings
  only for a lifecycle operation; the v15 migration retains `v15.json.bak`.
- Schema v17 adds optional first-import Office source-history records. Its v16
  migration writes `source_history: null` for every existing document and
  retains `v16.json.bak`.
- Schema v18 adds the persisted Entra tenant binding. Its v17 migration retains
  `v17.json.bak` and leaves a group-only binding explicitly unverified until an
  operator reapplies the source; it never infers tenant identity from cache,
  OS-user configuration, or historic evidence.

## Verification

- `cargo test -p dms-core`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.