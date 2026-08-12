---
name: application-records-migration
description: Use when migrating legacy docs and plans into capability and change-record records.
version: 1.0.0
author: App Starter
license: MIT
metadata:
  hermes:
    tags: [migration, product, capability, change-records, documentation]
    related_skills: [application-records, phased-plan-execution]
---

# Application Records Migration

## Overview

Migrate a legacy documentation and planning tree into current capability (CAP)
and change-progress (CHG) records without fabricating implementation status or
destroying useful architecture, operational, evaluation, and historical
material. This is audit-first work: classify before moving, and verify every
current-state claim against source and tests.

## When to Use

- A repository has broad numbered documentation plus dated or phased plans.
- A team wants tickets to remain change requests while Git records current
  functionality and active implementation progress.
- A documentation tree has current architecture and operator documents, a
  support-status index, active `Pnnnn-*` plans, dated plans, and retained
  evaluation evidence.

Do not use to rewrite a healthy single capability or to delete an old plan
archive merely because a new record structure exists.

## Preconditions

1. Read the target repository's root and child `AGENTS.md` contracts.
2. Re-run `git status --short --branch` in the target repository.
3. Read the legacy documentation index, documentation ownership contract,
   active-plan contract, and any current architecture/privacy/runbook contracts.
4. Identify the target repository's existing tests, runtime source, and release
   process. A legacy support-status row is never proof by itself.
5. Create a repository-tracked migration CHG before mutating the legacy tree.

If the target has uncommitted unrelated documentation work, inventory it and
stop rather than sweeping it into the migration.

## Classification Matrix

Build a migration map before editing. Each legacy artifact receives exactly one
primary destination; link rather than duplicate content.

| Legacy artifact                                                    | Destination                                                 | Rule                                                                                                                        |
| ------------------------------------------------------------------ | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Current user/operator behaviour grounded in source and tests       | `CAP-*.md`                                                  | Extract concise falsifiable behaviour and test links; keep deep technical detail in its current architecture/runbook owner. |
| Architecture, privacy, security, setup, or operations contract     | Retain current owner                                        | Add CAP links where it proves a capability; do not copy it into every CAP.                                                  |
| Active, dependency-ready phased plan for a material request        | `active/CHG-*.md`                                           | Preserve stable scope, external request, impacts, dependencies, phases, and gates. The CHG becomes progress authority.      |
| Completed/cancelled plan or dated retrospective                    | `archive/CHG-*.md` only when it remains useful as a receipt | Do not bulk-convert every historical plan. Git history remains history.                                                     |
| Irreversible cross-cutting decision                                | ADR/design-decision record plus live contract               | Do not leave the decision only in a plan.                                                                                   |
| Evaluation fixture, result, rendered design, or generated evidence | Retain its evidence owner                                   | It proves a bounded result, not product behaviour or plan progress.                                                         |
| Vague request or unresolved design question                        | Tracker/Wayfinder map                                       | Resolve it before creating a CAP or implementation CHG.                                                                     |

## Structured legacy migration

For a tree with a documentation README support-status table, numbered runtime
contracts, `docs/plans/Pnnnn-*.md`, dated plans, and versioned evaluation
results:

1. Treat the support-status table as an inventory only. Verify each claimed
   capability against the named runtime modules and tests.
2. Keep numbered architecture, workflow, security, and runbook documents in
   place when they remain their best detailed owner. Create concise CAPs that
   link to those documents and executable tests.
3. Convert only genuinely active `Pnnnn-*` material-change plans into active
   CHGs. Preserve their direct dependencies and verification gates. If an old
   plan has no tracker, write `External request: Migrated internal request —
<legacy repository-relative path>`; do not fabricate a ticket.
4. Classify dated plans individually. Most are historical context and should
   remain in Git history or an existing archive, not become live CHGs.
5. Retain evaluation results, fixtures, and generated review artifacts under
   their existing evidence contract. Link them from a CAP or CHG only where they
   directly prove the current claim or phase gate.

## Phase Sequencing During Migration

Migration is itself a material change: it produces CHGs and updates CAPs. Hand
the migration CHG to `phased-plan-execution` as its plan path; that skill
remains the only phase-sequencing engine.

- One conversion phase is `in-progress` at a time and mirrored in `todo`,
  with a separate commit-checkpoint todo. A migration phase may span multiple
  legacy plans only when they share a verification gate; otherwise split via
  `phased-plan-refactoring` and chain the CHGs.
- Phase evidence must name the inventory rows consumed, the CAPs amended, and
  the `pnpm records:check` plus targeted-test results. Evidence of "moved the
  file" is not a phase gate.
- After the last conversion phase, the integration gate (records check,
  targeted tests for every migrated CAP, repository workspace gate) runs
  inside the engine's final phase before the migration CHG is set `done` and
  archived. The migration CHG is an implementation receipt, not a feature
  specification. Control then returns to `phased-plan-overview`; do not jump
  directly to a sibling CHG or unrelated plan.
- Resuming migration in a fresh session reads the migration CHG phase table,
  not the latest commit message or the most recently touched file.

## Procedure

1. **Inventory without mutation.** Produce a table with legacy path, current
   claim, source/test evidence, classification, target record, and action. Mark
   every unverified claim as a coverage gap.
2. **Install the target contract.** Add product/change record directories,
   templates, DOX routing, a structural check, and a migration CHG. Do not claim
   the migration complete yet.
3. **Migrate the minimum current baseline.** Start with two to four material
   capabilities that have existing behaviour tests. Add CAPs and update the
   product index. Leave unverified areas out of the implemented baseline.
4. **Convert active work at verified phase boundaries.** Create CHGs from active
   legacy plans; keep original plan IDs as references where useful, but do not
   leave two mutable progress authorities.
5. **Promote durable decisions.** Move only live cross-cutting decisions into
   the ADR/live-contract pair. Keep routine implementation reasoning out of
   permanent documents.
6. **Reconcile and retire carefully.** Move or delete a legacy document only
   after every durable current claim has a surviving owner, inbound links are
   repaired, and the new CAP/CHG record validates. Prefer retaining a detailed
   architecture or runbook document over flattening it into a CAP.
7. **Verify in layers.** Run the records check, the target Markdown-link check,
   targeted tests for every migrated CAP, and the target workspace gate. Update
   migration CHG phase evidence only after each gate passes.

## Common Pitfalls

1. **Mass conversion.** A filename pattern cannot distinguish a current contract
   from a historical plan. Classify first.
2. **Status laundering.** Never turn a legacy "implemented" cell into a CAP
   without confirming source and executable evidence.
4. **Duplicate authorities.** Do not retain an old active plan and a new CHG with
   independent phase statuses. Select one progress authority — the new CHG,
   sequenced by `phased-plan-execution` — and archive or clearly retire the
   other after links are repaired.
5. **Architecture flattening.** CAPs are concise behaviour indexes, not a place
   to copy whole architecture or runbook chapters.
6. **History deletion.** Git carries historical rationale. Do not delete dated
   plans or evidence merely to make the new structure look tidy.
7. **Migration without a phase engine.** Running conversion steps ad hoc,
   outside `phased-plan-execution`, leaves the migration CHG out of sync with
   Git history and breaks session resume. Always route migration phases
   through the engine and update the CHG row before the commit.
8. **Inventory drift.** A migration map recorded only in the agent's scratch
   notes is lost on session restart. Promote the map into a CAP or the
   migration CHG's accepted evidence so it survives the change.

## Verification Checklist

- [ ] Every legacy artifact appears once in the migration map, persisted in a
      CAP or migration CHG accepted evidence (not profile-private notes).
- [ ] Each implemented CAP is grounded in source and an executable test.
- [ ] Active CHGs have one mutable status authority and valid dependencies,
      sequenced by `phased-plan-execution`.
- [ ] Architecture, operational, decision, and evidence documents retain clear
      owners.
- [ ] Inbound Markdown links resolve after moves.
- [ ] The migration CHG's last phase ran `pnpm records:check`, targeted tests
      for every migrated CAP, and the target workspace gate before archive.
- [ ] No legacy `Pnnnn-*` active plan remains as a parallel progress authority.
