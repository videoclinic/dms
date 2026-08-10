# dms-cli

## Purpose

Expose the implemented `dms-core` operations through the headless `dms`
executable.

## Ownership

| Path | Owns |
| --- | --- |
| `src/main.rs` | Clap command contract and output presentation |
| `tests/` | End-to-end CLI command tests |

## Local Contracts

- Parse arguments and render results; domain validation remains in `dms-core`.
- Mutations require explicit command targets and must not create a desktop
  runtime or invoke a sidecar.
- `--json` results are structured and contain no document bytes or credentials.
- Policy commands use explicit edit-root-relative targets. Injected eligible-
  person snapshots use the explicit `@file:PATH` marker and contain only
  non-secret display data.
- Library commands expose filesystem-derived tree/list/search results separately
  from explicit document add, unregister, reassociate, and permalink mutations.

## Work Guidance

- Keep normal output concise and send failures to stderr.
- Keep command names stable once released; add a new explicit command rather
  than silently changing a mutating command's meaning.

## Verification

- `cargo test -p dms-cli`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
