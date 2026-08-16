# CHG-0009 — Selection pane foldable topics

| Field | Value |
| --- | --- |
| ID | CHG-0009 |
| Status | done |
| External request | Direct operator request: Redesign the document control data pane so the main topics are foldable; remove the surrounding frames; keep the folding open/closed while switching documents |
| Affected CAPs | CAP-0006, CAP-0015 |
| Decision records | none |

**Plan ID:** `CHG-0009-selection-pane-foldable-topics`
**Created:** 2026-08-16
**Depends on:** —
**Produces:** Frameless independently foldable Document control data / Actions / Revision cycle / Releases sections with session-only open state preserved across document switches.

## Goal

1. Group the single-document selection pane into four main foldable topics.
2. Remove surrounding card frames from those topics.
3. Keep each section's open/closed state when the operator switches documents in the Library session.

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | Library selection markup/CSS/state + CAP/tests | done (`node --test crates/dms-desktop/ui/library.test.mjs` 23 pass; CAP-0006/0015 wireframes regenerated) | Frontend library tests pass |

## Current behaviour

- Single-document selection pane topics: **Document control data**, **Actions**, **Revision cycle**, **Releases**
- Frameless sections; header keeps title, number, lifecycle badge, Source file identity
- `library.selection_open` is session-only and survives document switches

