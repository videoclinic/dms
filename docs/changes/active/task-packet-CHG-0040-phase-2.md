# CHG-0040 Phase 2 task packet

**Prepared:** 2026-10-11T16:57:18+02:00
**Status:** implementation, independent verification, and configured reviewer-model review complete; Phase 2 integrated without a commit
**Progress authority:** [CHG-0040](CHG-0040-document-type-id-migration.md), Phase 2

## Outcome

Finish CHG-0040 Phase 2: Configuration → Document defaults explicitly maps one retained document-type ID to a different enabled type for future candidates, while Library Document control data renders catalogue-resolved type IDs as `Display label (type-id)` in mutable and immutable views. The core owns validation and persistence; the frontend only submits explicit IDs and refreshes the returned snapshot.

## Scope

- Complete the narrow Tauri `migrate_document_type` adapter command, registration, and focused adapter coverage begun in `crates/dms-desktop/src/lib.rs`.
- Extend `crates/dms-desktop/ui/configuration.mjs` and its tests with Document defaults migration rows patterned after the confidentiality-type migration control, without creating a secondary document-type page.
- Extend `crates/dms-desktop/ui/library.mjs` and its tests so current control, the editable selector, and current immutable release profile display `Display label (type-id)` from `detail.document_types`.
- Keep the visible, ID-only fallback for a missing catalogue entry in corrupt/legacy detail responses.
- Update CHG-0040 Phase 2 evidence only after every Phase 2 gate passes.

## Non-goals

- Do not change `dms-core`, schema migration, candidate/release semantics, Markdown projection, PDF filenames, confidentiality rules, CAPs, wireframes, or Phase 3 records.
- Do not rewrite live document control, candidates, release history, audit evidence, or existing source IDs.
- Do not commit, push, deploy, access credentials, or make external changes.

## Starting workspace

- Repository: `/mnt/c/Users/RaphaelBossek/src/dms`
- Branch: `main`
- HEAD: `a75e524ebb192c51026273d6405b4c427aef9203` (`feat(dms-core): support document type migrations`)
- Merge base with `origin/main`: `7740a538c3883397a5b24c90e54f13261b1472c5`
- Existing unrelated-or-in-progress overlay must be preserved. At preparation it modifies:
  - `crates/dms-desktop/src/lib.rs` — the adapter command, handler registration, and a partial persistence assertion
  - `docs/changes/active/CHG-0040-document-type-id-migration.md` — Phase 2 set to `in-progress`
- The exact prepared overlay is identified by SHA-256 `aff2225792a432a98277cadf80250e327a66c8c5503f1bcac67a493fb7cc4ef5` over `git diff --binary -- crates/dms-desktop/src/lib.rs docs/changes/active/CHG-0040-document-type-id-migration.md`. It passed `git diff --check` at preparation.

## Execution isolation

The implementation worktree is `/home/raphael/.hermes/profiles/hermes-vc/cache/scratch/dms-chg0040-phase2` on `hermes/chg-0040-phase2`, seeded from the recorded HEAD plus the reviewed primary overlay. After the configured reviewer-model returned `No findings`, its reviewed delta was applied to the primary checkout without a commit. The implementer used the configured session-only `astra` selector after `/new implement-document-type-migration-ui`; the reviewer used a fresh configured `grok` session. The durable evidence and exact outcomes are in the handoff.

## Relevant code and contracts

- `crates/dms-core/src/catalogues.rs`: `Workspace::migrate_document_type` is the sole validation/persistence authority; it is already present.
- `crates/dms-desktop/src/lib.rs`: `mutate_workspace_configuration` is the existing configuration-save path. The partial `migrate_document_type` command must continue to use it and must appear in `generate_handler!`.
- `crates/dms-desktop/ui/configuration.mjs`: `documentTypesMarkup` currently renders only add/rename/enable rows; `configurationMutationRequest` currently has only the confidentiality migration request mapper.
- `crates/dms-desktop/ui/library.mjs`: `selectionMarkup` currently renders raw `document_type` IDs in the current control summary and immutable release profile, and only labels in the edit selector.
- `crates/dms-desktop/ui/configuration.test.mjs` and `crates/dms-desktop/ui/library.test.mjs`: extend the existing fixture style; do not introduce a test framework.
- `crates/dms-desktop/AGENTS.md`: frontend cannot duplicate core rules; Document defaults remains the document-type surface; explicit source/replacement IDs only.
- `docs/changes/active/CHG-0040-document-type-id-migration.md`: Phase 2 steps and gate are binding.

## Acceptance criteria

1. A registered `migrate_document_type(edit_root, source_type_id, replacement_type_id)` Tauri command invokes the core operation via the existing configuration mutation path and returns a refreshed `WorkspaceConfiguration`; successful persistence is adapter-tested.
2. Each Document defaults document-type row has a `Future release type` selector with only other enabled types, retains its source ID in an explicit submitted field, and disables its migration action when no eligible target exists.
3. The action says `Migrate future candidates` before a mapping and `Update future-candidate migration` when one exists. The form submits exactly trimmed `sourceTypeId` and `replacementTypeId` to `migrate_document_type`; JavaScript does not duplicate core target/reference/cycle validation.
4. Current document-control summary, document-type edit selector, and current immutable release profile render a known ID as `Display label (type-id)`. A later catalogue-label rename therefore changes presentation only. Unknown/missing catalogue entries visibly fall back to the retained ID only.
5. Existing type IDs remain values and persistence keys; no labels are written into document control, candidates, releases, audit records, Markdown frontmatter, or PDF filenames.
6. Phase 2 gates all exit 0:
   - `cargo test -p dms-desktop`
   - `node --test crates/dms-desktop/ui/configuration.test.mjs`
   - `node --test crates/dms-desktop/ui/library.test.mjs`
7. The final diff is restricted to this phase plus truthful CHG evidence; it preserves unrelated primary-checkout changes and passes `git diff --check`.

## Implementer report contract

Read the applicable AGENTS chain before edits. Inspect existing code and tests before choosing APIs. Report exact changed paths, each command and exit status, design decisions, assumptions, and remaining risks. Do not commit, push, deploy, access secrets, or mutate external state.

## Later review packet requirements

After implementation stops at a stable checkpoint, the orchestrator must independently inspect the worktree diff and rerun the Phase 2 gates before review. A fresh read-only session named `review-document-type-migration-ui` must receive: this packet; base `a75e524ebb192c51026273d6405b4c427aef9203`; final changed-path list; final diff reference; and actual test outputs. Switch it to the configured session-only `grok` selector and confirm acceptance before the review prompt. Require only introduced defects, ordered by severity with `path:line`, impact, trigger, and correction, or an explicit `No findings` verdict.
