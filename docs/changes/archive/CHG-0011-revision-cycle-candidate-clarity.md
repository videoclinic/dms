# CHG-0011 — Revision cycle candidate clarity

| Field | Value |
| --- | --- |
| ID | CHG-0011 |
| Status | done |
| External request | Direct operator request: In document control data pane: (1) The "Revision cycle" topic the "View workflow evidence" button is used for what? Looks like unfolding "Canonical workflow evidence * valid" has the same effect (2) Reposition "Submit release candidate" on top of "Revision cycle" (3) If the "Target version" is not "Manual target" disabled the "Manual major" and "Manual minor" fields and show the next version would be applied: start counting with "0.1"; show the effective target version (4) The wording "Submit" is not clear what happens then submit where ? submit why ? Make clear what happens then |
| Affected CAPs | CAP-0002, CAP-0006, CAP-0015 |
| Decision records | none |

**Plan ID:** `CHG-0011-revision-cycle-candidate-clarity`
**Created:** 2026-08-16
**Depends on:** CHG-0010
**Produces:** Clearer Revision cycle candidate action copy, target-version preview with manual fields gated, candidate form first in the section, redundant evidence button removed.

## Goal

1. Drop the redundant **View workflow evidence** button; keep only the foldable
   **Canonical workflow evidence** disclosure (same effect, one control).
2. Place **Create release candidate** at the top of **Revision cycle**.
3. When Target version is not Manual, disable Manual major/minor and show the
   effective target that Next minor / Next major would apply (first release still
   `V1.0` per CAP-0002; later next-minor steps advance the minor component by 1,
   e.g. `V1.0` → `V1.1`).
4. Replace vague **Submit** labels with wording that states the candidate is
   created in this workspace and what follows (direct export vs approver review).

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | UI + CAP/wireframe + library tests | done (`node --test crates/dms-desktop/ui/library.test.mjs`; wireframes regenerated) | Frontend library tests pass; CAP-0015/CAP-0002 HTML+PNG current |
