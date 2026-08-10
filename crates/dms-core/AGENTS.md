# dms-core

## Purpose

Provide the Tauri-independent local DMS domain model and `.dms` workspace
persistence.

## Ownership

| Path | Owns |
| --- | --- |
| `src/lib.rs` | Versioned workspace store, migration boundary, document-control data, and notes API |
| `src/assistance.rs` | Optional Claude Desktop policy, released-PDF/source comparison payloads, and assistance evidence |
| `src/catalogues.rs` | Document-type catalogue primitives plus shared stable-ID and label validation |
| `src/library.rs` | Folder/file discovery, membership, search, registration state, reassociation, and permalinks |
| `src/lifecycle.rs` | Version candidates, Entra/notification/export ports, content conformance, review decisions, release commits, and hash-chained evidence |
| `src/maintenance.rs` | Release checksum verification, periodic-review scheduling, result/cancellation/reminder transitions, and full-workspace ZIP backup with SHA-256 manifest |
| `src/policies.rs` | Folder-policy tree, confidentiality inheritance, Entra display binding, and workflow-role resolution |
| `tests/` | Domain, migration-fixture, and persistence behaviour tests |

## Local Contracts

- Stable workspace, document, and note IDs are persisted identifiers.
- Source paths are canonicalized, must resolve under the configured edit root,
  and are stored relative to it with platform-independent `/` separators in
  metadata and machine-readable output.
- Source filename/path are locator facts; control data is independent metadata.
- Folder discovery exposes only edit-root-relative regular files and directories,
  excludes `.dms` and Office temporary sidecars, and never auto-registers or
  auto-reassociates a source.
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
- Lifecycle candidates snapshot the requesting person, effective editor,
  effective approver, control data, confidentiality, source digest, target, and
  changelog. Only committed releases occupy versions.
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
- `Workspace::backup_workspace` refuses to overwrite an existing archive,
  refuses symlinks and non-regular files, and writes a Zip archive containing
  metadata, every registered draft, every recorded release PDF, and a
  SHA-256 manifest entry per file.
- Claude assistance is disabled by default, permits only configured
  confidentiality type IDs, verifies the current release before extracting
  comparison text, refuses oversized payloads without truncation, and records
  only explicit accepted-use evidence in lifecycle records.

## Work Guidance

- Keep public operations deterministic and explicit.
- Add migration support before increasing `SCHEMA_VERSION`.
- Schema v6 adds the default-disabled workspace Claude-assistance policy and
  optional candidate/release/workflow assistance evidence, with a
  `v5.json.bak` retained during migration.

## Verification

- `cargo test -p dms-core`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.