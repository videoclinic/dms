# CHG-0040 Phase 3 task packet

**Prepared:** 2026-10-11T18:17:40+02:00
**Status:** implemented, reviewer-cleared, integrated, and archived without a commit
**Progress authority:** [CHG-0040](../archive/CHG-0040-document-type-id-migration.md), Phase 3

## Outcome

Close CHG-0040 by publishing current product contracts and canonical wireframe evidence for the implemented retained document-type ID migration. Current document control and history retain source IDs; only future candidates/release profiles snapshot resolved replacement IDs; every Document control view resolves known IDs through the current catalogue as `Display label (type-id)`.

## Scope

- Update CAP-0001, CAP-0002, CAP-0013, and CAP-0015 only where Phase 1/2 changed their existing outcomes.
- Update the canonical `docs/product/wireframes/generate.mjs` definitions for CAP-0013 and CAP-0015. Regenerate HTML, manifest, and index; produce both PNG exports; visually inspect both rendered PNGs.
- Record passing Phase 3 evidence; mark the final CHG phase done; set CHG-0040 done; move it to `docs/changes/archive/`; refresh `docs/changes/README.md`.

## Non-goals

- No runtime-code changes beyond already integrated Phase 1/2 changes.
- No changes to CAP-0008 or confidentiality ADR-0010, PDF filename semantics, Markdown controlled frontmatter, document type deletion/renaming, historical data, or any other active CHG.
- No commit, push, deploy, credential access, or external state mutation.

## Starting workspace

- Primary checkout: `/mnt/c/Users/RaphaelBossek/src/dms`
- Branch and HEAD: `main` at `a75e524ebb192c51026273d6405b4c427aef9203`
- Current tracked overlay SHA-256: `27e2fb1294e3624192f138b7d10fe102bfcdb27d536f4e5fc8465660f7898a8a`
- The overlay includes verified Phase 1/2 code, their CHG evidence, and the Phase 3 in-progress transition. Preserve it in a dedicated worktree; do not overwrite the primary checkout.

## Relevant contracts

- `docs/changes/active/CHG-0040-document-type-id-migration.md`, Phase 3 and its gate.
- `docs/product/capabilities/CAP-0001-local-folder-dms.md`, outcome 8.
- `docs/product/capabilities/CAP-0002-document-lifecycle.md`, candidate/release snapshot outcome only.
- `docs/product/capabilities/CAP-0013-library-maintenance.md`, outcome 12.
- `docs/product/capabilities/CAP-0015-document-control-data.md`, outcomes 1, 9, and 11.
- `docs/product/wireframes/AGENTS.md`: edit `generate.mjs` only; do not hand-edit generated HTML, manifest, index, or PNG exports.

## Acceptance criteria

1. CAP-0001 identifies retained document-type migration mappings as workspace metadata without changing privacy/credential claims.
2. CAP-0002, CAP-0013, and CAP-0015 precisely distinguish retained current/historical IDs from future candidate/release replacement snapshots; no CAP implies frontmatter, PDF, or confidentiality behavior changed.
3. CAP-0015 states `Display label (type-id)` current-catalogue rendering and label-renames-as-presentation-only. CAP-0013 defines retained-ID mapping/reference protection instead of simple add/rename/disable parity.
4. CAP-0013's canonical wireframe shows the Document defaults Future release type selector/action. CAP-0015 shows current-control and later candidate/release profile types as `Display label (type-id)`; synthetic content only.
5. Generated `html/`, `index.html`, `manifest.json`, and CAP-0013/CAP-0015 PNGs derive from `generate.mjs`; both screenshots are visually inspected for the new controls and labels.
6. CHG-0040 final phase and top-level status are truthful; the record is archived and `docs/changes/README.md` indexes it as done. No active CHG is marked done.
7. All Phase 3 gates exit 0: generator; both specified Chrome screenshot commands; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; and `git diff --check`.

## Implementer report contract

Read all applicable AGENTS.md files before editing. Keep the diff minimal. Return changed paths, generated-artifact evidence, visual-inspection result, commands with exit statuses, decisions, and remaining risks. Do not commit, push, deploy, access secrets, or mutate external systems.

## Review contract

A fresh configured reviewer-model session must inspect the final worktree diff, the archived CHG/index changes, CAP claims, generated artifacts, screenshot evidence, and all gate outcomes. It must remain read-only and end with numbered introduced-defect findings or `No findings`.
