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

## Standalone conversion operation

`dms convert markdown` creates conversion-only DOCX/PDF pairs, not registered,
approved or released documents. It reads export configuration without opening,
migrating, saving or locking the workspace; no Entra session or notification is
started. Desktop Release keeps its separate `.dms`-owned approval/version flow.

### Prerequisites and invocation

- Build the headless executable with `cargo build -p dms-cli --locked`.
  Native Windows uses `target\debug\dms.exe`; macOS/Linux use `target/debug/dms`.
  Actual PDF conversion requires installed, licensed desktop Word. Windows has
  [recorded native full-selection evidence](changes/archive/CHG-0053-markdown-batch-conversion-retained-docx.md#handoff);
  the macOS adapter has synthetic/static coverage, not installed-Word fidelity
  verification. Linux/WSL supports dry-run only; there is no automatic Word bridge.
- Use the native host's existing `<edit-root>/.dms/workspace.json`, with schema
  18, 19 or current, an enabled confidentiality catalogue and a configured Word
  template whose digest and contract validate. Configured edit/publish roots
  must exist and match that host; Windows locators are not reinterpreted on Linux.
- Input must be an external directory disjoint from both workspace roots.
  Output must not exist, its parent must exist, and it must be disjoint from
  input/edit/publish trees. These rules apply to dry-run too.
- Each UTF-8 Markdown source needs flat `title`, numeric `version: MAJOR.MINOR`
  (no `V`), and `confidentiality` containing an enabled type ID, not its label.
  Omit `document_number` when unset. Supply `last_modified` when required by
  [the template's `{DATE}` token](#standalone-conversion-date).

Example from native Windows PowerShell, using illustrative operator-chosen paths:

```powershell
.\target\debug\dms.exe --json convert markdown --workspace-config 'C:\DMS\edit\.dms\workspace.json' --input-root 'C:\Conversion\sources' --output-root 'C:\Conversion\run-1' --dry-run
.\target\debug\dms.exe --json convert markdown --workspace-config 'C:\DMS\edit\.dms\workspace.json' --input-root 'C:\Conversion\sources' --output-root 'C:\Conversion\run-1'
```

Run the second command only after reviewing dry-run results and selection.
Default discovery selects every immediate `.md` child; `--recursive` explicitly
includes descendants except `.dms` and `assets` subtrees. There is no per-file
selection flag: keep non-target Markdown outside the selected scope. PNG/JPEG
references resolve relative to their source and must stay within the input root,
including shared assets. Remote, escaping, missing or unsupported images fail.
Keep sources/assets/template stable through conversion; hashes record consumed
bytes, not a guarantee against concurrent writers.

The template TOC stays by default. `--without-toc` removes only its TOC
structured-document tag, not a source-authored contents section. Static template
text is not translated. Avoid manual heading-number prefixes when template styles
already number headings. Link labels survive, but Markdown-relative links are
not rewritten into PDF navigation. The command has no TODO-filter or
language-selected-template option; it uses the one configured template.

### Output and failures

- The exact selected paths go to stderr before processing. With `--json`, stdout
  contains the final manifest/result object, including mixed per-file failures.
  Normal stdout reports conversion-only totals; per-file errors go to stderr.
- Dry-run validates metadata, assets and temporary DOCX assembly without Word
  preflight or output-root creation. Successful entries are `validated`, with no
  DOCX/PDF hashes or Word receipt. It does not prove licensing, Word automation,
  pagination or visual fidelity.
- Actual conversion mirrors relative parents and writes `<stem>.docx`,
  `<stem>.pdf` and root `manifest.json`. DOCX is the final controlled-field-filled
  document saved by Word after field/TOC/pagination updates, then exported to PDF
  without a later DOCX edit. `succeeded` entries include both final hashes;
  `conversion_only: true` belongs to the manifest/result, not a document watermark.
- Configuration/template/root/collision/host-preflight errors stop before document
  work and may produce no manifest. Document-specific errors continue sequentially
  and any failed entry makes the command exit nonzero. Failed pairs have no
  success hashes; owned incomplete files are removed or listed in
  `retained_diagnostics`. An output/manifest-write failure or interruption may
  leave files without a complete receipt: file existence alone is not success.
- Inspect stderr and each result's `status`, `error` and `retained_diagnostics`.
  Correct the input/configuration or host problem and use a fresh output root;
  there is no overwrite/resume mode. Do not terminate unrelated Word sessions
  after an interruption. Protect retained DOCX/PDF, temporary/diagnostic copies
  and rendered verification pages as confidential content, and captured output
  as metadata; see [privacy](privacy.md). Rendered inspection is still needed for
  a new source/template combination; structural success is not a layout guarantee.

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

Those markers use the **label**, not the type ID. Released PDF filenames use the
confidentiality type ID regardless of draft format; standalone conversion uses
the unversioned source stem as described above.

## Related contracts

- CAP-0002 document lifecycle (content checks)
- CAP-0007 draft PDF export (template fill)
- CAP-0008 confidentiality classification (type IDs and labels)
- CAP-0015 document control data (DMS authority)
- ADR-0008 format-specific local PDF export
