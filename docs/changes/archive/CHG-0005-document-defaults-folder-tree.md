# CHG-0005 — Document defaults folder tree

| Field | Value |
| --- | --- |
| ID | CHG-0005 |
| Status | done |
| External request | Direct operator request: In Configuration→"Document defaults" the "Choose default or exception" is not a folder tree view like in "Workflow" for "Choose default or exception". Apply the same logic and useability: Unfold only that folders where the configuration is not inherited |
| Affected CAPs | CAP-0008 |
| Decision records | none |

**Plan ID:** `CHG-0005-document-defaults-folder-tree`
**Created:** 2026-08-16
**Depends on:** `CHG-0001#phase-9k.5`
**Entry checkpoint:** Clean `main` with Document defaults still using the flat folder list.
**Produces:** Document defaults **Choose default or exception** uses the same semantic expandable folder tree as Workflow, auto-expands paths to direct confidentiality policies, and badges those direct assignments.

## Goal

Match Document defaults folder picking to Workflow: a nested tree with independent branch toggles, initial expansion only along paths that hold a direct (non-inherited) confidentiality policy, and visible markers for those direct policies.

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | CAP-0008 tree contract + Configuration UI/tests + DOX | done (`node --test crates/dms-desktop/ui/configuration.test.mjs` — 18 pass) | Frontend configuration tests and DOX pass |

## Current state

- Document defaults reuses the Workflow semantic folder tree (`role=tree` / `treeitem`).
- First snapshot expands only paths that reveal a direct confidentiality or workflow policy; fully inherited branches stay collapsed.
- Direct confidentiality types are badged on their folders; inherited types are not.
- CAP-0008 outcome 3 describes that tree contract; the stale folder-exceptions-table outcome is removed.

## Steps

1. Amend CAP-0008: replace the flat/exceptions-table wording with the semantic tree contract shared with CAP-0019 (expandable levels, direct-policy badges, auto-reveal exception paths only).
2. Reuse `folderTreeMarkup` on Document defaults; auto-expand confidentiality policy paths on first snapshot; badge direct types.
3. Prove with frontend tests; update `crates/dms-desktop/AGENTS.md`.
