# Markdown frontmatter and Word template variables

Operator reference for Markdown drafts registered in a DMS library and for the
workspace Word template used on Markdown → PDF release.

## Controlled frontmatter (DMS-owned)

DMS is the source of truth for these keys on **registered** Markdown library
members. It prefills them when a confidentiality policy exists and overwrites
them when document control, effective confidentiality, or the candidate target
version changes. Non-controlled keys are preserved. Frontmatter never updates
`.dms` control data.

| Key | Required | Value | Notes |
| --- | --- | --- | --- |
| `title` | yes (when synced) | DMS document title | Defaults from file stem on first add |
| `document_number` | no | DMS document number | Omitted from frontmatter when unset |
| `version` | yes (when synced) | `MAJOR.MINOR` without a `V` prefix | Idle default is next minor (`1.0` first); candidate submit writes the chosen target |
| `confidentiality` | yes (when synced) | Confidentiality **type ID** | Stable catalogue id (e.g. `internal`), not the display label |

Example after library import with root policy `internal` → label “Internal”:

```yaml
---
title: Employee handbook
document_number: HB-001
version: 1.0
confidentiality: internal
author: Ada Lovelace
---
# Body
```

Review and release content checks for Markdown compare those controlled keys to
the candidate snapshot (title/number when present; version; confidentiality
**type ID**).

## Optional frontmatter → Word template variables

Any other flat ASCII-identifier frontmatter key is treated as a template
variable during temporary DOCX assembly:

- Frontmatter key `author` → placeholder `{AUTHOR}`
- Frontmatter key `department` → placeholder `{DEPARTMENT}`
- Keys are case-insensitive for the token (uppercased); values are XML-escaped
- Nested YAML, lists, and non-identifier keys are not supported
- These variables never become document-control data

Reserved tokens that optional frontmatter must **not** fill (export chrome owns
them):

- `{TITLE}`
- `{DOCUMENT_NUMBER}`
- `{VERSION}`
- `{CONFIDENTIALITY}`

## Word template / export chrome

| Mechanism | Token / property | Source at release |
| --- | --- | --- |
| Literal placeholder | `{TITLE}` | `.dms` release snapshot title |
| Literal placeholder | `{DOCUMENT_NUMBER}` | snapshot document number (empty if none) |
| Literal placeholder | `{VERSION}` | candidate version label without `V` |
| Literal placeholder | `{CONFIDENTIALITY}` | effective confidentiality **display label** |
| Custom property | `DMS_TITLE` | same as `{TITLE}` |
| Custom property | `DMS_DOCUMENT_NUMBER` | same as `{DOCUMENT_NUMBER}` |
| Custom property | `DMS_VERSION` | same as `{VERSION}` |
| Custom property | `DMS_CONFIDENTIALITY` | same as `{CONFIDENTIALITY}` |

Word `DOCPROPERTY` fields bound to the `DMS_*` properties refresh on export.
Body assembly replaces CommonMark content using heading/paragraph/list/table
prototypes in the template; frontmatter is stripped from the body.

## Office drafts (contrast)

`.docx` drafts still use visible body/header/footer markers:

- `Version: <major>.<minor>`
- `Vertraulichkeitsstufe: <display label>`

Those markers use the **label**, not the type ID. PDF filenames always use the
confidentiality type ID regardless of draft format.

## Related contracts

- CAP-0002 document lifecycle (content checks)
- CAP-0007 draft PDF export (template fill)
- CAP-0008 confidentiality classification (type IDs and labels)
- CAP-0015 document control data (DMS authority)
- ADR-0008 format-specific local PDF export
