# CHG-0010 — Selection fold affordance and review schedule section

| Field | Value |
| --- | --- |
| ID | CHG-0010 |
| Status | done |
| External request | Direct operator request: (1) Why is "Document review schedule" not foldable? (2) From useability perspective it's not visable that the topics are foldable |
| Affected CAPs | CAP-0006, CAP-0015 |
| Decision records | none |

**Plan ID:** `CHG-0010-selection-fold-affordance-schedule`
**Created:** 2026-08-16
**Depends on:** CHG-0009
**Produces:** Review schedule as its own foldable topic; visible disclosure chevron + Expand/Collapse cue on each selection-pane topic.

## Goal

1. Promote Document review schedule out of Document control data into an independently foldable section.
2. Make foldability obvious with disclosure chevrons and Expand/Collapse cues.

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | Schedule section + summary affordance + CAP/tests/wireframes | done (`node --test crates/dms-desktop/ui/library.test.mjs`) | Frontend library tests pass |

## Current behaviour

- Foldable topics: Document control data, Document review schedule, Actions, Revision cycle, Releases
- Each summary shows chevron (▸/▾) and Expand/Collapse text via CSS
- Session-only fold state still survives document switches
