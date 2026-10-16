# CHG-0055 — Configurable Markdown TODO projection

**Plan ID:** CHG-0055-configurable-markdown-todo-projection
**Created:** 2026-10-15
**Depends on:** archived CHG-0053 is the implementation receipt for retained-DOCX batch-conversion fidelity.
**Status:** in-progress

| Field | Value |
| --- | --- |
| ID | CHG-0055 |
| Status | in-progress |
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
- With projection enabled, a visible paragraph, heading, list item or code block
  is omitted only when its first visible content is `TODO:` (case-insensitive).
  For a list item, the marker follows the list marker; for example,
  `- TODO: ...` and `4. TODO: ...` are eligible. `**TODO:**` is accepted for
  existing sources, but `TODO:` is the canonical source form. Frontmatter is not
  visible content. Surviving ordered list items are renumbered without gaps.
- A `TODO:` marker following substantive visible text is an invalid mixed entry:
  that source fails conversion with a precise diagnostic and no artifact pair is
  emitted for it. The converter never silently removes the substantive text. An
  author must split the content into a retained entry and a standalone TODO entry.
- A Markdown table cell whose first visible content is `TODO:` is projected to one
  non-breaking space in the DOCX/PDF cell. The row, its other cells and the table
  grid remain intact. A cell containing substantive visible text before `TODO:`
  is an invalid mixed cell and fails that source; a table row is never removed
  merely because one cell is a TODO.
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
| 1 | Define persistent library policy and per-file override | done (gate passed; independent review found no findings; evidence below) | Focused core and CLI policy-resolution tests exit 0 for disabled default, explicit library settings, override precedence, and invalid values. |
| 2 | Project enabled TODO entries through standalone conversion | pending | Conversion/domain and CLI tests exit 0 for entry kinds, table-cell whitespace projection, mixed-entry/cell rejection, compact ordered lists, manifests, unchanged source hashes, and unchanged Release assembly. |
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

### Phase 1 evidence and boundary

- Schema 22 persists `ConversionPolicy` with a default-disabled `TodoProjection`.
  `Workspace::conversion_policy` / `set_conversion_policy` use the ordinary
  explicit `save` boundary. Migration retains the exact prior metadata backup;
  missing policy defaults disabled, while present malformed policy fails closed.
- `ConversionPolicy::resolve_todo_projection` and the read-only `ConversionPlan`
  resolver reuse strict flat conversion frontmatter. Exact lowercase
  `enabled` / `disabled` scalars (including ordinary quoted scalars) override
  the library default; omission inherits. Invalid/duplicate/nested values fail
  with source-free diagnostics. No new parser, dependency or CLI command.
- Read-only conversion continues to accept schemas 18, 19 and 21 as well as 22;
  schema 20 remains unsupported by conversion, preserving the existing allowlist.
  The workspace store retains its existing older-schema migration support.
- Core tests cover persistence-only mutation, v21 migration, invalid stored
  shapes, source-owned override preservation during controlled-key sync,
  unchanged source hashes and manifest metadata, and an approved Release with
  an invalid conversion override and retained TODO body. CLI tests exercise
  resolution through the public core plan and subprocess configuration rejection.
- Resolution is not wired into `ConversionPlan::run` or either assembler.
  Thus even `enabled` does not remove content in this phase, invalid overrides
  are rejected by the explicit resolver rather than the CLI conversion run,
  and manifests do not claim an applied policy. Those integrations belong to
  Phase 2; operator configuration belongs to Phase 3.

Passed in the dedicated Phase 1 worktree (each exit 0):

- `cargo test -p dms-core --test conversion_policy --test conversion --test lifecycle --test workspace --test policies --locked`
- `cargo test -p dms-cli --test cli --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `git diff --check`

DOX: core and CLI owning contracts describe policy ownership, schema compatibility
and the explicit resolution-only boundary; the active index matches this record.
Parent contracts and child indexes stay unchanged because ownership boundaries did
not move. CAP-0022 stays unchanged: this is policy infrastructure, not an available
TODO-filtering outcome or configuration command. Phase 4 remains responsible for
the completed capability contract and integration evidence.

Independent autonomous review of the frozen Phase 1 diff reported no findings.
The reviewed phase state is integrated on `main`; no push or deployment occurred.

Limitations: Rust evidence is synthetic/Linux, not installed-Word fidelity. Older
binaries cannot open schema 22. Phases 2–4 remain pending by operator scope.

## Phase 2 — Project enabled TODO entries through standalone conversion

**Goal:** Enabled projection removes only defined visible entries from conversion
artifacts while leaving all controlled/release inputs unchanged.

1. Apply the resolved policy only during standalone CommonMark assembly.
2. Preserve remaining paragraph, heading, table, image and list fidelity; compact
   ordered list ordinals only after a projected item is removed. Project eligible
   table cells to non-breaking spaces and reject mixed entries/cells without
   discarding their substantive content.
3. Add manifest effective-policy evidence, invalid-mixed-entry diagnostics and
   structural DOCX/PDF regressions.

**Verification gate:** Conversion/domain and CLI tests exit 0 for entry kinds,
table-cell whitespace projection, mixed-entry/cell rejection, compact ordered
lists, manifests, unchanged source hashes, and unchanged Release assembly.

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
- [CHG-0053](../archive/CHG-0053-markdown-batch-conversion-retained-docx.md)
- [Markdown metadata and template variables](../../markdown-frontmatter-and-template-variables.md)
- [Architecture](../../architecture.md) and [privacy](../../privacy.md)
