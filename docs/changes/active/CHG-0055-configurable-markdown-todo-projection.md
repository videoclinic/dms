# CHG-0055 — Configurable Markdown TODO projection

**Plan ID:** CHG-0055-configurable-markdown-todo-projection
**Created:** 2026-10-15
**Depends on:** CHG-0053 Phase 4 remains the authority for retained-DOCX batch-conversion fidelity.
**Status:** planned

| Field | Value |
| --- | --- |
| ID | CHG-0055 |
| Status | planned |
| External request | Direct operator request: incorporate the TODO filter as new CHR with the application-record skill. it should be possible to enable / disable the filter per file and library wide |
| Affected CAPs | CAP-0022 |
| Decision records | None; the conversion-only boundary remains ADR-0023. |

## Outcome

An operator can explicitly choose whether standalone Markdown conversion projects
visible `TODO:`-marked entries out of generated DOCX/PDF artifacts for a whole
workspace library or an individual source file. The setting is conversion-only:
it never rewrites source Markdown, input hashes, workspace release authority, or
desktop Release assembly.

## Planned behaviour

- The workspace stores one explicit library-wide default for standalone Markdown
  conversion. Its initial value is `disabled` to preserve current output unless an
  operator enables projection.
- A flat, source-owned per-file conversion override is explicit: `todo_projection:
  enabled` or `todo_projection: disabled`. It wins over the library-wide default.
  Omission inherits the library-wide setting; other values fail that source.
- With projection enabled, a visible paragraph, heading, list item, code block, or
  complete Markdown table row containing `TODO:` is omitted from conversion output.
  Surviving ordered list items are renumbered without gaps. Marker matching is
  case-insensitive; frontmatter is not a visible entry.
- CLI output and the conversion manifest record the effective per-file projection
  policy without source bodies. The input SHA-256 continues to cover the unmodified
  source bytes.
- Projection applies only to `dms convert markdown`. Desktop Release, registered
  document control, candidates, approvals, release evidence, and publish-root
  artifacts are unchanged.
- The workspace setting has an explicit headless CLI configuration path. A desktop
  configuration surface is not part of this request unless required to preserve an
  existing configuration contract.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Define persistent library policy and per-file override | pending | Focused core and CLI policy-resolution tests exit 0 for disabled default, explicit library settings, override precedence, and invalid values. |
| 2 | Project enabled TODO entries through standalone conversion | pending | Conversion/domain and CLI tests exit 0 for entry kinds, table rows, compact ordered lists, manifests, unchanged source hashes, and unchanged Release assembly. |
| 3 | Expose explicit headless configuration and operator evidence | pending | CLI integration tests exit 0 for reading/updating the library setting and structured effective-policy output. |
| 4 | Integrate capability contract and full gates | pending | CAP-0022, DOX, record/index checks, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, verifier tests, link check, and `git diff --check` exit 0. |

Keep at most one phase `in-progress`. This record is the sole progress authority
for configurable TODO projection; it does not advance CHG-0053.

## Phase 1 — Define persistent library policy and per-file override

**Goal:** The policy is explicit, schema-safe, and resolves deterministically
without affecting release paths.

1. Add the workspace conversion-policy shape, migration/default handling and a
   narrow read/write API owned by `dms-core`.
2. Add strict source override parsing and define precedence with the library-wide
   default.
3. Cover schema compatibility, invalid values, source preservation, manifest
   metadata boundaries and no Release-path invocation.

**Verification gate:** Focused core and CLI policy-resolution tests exit 0 for
disabled default, explicit library settings, override precedence, and invalid
values.

## Phase 2 — Project enabled TODO entries through standalone conversion

**Goal:** Enabled projection removes only defined visible entries from conversion
artifacts while leaving all controlled/release inputs unchanged.

1. Apply the resolved policy only during standalone CommonMark assembly.
2. Preserve remaining paragraph, heading, table, image and list fidelity; compact
   ordered list ordinals only after a projected item is removed.
3. Add manifest effective-policy evidence and structural DOCX/PDF regressions.

**Verification gate:** Conversion/domain and CLI tests exit 0 for entry kinds,
table rows, compact ordered lists, manifests, unchanged source hashes, and
unchanged Release assembly.

## Phase 3 — Expose explicit headless configuration and operator evidence

**Goal:** Operators can inspect and change the library default without hand-editing
workspace JSON.

1. Add an explicit `dms workspace` conversion-policy command with structured JSON
   output and no implicit target discovery.
2. Prove the command changes only the workspace policy, rejects invalid values,
   and never runs conversion or Release.
3. Document the command, per-file override, manifest evidence and retained-content
   handling under the owning contracts.

**Verification gate:** CLI integration tests exit 0 for reading/updating the
library setting and structured effective-policy output.

## Phase 4 — Integrate capability contract and full gates

**Goal:** Current contracts describe only the implemented conversion policy.

1. Update CAP-0022 and the Markdown conversion reference with present-tense
   behaviour and executable evidence.
2. Complete the DOX pass for config ownership and conversion-only boundaries.
3. Run the integration commands named in the phase table. Archive this record only
   when all gates pass.

**Verification gate:** CAP-0022, DOX, record/index checks, `cargo fmt --all --
--check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked`, verifier tests, link check, and `git diff
--check` exit 0.

## Out of scope

- Changing CHG-0053's full-corpus fidelity gate, source corpus, template, or
  retained artifacts.
- Applying TODO projection to desktop Release, approval, candidate, publish-root,
  or workflow state.
- Implicit source discovery, automatic policy promotion, source rewriting, or a
  generic filtering language.
- A desktop configuration UI unless Phase 1 proves one is required by an existing
  workspace-config contract.

## Related contracts

- [CAP-0022](../../product/capabilities/CAP-0022-headless-core-cli.md)
- [CHG-0053](CHG-0053-markdown-batch-conversion-retained-docx.md)
- [Markdown metadata and template variables](../../markdown-frontmatter-and-template-variables.md)
- [Architecture](../../architecture.md) and [privacy](../../privacy.md)
