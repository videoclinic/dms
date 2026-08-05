---
name: application-records
description: Use when a material App Starter change needs capability, progress, and decision records.
version: 1.0.0
author: App Starter
license: MIT
metadata:
  hermes:
    tags: [product, capability, change-records, documentation]
    related_skills: []
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

## Procedure

1. **Classify the request.**
   - If it is vague or blocked by unresolved decisions, use Wayfinder only to
     resolve the decision frontier. The tracker map remains discovery material.
   - If it changes material behaviour, identify affected CAP IDs and create or
     resume one CHG record before implementation.
   - If it preserves observable behaviour, state that no CAP/CHG change is
     needed and proceed under the applicable implementation contract.

2. **Create or resume the CHG authority.**
   - Use `docs/changes/README.md`'s shape.
   - Link the external ticket or write `Direct operator request: <verbatim
request>` when no ticket exists. Never invent a ticket ID.
   - Set a single phase to `in-progress`; include executable verification gates.
   - For material work, the tracked CHG is the execution plan. Do not create a
     competing profile-private plan that carries the same progress.

3. **Make current behaviour explicit.**
   - Add or amend the affected CAP in the same vertical slice as implementation
     and behaviour tests.
   - State only present-tense, falsifiable outcomes. Link architecture and
     privacy contracts instead of copying their rules.
   - If code/tests contradict the CAP, fix the code or CAP before completion;
     do not leave an ambiguous claim.

4. **Place decisions correctly.**
   - Record a capability-local rule in its CAP only when it changes how the
     implemented behaviour must be understood.
   - Record irreversible or cross-cutting runtime, authentication,
     authorization, persistence, privacy, or observability choices in
     `docs/design-decisions.md` and update the live architecture/privacy
     contract in the same change.
   - Leave short-lived implementation reasoning in the CHG or Git history.

5. **Prove and record progress.**
   - Run the current phase gate. Mark it `done (<commit or command evidence>)`
     only after it passes.
   - Run `pnpm records:check` plus affected tests. Run the root workspace gate
     for cross-package or architecture work.
   - Update the CHG before the implementation commit so a new session can
     resume from repository truth.

6. **Close the change.**
   - Confirm affected CAPs describe the merged behaviour and link executable
     tests.
   - Run the full CHG integration gate.
   - Set the CHG `done`, move it from `active/` to `archive/`, and retain it as
     an implementation receipt. Do not use it as a feature specification.

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

## Verification Checklist

- [ ] Every material changed behaviour has an affected CAP and behaviour-test evidence.
- [ ] Every active material request has exactly one CHG progress authority.
- [ ] Every CHG references an external request and valid CAP IDs.
- [ ] `pnpm records:check` exits 0.
- [ ] The active phase gate and required workspace checks pass.
- [ ] Completed CHGs are in `archive/`; active CHGs are not marked `done`.
