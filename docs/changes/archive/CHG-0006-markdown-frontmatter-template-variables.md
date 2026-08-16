# CHG-0006 — Markdown frontmatter template variables

| Field | Value |
| --- | --- |
| ID | CHG-0006 |
| Status | done |
| External request | Direct operator request: While converting a Markdown file with the Word template remove the frontmatter from The Markdown file for the target output; use the frontmatter as "variable" definitions |
| Affected CAPs | CAP-0002, CAP-0007 |
| Decision records | ADR-0008 |

**Plan ID:** `CHG-0006-markdown-frontmatter-template-variables`
**Created:** 2026-08-16
**Depends on:** `CHG-0004`
**Entry checkpoint:** Markdown release already stripped frontmatter from the CommonMark body and filled only chrome-controlled `{TITLE}` / `{DOCUMENT_NUMBER}` / `{VERSION}` / `{CONFIDENTIALITY}` tokens.
**Produces:** Flat Markdown frontmatter scalars become optional Word-template variables; frontmatter never appears in the assembled body; controlled release chrome remains authoritative for the four DMS fields.

## Goal

When assembling a temporary DOCX from Markdown and the workspace Word template:

1. Drop YAML frontmatter from the converted body.
2. Treat additional flat frontmatter key/value scalars as template variable definitions.
3. Substitute matching `{KEY}` placeholders in temporary package XML from those values.
4. Leave `TITLE`, `DOCUMENT_NUMBER`, `VERSION`, and `CONFIDENTIALITY` owned by export chrome / `.dms` release snapshot.

## Phases

| Phase | Goal | Status | Gate |
| --- | --- | --- | --- |
| 1 | Frontmatter variable parse + DOCX assembly fill + CAP/ADR/tests | done (`cargo test -p dms-core --test markdown_template`; `cargo test -p dms-core --lib frontmatter`; `cargo test -p dms-desktop export::`; `cargo clippy -p dms-core -p dms-desktop --all-targets -- -D warnings`) | Focused core frontmatter/template tests and desktop Markdown export tests pass; clippy clean |

## Current behaviour

- `parse_markdown_frontmatter` keeps every flat scalar in `variables`.
- `MarkdownFrontmatter::template_variables` exposes non-reserved ASCII-identifier keys as uppercased `{KEY}` tokens.
- `assemble_markdown_docx` converts only the body and fills those tokens across temporary package XML parts; reserved controlled tokens stay for chrome fill.
- CAP-0002, CAP-0007, ADR-0008, architecture, and `dms-core` DOX describe the split between validation, variables, and chrome authority.
