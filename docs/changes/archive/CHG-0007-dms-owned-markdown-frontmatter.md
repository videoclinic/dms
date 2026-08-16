# CHG-0007 — DMS-owned Markdown frontmatter

| Field | Value |
| --- | --- |
| ID | CHG-0007 |
| Status | done |
| External request | Direct operator request: DMS ready the frontmatter of markdown files in order to prefill default vaules (as long they match the library settings) and overwrite the frontmatter in markdown files if this values are changes/updated in DMS itself for the file; DMS is source of truth for imported markdown files in the DMS library |
| Affected CAPs | CAP-0002, CAP-0007, CAP-0015, CAP-0008 |
| Decision records | ADR-0008 |

**Plan ID:** `CHG-0007-dms-owned-markdown-frontmatter`
**Created:** 2026-08-16
**Depends on:** `CHG-0006`
**Produces:** Registered Markdown library members get controlled frontmatter keys written and kept in sync from DMS document control and library settings; `.dms` remains the authority.

## Goal

For Markdown drafts registered in the library:

1. Prefill controlled frontmatter from DMS defaults (title, effective confidentiality label, default next version, optional document number).
2. When DMS control data, confidentiality, or lifecycle target version changes, overwrite those controlled frontmatter keys in the source file.
3. Preserve non-controlled frontmatter keys (template variables).
4. Never import frontmatter into `.dms` control data.

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | Core sync + CAP/ADR/tests | done (`cargo test -p dms-core`; `cargo clippy -p dms-core --all-targets -- -D warnings`) | Core frontmatter/workspace/lifecycle tests pass; clippy clean |

## Current behaviour

- Controlled keys: `title`, `document_number` (omit when unset), `version`, `confidentiality`.
- Prefill on add/reassociate; overwrite on control update, confidentiality override/policy/type label change, candidate submit (target version), begin revision, cancel review.
- Batch add defers file writes until metadata commit succeeds.
- Sync no-ops until a confidentiality policy exists.
- Non-controlled frontmatter keys are preserved for Word-template variables.
- CAP-0002/0007/0015, ADR-0008, architecture, and DOX describe one-way DMS authority.
