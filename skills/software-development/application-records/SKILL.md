---
name: application-records
description: Use when a material App Starter change needs capability, progress, and decision records.
version: 1.0.0
author: App Starter
license: MIT
metadata:
  hermes:
    tags: [product, capability, change-records, documentation]
    related_skills: [phased-plan-execution, github/dev-git-commit-message]
---

# Application Records

## Overview

Maintain the repository's three distinct truths: a ticket requests change, a
CAP file states current behaviour, and a CHG file states active implementation
progress. Code and executable tests remain the final proof of repository
behaviour.

## When to Use

- A request adds, removes, fixes, or changes material user-visible behaviour.
- A permission, privacy, data-handling, or operator-visible outcome changes.
- An agent needs to create, resume, split, complete, or archive a CHG record.
- A durable choice must be classified as capability-local or cross-cutting.

Do not use for a refactor, formatting change, dependency update, or internal
hardening that preserves observable behaviour. Do not use a CAP or CHG to
replace architecture, privacy, or deployment documentation.

## Authoritative Surfaces

Read these before changing a record:

| Need                                            | Read                                                                       |
| ----------------------------------------------- | -------------------------------------------------------------------------- |
| Repository-wide obligation                      | `AGENTS.md`                                                                |
| Current capability contract                     | `docs/product/README.md` and affected `docs/product/capabilities/CAP-*.md` |
| Implementation-progress lifecycle               | `docs/changes/README.md` and affected `docs/changes/active/CHG-*.md`       |
| Architecture, privacy, or irreversible decision | `docs/architecture.md`, `docs/privacy.md`, `docs/design-decisions.md`      |

## Phase Sequencing

The tracked CHG is the execution plan and it is sequenced one phase at a time
by the shared `phased-plan-execution` skill. That skill is the only engine for
phase ordering, gate verification, scope staging, commit checkpoints, and push
verification; this skill decides what each phase must produce.

- Load `phased-plan-execution` with `skill_view` before touching a CHG phase.
  Pass the active CHG path as the plan path prerequisite.
- One CHG phase is `in-progress` at a time. The engine mirrors it in `todo`
  and adds a separate commit-checkpoint todo.
- The CHG phase table is the engine's status surface. Phase rows are the only
  mutable progress authority; prose outside the table, profile-private plans,
  `/tmp` notes, and tracker checkboxes do not schedule work.
- A phase is marked `done (<evidence>)` only after the engine reports the phase
  gate passing. `pnpm records:check` plus the affected test, lint, build, and
  workspace gates are part of that gate, not a separate post-hoc checklist.
- Findings that change scope update the CHG phase table before the source
  changes; required-now work adds an in-phase acceptance item, named
  prerequisite phase, or a split routed through `phased-plan-refactoring`.
- Resuming work in a fresh session reads the CHG phase table, not memory or
  the most recent commit, then loads `phased-plan-execution` and resumes from
  the recorded `Entry checkpoint`.

## Procedure

1. **Classify the request.**
   - If it is vague or blocked by unresolved decisions, use Wayfinder only to
     resolve the decision frontier. The tracker map remains discovery material.
   - If it changes material behaviour, identify affected CAP IDs and create or
     resume one CHG record before implementation. Hand the CHG path to
     `phased-plan-execution` as its plan path.
   - If it preserves observable behaviour, state that no CAP/CHG change is
     needed and proceed under the applicable implementation contract.

2. **Create or resume the CHG authority.**
   - Use `docs/changes/README.md`'s shape, including the phase table the engine
     will mirror in `todo`.
   - Link the external ticket or write `Direct operator request: <verbatim
request>` when no ticket exists. Never invent a ticket ID.
   - Set a single phase to `in-progress`; include an executable verification
     gate that names `pnpm records:check`, the affected tests, and the root
     workspace gate for cross-package work.
   - For material work, the tracked CHG is the execution plan. The engine
     refuses profile-private or `/tmp` plan duplicates; do not create them.

3. **Run a phase under the engine.**
   - Load `phased-plan-execution` and pass the active CHG path as the plan
     path. The engine selects exactly one phase, stages only reviewed paths,
     and runs the gate through `terminal`.
   - Before the commit, update the CHG row (`Phase`, `Status`, evidence) and
     any CAP delta for the vertical slice. A CAP without a behaviour-test
     reference is a coverage gap; add focused coverage or state the bounded
     gap precisely in the CHG before staging.
   - Generate the conventional commit message with
     `github/dev-git-commit-message`, push the branch, and verify
     synchronization through the engine before marking the commit todo done.

4. **Make current behaviour explicit.**
   - Add or amend the affected CAP in the same vertical slice as the phase
     that proves it.
   - State only present-tense, falsifiable outcomes. Link architecture and
     privacy contracts instead of copying their rules.
   - If code/tests contradict the CAP, fix the code or CAP before the next
     phase; do not leave an ambiguous claim.

5. **Place decisions correctly.**
   - Record a capability-local rule in its CAP only when it changes how the
     implemented behaviour must be understood.
   - Record irreversible or cross-cutting runtime, authentication,
     authorization, persistence, privacy, or observability choices in
     `docs/design-decisions.md` and update the live architecture/privacy
     contract in the same phase.
   - Leave short-lived implementation reasoning in the CHG row, code
     comments, or Git history.

6. **Close the change.**
   - Confirm affected CAPs describe the merged behaviour and link executable
     tests as the final phase evidence.
   - Run the CHG integration gate (`pnpm records:check`, full test suite,
     root workspace gate) inside the engine's last phase; do not collapse it
     into the post-commit checklist.
   - Set the CHG status `done`, move it from `active/` to `archive/`, and let
     `phased-plan-execution` return to `phased-plan-overview`. The CHG is an
     implementation receipt, not a feature specification.

## Common Pitfalls

1. **Tracker-shaped truth.** A ticket marked complete is not proof of merged or
   deployed behaviour. Link it from a CHG; do not copy its status into a CAP.
2. **Plan-shaped truth.** A completed phase table explains work history. It does
   not describe current behaviour; the CAP must carry that outcome.
3. **CAP churn.** Do not touch a CAP just because TypeScript changed. Update it
   only when a material outcome changed.
4. **Unproved claims.** A CAP without a behaviour-test reference is a coverage
   gap, not an implemented proof. Add focused coverage or state the bounded
   gap precisely before calling the record complete.
5. **Decision inflation.** Do not create an ADR for local naming or code-layout
   choices. Keep ADRs for durable cross-cutting forks.
6. **Plan duplication.** A profile-private plan or `/tmp` plan that mirrors a
   CHG's progress is forbidden by the engine; route any parallel scope through
   a `phased-plan-refactoring` split that names the CHG as a dependency.
7. **Out-of-order evidence.** Marking a phase `done` before
   `pnpm records:check` and the affected tests pass lets the engine commit a
   broken authority. The gate runs first; the CHG row reflects it second.
8. **Pre-commit CAP drift.** Updating a CAP after the engine has already
   staged the slice hides coverage gaps from the reviewed diff. Update the CAP
   in the same staging window as the source it proves.

## Verification Checklist

- [ ] The active CHG is the only progress authority for the change; no
      profile-private or `/tmp` plan duplicates it.
- [ ] Exactly one CHG phase is `in-progress`, mirrored in `todo`, with a
      separate commit-checkpoint todo.
- [ ] Every material changed behaviour has an affected CAP and behaviour-test
      evidence referenced from both the CAP and the CHG phase row.
- [ ] Every active material request has exactly one CHG progress authority
      linked from `phased-plan-execution`.
- [ ] Every CHG references an external request and valid CAP IDs.
- [ ] `pnpm records:check` and the affected tests pass inside the phase gate
      before the phase is marked `done (<evidence>)`.
- [ ] The root workspace gate passes for cross-package or architecture work.
- [ ] Completed CHGs are in `archive/`; active CHGs are not marked `done`,
      and the engine has returned control to `phased-plan-overview`.
