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
| `src/policies.rs` | Folder-policy tree, confidentiality inheritance, Entra display binding, and workflow-role resolution |
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
- Folder discovery exposes only edit-root-relative regular files and directories,
  excludes `.dms` and Office temporary sidecars, and never auto-registers or
  auto-reassociates a source. Every discovered folder carries recursive counters
  for draft registered documents, addable supported files, and unsupported files;
  each visible file contributes to exactly one counter.
- Unregister and reassociate preserve stable document identity and retained
  document metadata; batch mutations validate atomically before changing state.
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
- Review and release content checks scan rendered Markdown and visible DOCX
  body/header/footer text; unsupported formats fail closed. Overrides require a
  reason and remain bound to the checked digest, target, confidentiality, and
  phase in the workflow chain.
- Workflow evidence is append-only, SHA-256 predecessor-linked, newest-first at
  the public history boundary, and validated whenever workspace metadata opens
  or saves.
- Begin revision, cancel review, and mark obsolete use the core lifecycle
  preconditions and append canonical `revision_begun`, `review_cancelled`, or
  `document_obsoleted` evidence. Cancellation and obsolescence require reasons.
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
- Canonical `dms://open` permalinks parse only stable workspace/document UUIDs.
  The optional `review` and `notes` targets refine navigation without replacing
  those identity keys; unknown extra parameters do not affect resolution.
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

## Verification

- `cargo test -p dms-core`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.