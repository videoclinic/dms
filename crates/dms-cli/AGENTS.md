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
- Document-profile owner mutation accepts only explicit
  `--owner-object-id <UUID>` under the configured identity-source binding; no
  free-text owner alias is retained. Effective date belongs to candidate
  submission and review scheduling, not `update-control`.
- Periodic-review commands expose list, start, result, cancellation, and reminder
  operations. Result, cancellation, and reminder require `--confirm`; result and
  reminder report the unavailable live Entra/notification adapter explicitly
  until phase 9i supplies production integrations.
- Report commands generate explicitly filtered CSV/PDF files, list their
  canonical evidence, and verify the recorded file digest. JSON output contains
  report metadata and hashes only, never source or release bytes.
- Workspace integrity commands expose advisory-lock status, explicit stale
  takeover and confirmed release, lock-threshold configuration, manifest backup,
  and confirmed restore into explicit existing replacement roots.
- `workspace conversion-policy --edit-root <dir>` inspects the library default;
  optional `--todo-projection enabled|disabled` updates it through the typed core
  setter and explicit `Workspace::save`. JSON is the `ConversionPolicy` object,
  not per-source resolution. No implicit target, source access, conversion,
  Release or lock operation is added. Ordinary `Workspace::open` migration and
  managed-instruction synchronization still apply, including during inspection.
- `convert markdown` is standalone, non-release conversion: it reads only the
  read-only export projection of an explicit `workspace.json` (schema 18, 19, 21 or
  current) and writes new paired DOCX/PDF artifacts plus a `manifest.json` into a
  disjoint, previously nonexistent output root. It never opens, migrates or
  writes a workspace, acquires a lock, or contacts Entra or notification
  services, and its manifest is marked `conversion_only`.
- `convert markdown` reports the exact selected set to stderr before converting;
  `--json` stdout carries only the final manifest. The manifest is emitted even
  when individual files fail, and any failure makes the command exit nonzero.
  `--dry-run` validates and assembles into temporary locations without Word
  preflight, Word calls or output-root writes.
- Conversion JSON/manifest results carry each file's effective `todo_projection`
  (`enabled`/`disabled`, null if unresolved). Normal-mode stderr reports the
  policy or `unresolved` beside the source path. Core owns projection and
  source-free mixed-entry diagnostics; input hashes still cover original bytes.

## Work Guidance

- Keep normal output concise and send failures to stderr.
- Keep command names stable once released; add a new explicit command rather
  than silently changing a mutating command's meaning.

## Verification

- `cargo test -p dms-cli`
- `cargo test -p dms-cli --test cli --locked` includes policy-resolution evidence
  through the public core plan, CLI rejection of invalid library/source settings,
  mixed-entry/cell failures, override precedence, effective-policy JSON and normal
  diagnostics (including original CRLF/Unicode cell locations), unchanged source
  SHA-256 and dry-run no-write behaviour. The
  existing workspace `sha2` dependency is test-only for CLI digest assertions.
- The same CLI gate covers conversion-policy default/missing-policy inspection,
  enabled/disabled persistence and readback, explicit-target/invalid-value
  rejection, malformed stored policy, normal/JSON output, core migration with
  exact backup, and whole-fixture snapshots proving policy-only mutation.
  Separate explicit conversion dry-runs prove configured-default/override
  precedence in effective-policy receipts without output or protected-tree writes.

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
