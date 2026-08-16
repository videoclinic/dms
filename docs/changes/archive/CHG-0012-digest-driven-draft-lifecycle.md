# CHG-0012 — Digest-driven draft lifecycle (drop Begin revision)

| Field | Value |
| --- | --- |
| ID | CHG-0012 |
| Status | done |
| External request | Direct operator request: Drop the "Begin revision" functionality; Mark documents as draft if the digest does not match with the latest release or if there where no release at all for a registered document |
| Affected CAPs | CAP-0002, CAP-0006, CAP-0011, CAP-0015, CAP-0017 |
| Decision records | ADR-0016 |

**Plan ID:** `CHG-0012-digest-driven-draft-lifecycle`
**Created:** 2026-08-16
**Depends on:** CHG-0011
**Produces:** No Begin revision action; Draft/Released derived from draft digest vs latest release.

## Goal

1. Remove **Begin revision** from core, desktop, and UI.
2. Registered documents are `draft` when never released or when current draft
   bytes differ from the latest non-withdrawn release source digest; they are
   `released` when that digest still matches (open review/approved/obsolete
   unchanged).
3. Library load and document selection reconcile and persist lifecycle.

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | Core sync + remove begin_revision + tests/UI/CAP/ADR/wireframes | done (`cargo test --workspace`; `node --test crates/dms-desktop/ui/library.test.mjs`) | Workspace + library tests pass |
