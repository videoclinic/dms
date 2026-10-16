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
| 2 | Project enabled TODO entries through standalone conversion | done (core conversion/markdown-template/lifecycle, CLI and clippy gates passed; final autonomous review: no findings) | Conversion/domain and CLI tests exit 0 for entry kinds, table-cell whitespace projection, mixed-entry/cell rejection, compact ordered lists, manifests, unchanged source hashes, and unchanged Release assembly. |
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
- Phase 1 covers resolution only. Integration into `ConversionPlan::run`,
  conversion assembly and manifests belongs to Phase 2; operator configuration
  belongs to Phase 3. The phase table and Phase 2 evidence below own that progress.

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
binaries cannot open schema 22.

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

### Phase 2 evidence and review boundary

Implementation is confined to worktree `dms-chg-0055-phase-2`, branch
`agent/chg-0055-phase-2`, rebased onto local `main` without conflicts using
`git rebase --autostash main` (exit 0). The implementation gate passed and the
final autonomous review of the corrected frozen diff reported no findings.
Phases 3/4 remain pending. No push or deployment occurred.

- `ConversionPlan::convert_one` hashes the original bytes, resolves policy, and
  passes it only to `assemble_markdown_conversion_docx`. Enabled assembly projects
  balanced CommonMark events before numbering, image loading or pair reservation.
  Disabled and Release assembly retain their ordinary streaming parser path.
- Eligible paragraphs/headings/code blocks and whole list items disappear;
  table cells become exactly one NBSP with grid/rows intact. Native ordered lists
  compact from their original start, including removed first/middle/final items;
  nested lists and separate restarts retain independent numbering instances.
- Decisions pinned by tests: inline formatting does not hide markers; fence
  language and image alt text are not code/body content. Images and rendered
  task checkboxes are substantive. A list item's own continuation text belongs
  to the item (including quoted continuations), while nested lists/table cells
  are independent entries. Removing a TODO item removes its descendants, not
  unmarked ancestors; removing a heading does not remove its following section.
  Rendered link URLs and literal raw HTML count as visible text. Mixed diagnostics
  identify original entry-start line/column and kind without quoting source,
  including CRLF/Unicode inputs. Failure emits no pair and processing continues.
- Each result records `todo_projection: enabled|disabled`, or null when unresolved;
  normal-mode stderr reports the policy beside the path. No body is added to
  receipts. Input hashes, sources, assets, template/configuration, existing lock
  and publish tree are preserved. `sha2` is added only as a CLI test dependency,
  reusing the locked workspace version; no runtime dependency/version changes.
- Traced paths: CLI `run_convert` → `ConversionPlan::run`/`convert_one` → private
  conversion assembler → `InstalledWord::export` → retained save/PDF export.
  Release remains `Workspace::release_candidate` → `PdfExporter::export` →
  desktop `LocalPdfExporter` → public `assemble_markdown_docx` → field fill/Office.
  Release receives neither conversion policy nor resolver invocation. Tests cover
  enabled/invalid overrides, mixed TODO release text and an approved release.

Candidate audit: the pre-existing implementation passed its original tests but
missed mixed TODO text in raw HTML blocks rendered as literal paragraphs.
The focused mixed-entry regression exited 101 for
`<p>PRIVATE_KEEP TODO: PRIVATE_DROP</p>` before the fix. Projection now checks
`Tag::HtmlBlock` as paragraph text; core and CLI regressions pass. The bounded
read-only audit's other concerns are pinned by tests: quoted/loose continuations
belong to their item, rendered URLs obey the literal mixed-marker rule, and an
unmarked empty parent matches assembly of the explicitly retained source.
Full DOCX package-part equality also covers nested lists/tables, formatted and
entity/escaped markers, images, non-TODO raw HTML and code-fence metadata.
This is not a claim of independent approval of the final frozen diff.

Broader candidate verification before the bounded review correction (each command
exit 0; not rerun in full for the correction):

- `cargo fmt --all`
- `cargo test -p dms-core --test conversion_policy --test conversion --test markdown_template --test lifecycle --test smoke_contract --test workspace --test policies --locked --offline`
  — 33 conversion, 6 conversion-policy, 30 template, 32 lifecycle, 4 smoke,
  14 workspace and 11 policy tests passed.
- `cargo test -p dms-cli --test cli --locked --offline` — 24 passed.
- `cargo test -p dms-export --locked --offline` — 18 passed; 0 doc tests.
- `cargo clippy -p dms-core -p dms-cli -p dms-export --all-targets --locked --offline -- -D warnings`
- `cargo fmt --all -- --check`
- `git diff --check`

A batched verification wrapper timed out at 300 seconds (tool exit -1); the
complete final command chain was rerun directly and returned exit 0.

The confirmed frozen-review finding is corrected by treating
`Tag::MetadataBlock(_)` and `Tag::DefinitionListTitle` as paragraph entries in the
existing enabled projection. Body metadata blocks (both `---` and `+++`, after
conversion frontmatter) and definition titles now receive leading removal and
mixed-entry rejection before pair reservation. Core tests assert retained DOCX
and synthetic PDF text, disabled/Release preservation, original source hashes,
no failed pair/export call, and source-free diagnostics. CLI dry-run tests cover
leading acceptance, mixed rejection, disabled acceptance and no output/source
leak. The new core removal and CLI mixed regressions each exited 101 before the
fix.

Correction verification (each command exit 0):

- `cargo test -p dms-core --test conversion --locked --offline` — 34 passed.
- `cargo test -p dms-cli --test cli --locked --offline` — 25 passed.
- `cargo fmt --all -- --check` (after `cargo fmt --all`).
- `git diff --check`.

Build/test paths are worktree-local: `CARGO_TARGET_DIR="$PWD/target"` and
`TMPDIR="$PWD/target/phase-2-tmp"`. The initial core run passed 33 tests but failed
the existing case-collision test on case-insensitive DrvFS. Enabling case
sensitivity only on `target/phase-2-tmp` with `fsutil.exe file
setCaseSensitiveInfo` allowed the unchanged full conversion suite to pass.
DOX recheck: existing paragraph/projection and CLI contracts remain sufficient;
no owning-contract or child-index changes are needed for this correction.

Evidence limits: structural DOCX assertions cover preserved grid, NBSP cells,
native list instances, starts, continuations, formatting and retained images.
PDF regressions parse synthetic PDFs generated from the saved DOCX's actual text;
they prove projected/retained text transport, not installed-Word pagination,
visual fidelity or tagged PDF table/list structure. Native Word/desktop execution
and the full-workspace Phase 4 gates were not run.

DOX: core/CLI owning contracts now describe applied projection, receipts and these
verification limits. Root/parent contracts and child indexes remain unchanged:
ownership/structure did not move. CAP-0022, operator reference and Phase 3 docs
remain untouched by explicit scope; Phase 4 owns capability integration.

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
