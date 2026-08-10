# dms-core

## Purpose

Provide the Tauri-independent local DMS domain model and `.dms` workspace
persistence.

## Ownership

| Path | Owns |
| --- | --- |
| `src/lib.rs` | Versioned workspace store, migration boundary, document-control data, and notes API |
| `src/catalogues.rs` | Document-type catalogue primitives plus shared stable-ID and label validation |
| `src/library.rs` | Folder/file discovery, membership, search, registration state, reassociation, and permalinks |
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
- `Workspace::save` is the only persistence boundary for mutating callers.
- Schema migrations retain the source metadata backup and verify the migrated
  shape before atomically replacing `workspace.json`.
- Folder policies target only existing edit-root-relative directories, exclude
  `.dms`, and retain non-removable root defaults once configured.
- Microsoft Entra workspace metadata contains tenant/group identifiers and a
  read-only person display cache, never credentials or tokens.

## Work Guidance

- Keep public operations deterministic and explicit.
- Add migration support before increasing `SCHEMA_VERSION`.

## Verification

- `cargo test -p dms-core`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
