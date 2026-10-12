# CHG-0040 Phase 2 durable handoff

**Prepared:** 2026-10-11T16:57:18+02:00
**Checkpoint:** Phase 2 implementation integrated after independent verification and a configured reviewer-model `No findings` verdict

## Goal and constraints

Complete CHG-0040 Phase 2, as defined in [the task packet](task-packet-CHG-0040-phase-2.md) and [the active CHG](../archive/CHG-0040-document-type-id-migration.md). Implement the DMS Desktop adapter/configuration/Library presentation slice only. Preserve retained IDs and core-owned validation. Do not commit, push, deploy, access secrets, alter external state, or begin Phase 3.

## Workspace and baseline

- Repository: `/mnt/c/Users/RaphaelBossek/src/dms`
- Branch: `main`
- HEAD: `a75e524ebb192c51026273d6405b4c427aef9203`
- Merge base with `origin/main`: `7740a538c3883397a5b24c90e54f13261b1472c5`
- Active authority: `docs/changes/active/CHG-0040-document-type-id-migration.md`, Phase 2 is `in-progress`.
- Implementation worktree: `/home/raphael/.hermes/profiles/hermes-vc/cache/scratch/dms-chg0040-phase2` on branch `hermes/chg-0040-phase2`.
- Implementer session: pane `w6S:p2`, claimed as `exec`, fresh goal `implement-document-type-migration-ui`, selector accepted as `astra` (`gpt-6-astra-900k`).
- Reviewer sessions: same executor pane, fresh read-only goals `review-document-type-migration-ui` and `review-document-type-migration-ui-retry`, selector accepted as `grok` (`grok-4.7`). The first provider attempt failed before a verdict; the authorized fresh retry completed.

## Initial working-tree state

At preparation, the primary checkout had these existing modified paths:

- `crates/dms-desktop/src/lib.rs`
- `docs/changes/active/CHG-0040-document-type-id-migration.md`

The two-path binary diff SHA-256 was `aff2225792a432a98277cadf80250e327a66c8c5503f1bcac67a493fb7cc4ef5`. It represents a partial Phase 2 overlay: the desktop command and handler registration exist, its persistence test was extended, and Phase 2 was marked `in-progress`. It is not independently verified implementation evidence.

Before a later dispatch, re-run:

    git status --short
    git diff --check -- crates/dms-desktop/src/lib.rs docs/changes/active/CHG-0040-document-type-id-migration.md
    git diff --binary -- crates/dms-desktop/src/lib.rs docs/changes/active/CHG-0040-document-type-id-migration.md | sha256sum

If the overlay hash or paths differ, inspect and refresh this handoff and the task packet before creating a worktree. Never discard or overwrite primary-checkout changes.

## Grounded discovery

- The graph finds the core authority at `crates/dms-core/src/catalogues.rs:47-64` and the partial adapter at `crates/dms-desktop/src/lib.rs:1007-1017`.
- The core migration is covered by `crates/dms-core/tests/policies.rs:176-267` and `crates/dms-core/tests/lifecycle.rs:1935-2046`; Phase 2 must not alter that domain implementation.
- `crates/dms-desktop/ui/configuration.mjs:220-225` renders document types without migration controls. Its `configurationMutationRequest` at `:450-569` contains the confidentiality migration mapper to follow structurally, not semantically.
- `crates/dms-desktop/ui/library.mjs:1220-1252` emits raw IDs in the immutable release profile and current-control summary, with label-only selector options. The detail response already includes `document_types`; no data-model change is needed.
- `crates/dms-desktop/src/lib.rs:5176-5266` contains the partial adapter persistence test. Existing UI test fixtures are `configuration.test.mjs:24-80` and `library.test.mjs:467-634`.

## Implementation checkpoint

The worktree was seeded from `a75e524ebb192c51026273d6405b4c427aef9203` plus the verified primary overlay (`aff2225792a432a98277cadf80250e327a66c8c5503f1bcac67a493fb7cc4ef5`). Its current tracked diff SHA-256 is `9a0b1c3b7132e89c2de820c873c348212a5c47159b631888954ef99f3036531f`.

Changed worktree paths:

- `crates/dms-desktop/src/lib.rs`
- `crates/dms-desktop/ui/configuration.mjs`
- `crates/dms-desktop/ui/configuration.test.mjs`
- `crates/dms-desktop/ui/library.mjs`
- `crates/dms-desktop/ui/library.test.mjs`
- `crates/dms-desktop/ui/styles.css`
- `docs/changes/active/CHG-0040-document-type-id-migration.md`

The implementation adds the narrow desktop command and registration; Document defaults source/replacement forms; request mapping; `Display label (type-id)` catalogue presentation and ID fallback in Library; focused coverage; responsive migration-row styling; and Phase 2 CHG evidence. It does not modify `dms-core`, CAPs, wireframes, indexes, or AGENTS contracts. After review, the worktree delta was safely applied to the primary checkout over the verified initial overlay. No commit was created.

## Verification evidence

Preparation and orchestrator verification ran:

- `git status --short`, branch, HEAD, and merge-base discovery — exit 0.
- `git diff --check -- crates/dms-desktop/src/lib.rs docs/changes/active/CHG-0040-document-type-id-migration.md` — exit 0.
- Graph/code discovery for the migration command, adapter registration, and UI presentation paths — completed.
- `cargo test -p dms-desktop` in the isolated worktree — exit 0; 92 passed.
- `node --test crates/dms-desktop/ui/configuration.test.mjs` in the isolated worktree — exit 0; 27 passed.
- `node --test crates/dms-desktop/ui/library.test.mjs` in the isolated worktree — exit 0; 40 passed.
- `cargo fmt --all -- --check` and `git diff --check` in the isolated worktree — exit 0.
- Orchestrator inspected the final worktree diff and changed source paths before starting review.
- The exact reviewed delta passed `git apply --check` against the still-matching primary overlay, then was applied. The primary checkout reran `cargo test -p dms-desktop` (exit 0; 92 passed), both named Node suites (exit 0; 27 and 40 passed), `cargo fmt --all -- --check` (exit 0), and `git diff --check` (exit 0).

Not run: live WebView IPC, native Windows/macOS validation, or the Phase 3 workspace gate.

## Review findings and dispositions

The first configured `grok` review attempt stalled and fell through unavailable providers before returning a verdict; its final fallback was rejected with HTTP 402 insufficient OpenRouter credits. The operator authorized one compact fresh retry. `grok` then completed a read-only review of the current worktree diff and returned `No findings`. It checked the core-owned validation boundary, enabled-distinct target selection, request mapping, retained-ID label rendering, missing-catalogue fallback, and CHG evidence. No correction pass was needed.

## Next bounded phase

Phase 2 is complete. This task has no further bounded phase. CHG-0040 Phase 3 (CAP/wireframe publication and workspace closeout) remains pending and requires its own explicit continuation.

## Open risks and decisions

- The primary checkout now contains the reviewed Phase 2 diff plus the original user overlay; no commit was created. Preserve this state when starting Phase 3.
- `astra` and `grok` selectors were accepted session-only. The first reviewer attempt failed externally, but the authorized fresh `grok` retry reached a final no-findings verdict.
- Documentation/CAP/wireframe updates remain Phase 3 work. Do not report CHG-0040 complete after Phase 2.
- DOX pass: `docs/AGENTS.md` and `docs/changes/AGENTS.md` were read. They need no update because these records add no durable repository behavior, ownership boundary, or process contract; the active CHG remains the sole progress authority.
