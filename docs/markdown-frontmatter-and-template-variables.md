# Markdown frontmatter and Word template variables

Operator reference for registered Markdown drafts, the workspace Word template
used on Markdown → PDF release, and standalone external Markdown conversion.

## Controlled frontmatter (DMS-owned)

DMS is the source of truth for these keys on **registered** Markdown library
members. It prefills them when a confidentiality policy exists and overwrites
them when document control, effective confidentiality, or the candidate target
version changes. Non-controlled keys are preserved. Frontmatter never updates
`.dms` control data.

When a confidentiality type ID has a retained migration, the policy or document
override and an idle draft stay on the source ID. Preparing the next release
candidate resolves the enabled replacement and overwrites `confidentiality`
with that replacement ID before the candidate's content check. Existing
candidates and released records are not rewritten.

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

## Initial library import

On the **first** registration of a Markdown source, DMS imports these optional
flat scalar keys once into `.dms`. Re-addition, reassociation, later source
edits, and Office source files never repeat the import.

| Key | Imported destination | Validation |
| --- | --- | --- |
| `title` | DMS title | Non-empty text |
| `document_number` | DMS document number | Case-insensitively unique in the workspace |
| `version` | First DMS release target | `MAJOR.MINOR` without `V`; creates neither a release nor a reservation |
| `document_type` | DMS document type | Enabled configured type ID |
| `review_interval_months` | Document review-schedule override | Positive integer |
| `editor` | Document Editor override | Exact eligible display name/email, or one unique case-insensitive substring match |
| `approver` | Document Approver override | Exact eligible display name/email, or one unique case-insensitive substring match |

An editor or approver match with zero or multiple eligible people rejects the
entire add; DMS never guesses an identity. The imported version is the exact
target of the first successful DMS release and still requires first-release
approval. Once registration succeeds, DMS owns the imported values; only the
four controlled keys above are projected back into source frontmatter.

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

## Standalone conversion date

Only `dms convert markdown` maps the source-owned flat frontmatter scalar
`last_modified` to literal `{DATE}` during Markdown assembly:

```yaml
last_modified: 2024-02-29
```

- If the configured template contains `{DATE}`, every selected source must
  supply a real calendar date with exactly four year digits, two month digits
  and two day digits (`YYYY-MM-DD`). Leap-day validation uses the calendar, not
  just the string shape. Missing, empty and invalid values are document-specific
  errors in normal conversion and dry-run; independent documents continue.
- If the template has no `{DATE}`, missing or invalid `last_modified` does not
  fail conversion. The existing flat-frontmatter syntax rules still apply;
  this is not support for nested YAML or malformed quoted scalars.
- Literal tokens in template XML, including preserved body, header and footer
  text, are filled before the export adapter runs. This uses the existing
  literal assembly replacement, not Word's DATE field or a token split across
  XML runs. An optional `date` key cannot bypass the required `last_modified`
  value or override its replacement when the template contains `{DATE}`.
- The value does not become a DMS-controlled key, workspace metadata, release
  authority, custom property or filesystem timestamp. No `DMS_DATE` property is
  created. Source and template files are never rewritten, and the four reserved
  export tokens retain their existing authority.
- Desktop release does not use this alias or require `last_modified`. Its
  existing optional-variable rules remain unchanged; `last_modified` can still
  supply `{LAST_MODIFIED}` like any other non-controlled scalar.

Synthetic public-path tests in
[`crates/dms-core/tests/conversion.rs`](../crates/dms-core/tests/conversion.rs)
verify the date matrix, replacement before export, unchanged bytes and the
non-release boundary. These tests do not establish installed-Word rendering or
real-corpus acceptance. See [CAP-0022](product/capabilities/CAP-0022-headless-core-cli.md).

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
