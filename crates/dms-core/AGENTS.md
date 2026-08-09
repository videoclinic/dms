# dms-core

## Purpose

Provide the Tauri-independent local DMS domain model and `.dms` workspace
persistence.

## Ownership

| Path | Owns |
| --- | --- |
| `src/lib.rs` | Workspace schema, validation, document-control data, and notes API |
| `tests/` | Domain and persistence behaviour tests |

## Local Contracts

- Stable workspace, document, and note IDs are persisted identifiers.
- Source paths are canonicalized, must resolve under the configured edit root,
  and are stored relative to it.
- Source filename/path are locator facts; control data is independent metadata.
- `Workspace::save` is the only persistence boundary for mutating callers.

## Work Guidance

- Keep public operations deterministic and explicit.
- Add migration support before increasing `SCHEMA_VERSION`.

## Verification

- `cargo test -p dms-core`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
