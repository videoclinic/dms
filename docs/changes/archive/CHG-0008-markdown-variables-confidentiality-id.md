# CHG-0008 — Markdown variables reference and confidentiality type ID

| Field | Value |
| --- | --- |
| ID | CHG-0008 |
| Status | done |
| External request | Direct operator request: realise all recommendation as proposed — (1) documentation of the variables for markdown files; (2) confidentiality id instead of full text |
| Affected CAPs | CAP-0002, CAP-0007, CAP-0008, CAP-0015 |
| Decision records | ADR-0008 |

**Plan ID:** `CHG-0008-markdown-variables-confidentiality-id`
**Created:** 2026-08-16
**Depends on:** `CHG-0007`
**Produces:** Operator reference for Markdown frontmatter and Word template variables; controlled frontmatter `confidentiality` stores the catalogue type ID.

## Goal

1. Publish one durable reference doc listing controlled frontmatter keys, reserved Word tokens, and optional template variables.
2. Write and validate Markdown frontmatter `confidentiality` as the stable confidentiality **type ID**; keep display labels for Office body markers and export chrome `{CONFIDENTIALITY}`.

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | Reference doc + type-ID frontmatter + CAP/ADR/tests | done (`cargo test -p dms-core`; `cargo test -p dms-desktop export::`; `cargo clippy -p dms-core -p dms-desktop --all-targets -- -D warnings`) | Core/desktop tests and clippy pass |

## Current behaviour

- Reference: `docs/markdown-frontmatter-and-template-variables.md`
- Frontmatter `confidentiality` = type ID (e.g. `internal`); Word chrome still uses label
- Markdown content checks compare type ID; DOCX markers still compare label
