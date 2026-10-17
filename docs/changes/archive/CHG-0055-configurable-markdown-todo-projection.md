# CHG-0055 — Configurable Markdown TODO projection

**Plan ID:** CHG-0055-configurable-markdown-todo-projection
**Created:** 2026-10-15
**Depends on:** archived CHG-0053 is the implementation receipt for retained-DOCX batch-conversion fidelity.
**Status:** done

| Field | Value |
| --- | --- |
| ID | CHG-0055 |
| Status | done |
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
| 2 | Project enabled TODO entries through standalone conversion | done (132 core and 25 CLI tests, scoped clippy, fmt and diff gates passed; fresh independent review: no findings; evidence below) | Conversion/domain and CLI tests exit 0 for entry kinds, table-cell whitespace projection, mixed-entry/cell rejection, compact ordered lists, manifests, unchanged source hashes, and unchanged Release assembly. |
| 3 | Expose explicit headless configuration and operator evidence | done (31 CLI tests, fmt, scoped clippy and diff gates passed; independent review found no findings) | CLI integration tests exit 0 for reading/updating the library setting and structured effective-policy output. |
| 4 | Integrate capability contract and full gates | done (corrected full gate passed; fresh independent review found no findings) | CAP-0022, DOX, record/index checks, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, verifier tests, link check, and `git diff --check` exit 0. |

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

Phase 1 DOX covered policy ownership, schema compatibility and resolution only;
Phase 2 owning contracts cover rendering and result reporting. The active index
matches this record.
Parent contracts and child indexes stay unchanged because ownership boundaries did
not move. Phase 1 left CAP-0022 unchanged because policy infrastructure alone
provided no filtering outcome or configuration command. Phase 4 remains responsible for
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

Phase 2 implementation and corrections passed the gate and fresh independent
review in worktree `dms-chg-0055-phase-2`, branch `agent/chg-0055-phase-2`, on
candidate `0c2fcf7`. The operator authorized `phased-plan-closeout --commit-ff`
for the reviewed boundary: commit the corrections and this done transition,
fast-forward `main`, rerun the phase gate, verify the target record and remove
the clean dedicated worktree. Phases 3/4 remain pending. No push or deployment
is authorized.

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
  Rendered link URLs, body metadata and literal raw HTML count as visible text.
  Definition titles and tight definition bodies are visible entries; loose
  definition paragraphs remain independent. Mixed diagnostics
  identify original entry-start line/column and kind without quoting source,
  including CRLF/Unicode inputs. Failure emits no pair and processing continues.
- Each result records `todo_projection: enabled|disabled`, or null when unresolved;
  normal-mode stderr reports the policy beside the path. No body is added to
  receipts. Input hashes, sources, assets, template/configuration, existing lock
  and publish tree are preserved. The candidate's existing test-only `sha2`
  dependency is retained; this reconciliation changes no manifest or lockfile.
- Traced paths: CLI `run_convert` → `ConversionPlan::run`/`convert_one` → private
  conversion assembler → `InstalledWord::export` → retained save/PDF export.
  Release remains `Workspace::release_candidate` → `PdfExporter::export` →
  desktop `LocalPdfExporter` → public `assemble_markdown_docx` → field fill/Office.
  Release receives neither conversion policy nor resolver invocation. Tests cover
  enabled/invalid overrides, mixed TODO release text and an approved release.

Final-review corrections:

- Visible-text matching retains closed-inline event offsets. A full marker
  immediately after bold, emphasis, strikethrough or inline code is a later marker,
  not an identifier suffix. No separator is inserted inside split markers such as
  `**TO**DO:`. Uninterrupted `NOTODO:`/`mytodo:` remain ordinary text, including
  the existing Unicode/digit/underscore boundary cases.
- Definition classification checks direct Paragraph children only. A nested
  quote's paragraph cannot make a tight body loose. Tight-body mixed diagnostics
  identify the definition entry start (line 8, column 1 in the synthetic fixture),
  including a later marker inside the quote. Whole-package comparisons cover
  leading-body removal with descendants, ordinary retained quote/body content,
  loose paragraphs in both orders and disabled bypass.
- Projection normalizes a retained definition title to balanced Paragraph events
  only when no nonempty definition body remains. This preserves the term/following
  paragraph boundary after removing tight bodies with quoted/list descendants or
  all loose-body paragraphs, without changing the shared block renderer. Retained
  tight/loose definitions, multiple bodies and sibling nested-list/quote paths
  retain whole-package equality against explicitly retained-source assembly.
- Core rejection cases exercise normal and dry-run conversion, exact original
  SHA-256, source-free diagnostics, no artifact pair, protected-input snapshots and
  continued batch processing. CLI cases cover adjacent bold/emphasis/code runs,
  nested-quote definitions, disabled bypass and unchanged source/configuration.
- Candidate package-part equality, synthetic PDF text transport, CRLF/Unicode
  locations, compact native numbering, table NBSP/grid tests and approved-Release
  isolation remain intact. The CLI test-only `sha2` dependency is unchanged.

Reproduced before correction (exit 101):

- `cargo test -p dms-core --test conversion todo_projection_rejects_mixed_entries_and_cells_without_pairs_or_source_leaks --locked --offline`
  rejected the test expectation because `**bold**TODO: PRIVATE_DROP` succeeded.
  After the inline fix alone, the same command exposed the nested-quote tight
  definition succeeding instead of failing at the definition-body start.
- `cargo test -p dms-core --test conversion todo_projection_matches_renderer_visible_retained_source_without_widening_entries --locked --offline`
  exposed the retained TODO definition and its quoted descendant.
  Restoring the complete continuation fixture reproduced exit 101 with
  `Kept termKept final` in one OOXML paragraph instead of two. The final regression
  explicitly asserts the two retained paragraphs, no removed body/descendant, and
  whole-package equality. A broad flush-on-removal attempt failed the existing
  loose-definition equality check and was replaced by orphan-title normalization.

Continuation correction gates passed on the current source/tests (each command exit 0):

- `cargo test -p dms-core --test conversion_policy --test conversion --test markdown_template --test lifecycle --test smoke_contract --test workspace --test policies --locked --offline`
  — 132 passed: conversion 35, conversion-policy 6, template 30, lifecycle 32,
  smoke 4, workspace 14 and policies 11.
- `cargo test -p dms-cli --test cli --locked --offline` — 25 passed.
- `cargo clippy -p dms-core -p dms-cli --all-targets --locked --offline -- -D warnings`.
- `cargo fmt --all -- --check` (after `cargo fmt --all`).
- `git diff --check`.

Record/index/CAP-reference assertions cover the Phase 2 done transition,
Phases 3/4 pending, and the active index and CAP-0022 link. The reviewed
implementation fingerprint remains unchanged; no manifest or lockfile correction
is included against the candidate.
Fresh independent review reported no findings (receipt below). Full-workspace clippy is
unproven for this corrected candidate and reserved for Phase 4, not attempted in
this continuation. The final foreground gate rerun explicitly exported worktree-local
`CARGO_TARGET_DIR="$PWD/target"` and `TMPDIR="$PWD/target/phase-2-tmp"`; all commands
completed successfully. An earlier successful background run used the harness's
scratch TMPDIR instead; it is not the worktree-local temporary-path evidence.

Evidence limits: structural DOCX assertions cover preserved grid, NBSP cells,
native list instances, starts, continuations, formatting and retained images.
PDF regressions parse synthetic PDFs generated from the saved DOCX's actual text;
they prove projected/retained text transport, not installed-Word pagination,
visual fidelity or tagged PDF table/list structure. Native Word/desktop execution
and the full-workspace Phase 4 gates were not run.

### Phase 2 correction receipt — definition paragraph boundaries

**State:** paragraph-boundary correction and all listed correction gates passed;
fresh independent review approved with no findings. The full failing fixture is restored, not
bounded after the quote. The shared renderer ignored definition-list tags and
relied on the removed quote's Paragraph end to close the term. Conversion-only
orphan-title normalization restores that boundary without modifying disabled
conversion, Release, or the shared renderer.

Whole-package regressions cover quoted/list descendants, a following paragraph
or definition, ordinary retained tight definitions with nested blocks, multiple
bodies, all-removed bodies, loose paragraphs in both orders, sibling nested list
and quote removal, and disabled conversion against public Release assembly.
Existing inline adjacency, split markers, identifier exclusions, direct-child
definition classification, source-free diagnostics, exact SHA-256, CRLF/Unicode,
no-pair, synthetic PDF, numbering, NBSP/grid and approved-Release tests all pass.

Fresh independent read-only review (`deleg_fb8aad4d`, reviewer `sa-0-8e16e138`)
covered the complete Phase 2 candidate against `727ac94`, including committed
`0c2fcf7` and all uncommitted corrections, not only the continuation patch. The
reviewer inspected projection/assembly, conversion wiring, CLI output, diagnostics,
tests and owning contracts, and reported **No findings; approve for separate
Phase 2 closeout**. Its independent rerun reported the two focused regressions,
132 core tests, 25 CLI tests, scoped clippy, formatting and diff checks green.
The executor's exact gate commands/results above are independently recorded;
neither review nor synthetic tests establish native Word fidelity.

Reviewed implementation diff SHA-256:
`2b0ed65ee2220ca85faa454a8aea0fb3cf76da5daf0fc26467e2dc02b912191f`.
Computed from `git diff --binary --no-ext-diff 727ac94 -- .
':!docs/changes/active/CHG-0055-configurable-markdown-todo-projection.md'` and
rechecked unchanged after review. The CHG was reviewed too, but excluded from
this implementation fingerprint to permit this review receipt and progress-only
updates; no source, tests, manifests or owning DOX changed after review.

**Stop/handoff:** Phase 2 is done; Phases 3/4 stay pending. Closeout uses only
the operator-authorized fast-forward route described above. Stop after target-ref
verification and dedicated-worktree removal; do not begin Phase 3 or push/deploy.
The recovery patch under `target/` is not implementation input or review evidence.

DOX: the core owning contract states direct-child definition classification,
closed-inline marker boundaries and orphan-title paragraph normalization.
CLI reporting and diagnostic contracts already
cover these regressions.
Root/parent contracts, child indexes and the active CHG index remain unchanged:
ownership/structure did not move. CAP-0022 and the operator reference remain
assigned to Phase 4; no Phase 3 command or Phase 4 integration is included.

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

### Phase 3 evidence and review boundary

Phase 3 is **done**; Phase 4 remains **pending**. The Phase 2 receipt's
pending-phase statements describe its handoff boundary, not the current phase
table. Commit `c962c1f` integrates the verified, reviewed Phase 3 boundary on
`main`; the dedicated worktree is removed. No push or deployment is authorized.

- `workspace conversion-policy` requires `--edit-root`; omission of
  `--todo-projection` inspects, while explicit `enabled|disabled` updates through
  the typed core setter and `Workspace::save`. JSON serializes only the policy.
- Six new CLI integration tests cover default/missing-policy inspection,
  enabled/disabled persistence/readback, normal/JSON output, invalid/missing
  arguments, explicit targets, malformed stored policy rejection, and schema-21
  opening with exact migration backup. Whole-tree snapshots preserve source,
  external input, existing publish content, instructions and advisory lock;
  workspace JSON differs only in policy on a current synchronized workspace.
- A separate explicit conversion dry-run consumes the command's persisted
  default and proves both override directions in effective-policy JSON.
  Existing conversion tests retain source-hash, mixed-entry and diagnostic
  evidence. No conversion, core, exporter or Release implementation changed.
- Ordinary core opening can migrate metadata or synchronize managed instructions;
  inspection is not advertised as universally read-only. The operator reference
  documents this boundary, precedence, manifest evidence and confidential
  retained DOCX/PDF handling without granting release authority.

Passed on this candidate (each exit 0), with worktree-local
`CARGO_TARGET_DIR="$PWD/target"` and `TMPDIR="$PWD/target/phase-3-tmp"`:

- `cargo test -p dms-cli --test cli --locked` — 31 passed, 0 failed.
- `cargo fmt --all -- --check`.
- `cargo clippy -p dms-cli --all-targets --locked -- -D warnings`.
- `git diff --check`.

The coordinator independently reran the same four gates after implementation:
`cargo test -p dms-cli --test cli --locked` reported 31 passed, 0 failed; the
format, scoped clippy and diff gates also exited 0.

Fresh independent read-only review of the full Phase 3 diff against `6245faf`
reported **No findings**. The configured `grok` reviewer selector was unavailable
because xAI OAuth lacked an access token; the operator explicitly selected the
accepted session-only `astra` selector instead. The reviewer inspected the actual
diff, CLI tests, open/save boundary and DOX contracts without modifying the
worktree. This is not full-workspace or native-Word evidence.

An intermediate CLI run failed because the new conversion fixture omitted its
required root confidentiality policy; the fixture was corrected through the core
API. Clippy then rejected five uninlined assertion formats; all were corrected
before the passing rerun. No product workaround or gate suppression was added.

DOX pass: updated `crates/dms-cli/AGENTS.md` and the operator-reference ownership
entry in `docs/AGENTS.md`. Root/crates/changes parent contracts and child indexes
remain unchanged because ownership boundaries and lifecycle rules did not move.
CAP-0022 is intentionally untouched for Phase 4. Full-workspace gates and native
Word fidelity remain outside this phase. No commit, push, deployment or Phase 4
work is included.

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

### Phase 4 evidence and closeout

Phase 4 is **done** and this record is archived as a closed implementation
receipt. The first final review found that CAP-0022 stated mixed-entry/cell
rejection without limiting it to enabled projection. The correction states that
disabled projection keeps mixed markers as ordinary visible body text. The
corrected freeze passed the full gate, and a fresh independent review found no
findings. No push or deployment is authorized.

- CAP-0022 records present-tense outcomes for the conversion policy:
  `dms workspace conversion-policy` (inspect/persist the library default;
  outcome 13) and conversion-only TODO projection (outcome 14). Its `Tests`
  field links `crates/dms-core/tests/conversion_policy.rs`; the executable-
  evidence paragraph names the projection regressions.
- A prior independent review of the Phase 4 candidate
  found the new `workspace conversion-policy` outcome inserted as outcome 3,
  which shifted the pre-existing standalone-conversion outcome from 8 to 9 and
  broke planned CHG-0054's cross-reference to CAP-0022 outcome 8. The new
  outcome now sits at 13, immediately before the TODO-projection outcome at 14,
  so every pre-existing CAP-0022 outcome keeps its established number (outcomes
  1–12 are byte-identical to the record before this phase) and CHG-0054 stays
  untouched.
- The operator reference (`markdown-frontmatter-and-template-variables.md`)
  already carried the Phase 3 command/override/receipt contract. It was re-read
  against the current implementation and left accurate, so no fact-changing edit
  was needed and no unrelated facts were overwritten.
- DOX pass: the owning contracts (`crates/dms-core/AGENTS.md`,
  `crates/dms-cli/AGENTS.md`, the operator-reference ownership row in
  `docs/AGENTS.md`) already state the conversion-policy and projection contracts
  from Phases 1–3. Ownership, structure and lifecycle rules did not move, so no
  parent/child AGENTS.md or Child DOX Index changed in Phase 4.

Gates on the corrected freeze, run from the dedicated Phase 4 worktree with
worktree-local `CARGO_TARGET_DIR="$PWD/target"` and
`TMPDIR="$PWD/target/phase-4-tmp"` (each exit 0):

- `cargo fmt --all -- --check`.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- `cargo test --workspace --locked` — 304 passed, 0 failed.
- verifier tests (`uv run --no-project --with markdown-it-py==4.0.0 --with
  mdit-py-plugins==0.5.0 --with PyMuPDF==1.26.5 python -B -m unittest discover
  -s scripts -p test_markdown_batch.py -v`) — 82 tests OK, 2 native-host tests skipped.
- `python3 …/check-md-links.py --format summary .` — strict mode, 0 issues.
- `git diff --check`.

Manual record/index checks (no `records:check` script exists) confirm the active
index lists exactly the active CHGs, no active CHG is marked done, and CAP-0022
exists and is implemented. The final reviewer independently inspected the
corrected frozen diff and reported no findings. Rust and verifier evidence is
synthetic/Linux and does not establish installed-Word fidelity.

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
