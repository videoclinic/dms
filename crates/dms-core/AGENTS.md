# dms-core

## Purpose

Provide the Tauri-independent local DMS domain model and `.dms` workspace
persistence.

## Ownership

| Path | Owns |
| --- | --- |
| `src/lib.rs` | Versioned workspace store, migration boundary, DMS-managed agent instructions, document-control data, and notes API |
| `src/assistance.rs` | Optional Claude Desktop policy, released-PDF/source comparison payloads, and assistance evidence |
| `src/audit.rs` | Filtered deterministic CSV/PDF audit reports, report-file integrity, and workspace report evidence |
| `src/catalogues.rs` | Document-type catalogue primitives plus shared stable-ID and label validation |
| `src/conversion.rs` | Library conversion-policy API and source override resolution, read-only export-configuration projection, external batch discovery/metadata/path validation, sequential conversion port, retained-pair validation and manifest digests |
| `src/integrity.rs` | Advisory workspace locks, backup-manifest validation, and confirmed root-safe restore |
| `src/library.rs` | Folder/file discovery, membership, search, registration state, reassociation, and permalinks |
| `src/lifecycle.rs` | Version candidates, Entra/notification/export ports, content conformance, review decisions, release commits, and hash-chained evidence |
| `src/maintenance.rs` | Release checksum verification, workspace review defaults, periodic-review scheduling and transitions, and full-workspace ZIP backup with SHA-256 manifest |
| `src/policies.rs` | Folder-policy tree, retained confidentiality type-ID migrations, Entra display binding, and workflow-role resolution |
| `src/source_history.rs` | Bounded first-import OOXML source-history capture and schema validation |
| `src/frontmatter.rs` | Strict flat Markdown parsing, controlled-key rewrite from DMS, optional template-variable map, and expected/detected comparison |
| `src/template.rs` | Reusable workspace Word-template identity, validation, CommonMark-to-OOXML assembly with local PNG/JPEG asset embedding, and frontmatter variable fill |
| `tests/` | Domain, migration-fixture, and persistence behaviour tests |

## Local Contracts

- Stable workspace, document, and note IDs are persisted identifiers.
- Source paths are canonicalized, must resolve under the configured edit root,
  and are stored relative to it with platform-independent `/` separators in
  metadata and machine-readable output.
- Source filename/path are locator facts. Mutable document profile, immutable
  candidate/release snapshots, and mutable review schedules are separate
  metadata domains. Each actual profile edit appends canonical before/after
  workflow evidence before invalidating stale candidates.
- First registration of an Office draft may retain a source-history record bound
  to the imported bytes: at most three normalized, attributable observations or
  a sanitized no-data/malformed/unattributed outcome. It contains no source
  content, raw OOXML, Office properties, client IDs, inferred Entra identity,
  workflow evidence, candidate, or release data, and re-registration never
  rescans or rewrites it.
- Folder discovery exposes only edit-root-relative regular files and directories,
  excludes `.dms`, Office temporary sidecars, and the exact root `Open in DMS.lnk`
  helper, and never auto-registers or auto-reassociates a source. Nested or
  differently named shortcuts remain ordinary unsupported files. Every discovered
  folder carries recursive counters for draft registered documents, addable supported
  files, unsupported files, and lost-source ((re-)moved) registered documents; each
  visible file contributes to exactly one counter. Lost-source phantom rows appear in
  the folder of the stored locator.
- Unregister and reassociate preserve stable document identity and retained
  document metadata; batch mutations validate atomically before changing state.
  Unregister sets `source_state` only: it does not require an idle lifecycle,
  cancel an open content or periodic review, delete files, or append a
  workflow event. Adding the same in-root path again restores `registered` on
  the same ID and leaves lifecycle unchanged.
- `Workspace::save` is the persistence boundary for ordinary mutations; release
  export owns its save/rollback transaction so a committed PDF and metadata
  release record cannot be reported independently.
- Schema migrations retain the source metadata backup and verify the migrated
  shape before atomically replacing `workspace.json`.
- `.dms/AGENTS.md` is a versioned DMS artifact. Workspace metadata fingerprints
  its generated original; an instruction-template revision preserves modified
  operator guidance as `AGENTS-<STAMP>.md` for manual merge before writing the
  new original.
- Folder policies target only existing edit-root-relative directories, exclude
  `.dms`, and retain non-removable root defaults once configured.
- Microsoft Entra workspace metadata contains tenant/group identifiers and a
  read-only person display cache, never credentials or tokens.
- Configuration queries expose only enabled cached people for role pickers and
  return folder workflow assignments with their binding-qualified references;
  workflow inheritance and notification validation remain core-owned rules.
- Lifecycle candidates snapshot the requesting person, resolved owner, effective
  editor and approver, document profile, required effective date,
  confidentiality, source digest, target, and changelog. Authority comparisons
  use binding-qualified Entra object IDs, never mutable display names or email.
  Only committed releases occupy versions. A staged real owner/editor handover
  applies atomically with a successful release export.
- Approval-required operations refresh direct-user group membership and verify
  the interactive Entra actor through injected ports; core metadata stores no
  Graph, SMTP, or authentication credentials.
- Review and release content checks require scalar Markdown frontmatter
  `version` and `confidentiality`, compare optional `title` and
  `document_number` when present, and scan visible DOCX body/header/footer text;
  unsupported formats fail closed. Overrides require a reason and remain bound
  to the checked digest, target, confidentiality, and phase in the workflow
  chain. For registered Markdown members, DMS prefills and overwrites those
  controlled frontmatter keys from document control, effective confidentiality
  type ID, and candidate target version. First registration of a Markdown member
  alone may import flat `title`, `document_number`, `version`, `document_type`,
  `review_interval_months`, `editor`, and `approver` values into `.dms`; it never
  repeats on re-addition, reassociation, or later source edits. Sync is skipped
  until a confidentiality policy exists; policy/type/label
  changes re-sync all registered Markdown members. A retained type-ID migration
  leaves current policy/override references and idle drafts unchanged, invalidates
  an affected active candidate, then projects the enabled non-cyclic replacement
  ID while preparing the next candidate. Export chrome
  `{CONFIDENTIALITY}` still uses the display label.
- One optional stable workspace Word-template asset may reference an ordinary
  in-root non-symlink `.docx`. It is configuration rather than a controlled
  document, is excluded from Library file rows and counters, and preserves
  unrelated OOXML package parts during deterministic CommonMark body assembly.
  Its strict contract includes the four `DMS_*` custom properties with one
  release-value placeholder each. Markdown release passes only a configured,
  present, unchanged, valid template path to the export adapter; rejection does
  not call the adapter or commit release evidence. Assembly strips YAML
  frontmatter from the body and fills optional non-reserved `{KEY}` tokens from
  additional flat frontmatter scalars; reserved controlled tokens remain for
  export chrome.
- Markdown headings use native template numbering. Unordered and ordered Markdown
  use generated native Word bullet and decimal numbering respectively, applied to
  the normal paragraph prototype rather than the bullet paragraph prototype.
  Each source list has a distinct numbering instance; ordered lists preserve
  source start values and never inherit bullet formatting. Existing template
  numbering is preserved with non-colliding generated IDs. The `{BULLET LIST}`
  marker remains required by template contract v1, but does not style list bodies.
  Each paragraph within a list item remains a separate Word paragraph; only the
  first carries a marker. Continuations, including those after nested lists,
  retain list-body indentation with numbering explicitly suppressed.
  Synthetic OOXML tests do not establish Word's exported PDF structure ownership.
- Markdown table assembly retains the two-cell template contract. Every
  column count uses equal fixed grid/cell widths from
  the body section's page width minus left/right margins and gutter; absent or
  invalid page dimensions fail explicitly. The first column uses the first-cell
  prototype; all later columns use the second. Consecutive Markdown tables receive
  one empty template paragraph between their OOXML tables so Word save/export does
  not coalesce them. Cell/paragraph/run styling and inline emphasis/code/link text
  survive without modifying source or template.
- Standalone conversion is separate from Workspace open/save and lifecycle
  authority. It supports only schema 18, 19, 21 and current read-only projections,
  validates native configuration location and template identity, and never
  migrates, locks, authenticates, notifies or imports workflow data. Explicit
  config-file symlinks are rejected before canonicalization. Whitespace-only
  external document numbers are absent; nonblank values are preserved. Explicit
  external input and fresh disjoint output roots, portable collision checks,
  per-file strict metadata and sequential mixed results are core-owned rules.
  A host adapter fills/saves the retained DOCX and exports that same document;
  core reads both artifacts, validates ZIP/XML and parsed PDF text, hashes final
  bytes and writes the conversion-only manifest. Consumed Markdown/template and
  embedded asset bytes determine input hashes. Dry-run assembles temporarily
  without the host adapter or output-root writes.
- `Workspace::conversion_policy` / `set_conversion_policy` own the typed library
  setting; mutation changes memory only until `Workspace::save`. Missing policy
  defaults to disabled; a present object requires exactly `todo_projection` with
  `enabled` or `disabled`. `ConversionPolicy::resolve_todo_projection` and the
  read-only `ConversionPlan` resolver use strict flat conversion frontmatter:
  the exact source key overrides the library setting, omission inherits, and
  invalid values fail without echoing source data. Quoted scalar values follow
  the existing flat parser; case, aliases and booleans are not normalized.
  The setting is not document control, candidate/release evidence or a controlled
  frontmatter key. Standalone conversion resolves it against the exact hashed
  source bytes before assembly; no lifecycle or Release assembler resolves or
  applies it. Per-file manifest `todo_projection` is `enabled`/`disabled`, or null
  when resolution failed (never an implicit fallback).
- Enabled standalone TODO projection operates on balanced CommonMark events before
  numbering and asset loading. A case-insensitive initial visible `TODO:` removes
  a paragraph, heading, complete list item (including descendants), or code block;
  a table cell becomes one NBSP without changing the grid or row. Formatting does
  not obscure the marker; fence language metadata and image alt text are not body
  content. Images and rendered task checkboxes count as substantive content.
  Nested lists and tables have independent entries; an item's own continuation
  text (including quoted continuations) belongs to that item. Unmarked empty
  parents survive removal of marked children. Definition titles, tight definition
  bodies and body metadata blocks are visible entries; loose definition bodies
  have direct Paragraph children and retain independently projected paragraphs.
  Paragraphs nested in quotes/lists do not make a definition body loose; tight
  body diagnostics use the definition entry start. After projection, a retained
  definition title with no remaining nonempty body becomes an explicit paragraph,
  so deleting a definition and its nested blocks cannot join the title to following
  content. Retained definitions keep the ordinary renderer semantics; disabled
  conversion and Release bypass this normalization. Raw HTML blocks are literal
  paragraph text, not interpreted HTML; their markers receive the same checks.
  `TODO:` must start a word or follow a closed inline run: within uninterrupted
  text a preceding Unicode alphanumeric character or underscore makes it an
  identifier suffix, not a marker (`NOTODO:`/`mytodo:`). Preserve markers split
  across formatting events (`**TO**DO:`); a complete marker immediately after
  closed formatting or inline code triggers mixed-entry rejection.
  A later marker, including in a rendered link URL, after substantive content fails the
  source before artifact reservation with entry kind and original entry-start
  line/column, never source text. Surviving native list items compact from the
  original list start; distinct lists retain distinct numbering instances.
- Standalone conversion alone aliases flat source `last_modified` to literal
  `{DATE}` during assembly. A template containing that token requires a real
  `YYYY-MM-DD` date per source, including dry-run; otherwise missing/invalid dates
  are optional. Keep this out of controlled frontmatter, workspace metadata,
  custom properties, timestamps and release assembly. Existing reserved tokens
  retain their authority; errors must not expose source values.
- The default assembler preserves the template TOC. The explicit options API
  may remove only its TOC structured-document tag and returns consumed-asset
  digests; it never removes source-authored contents. No new template markers.
- Markdown image references resolve only through an explicit source/asset
  context of input root plus source path. Parent-relative references are allowed
  when the canonical target remains inside the input root. A reference without
  that context, a remote, absolute or fragment-bearing reference, a missing,
  structurally corrupt or unsupported asset, a target outside the canonical input
  root including symlink escapes, and raw HTML `<img>` markup all fail
  explicitly instead of emitting alt text alone. PNG and JPEG bytes are embedded
  verbatim as package media with non-colliding media part, relationship and
  drawing IDs. Drawing IDs start above the maximum `wp:docPr` / `pic:cNvPr`
  identity across all template XML parts, including preserved story parts.
  Repeated references reuse media/relationships, not drawing IDs, and preserve
  aspect ratio within the body printable width. Image-alt inline events are
  collected separately from body runs and formatting; line breaks become spaces,
  and image-only paragraphs contain only a drawing run. Content-type extensions
  are matched case-insensitively; each required default is added once and conflicting declarations fail before
  output creation. Image checks validate PNG chunk bounds/CRCs/data presence and
  JPEG segment/frame/scan structure/data presence, not compressed pixel decoding.
  Apart from body replacement and optional-variable fill, assembly adds media and
  list-numbering definitions with their package relationships and content types;
  source, template and asset files are never modified.
  Other CommonMark constructs (task lists, footnotes, math, raw inline HTML,
  definition lists, block quotes, nested lists) keep their visible text.
- Workflow evidence is append-only, SHA-256 predecessor-linked, newest-first at
  the public history boundary, and validated whenever workspace metadata opens
  or saves.
- Cancel review and mark obsolete use the core lifecycle preconditions and
  append canonical `review_cancelled` or `document_obsoleted` evidence.
  Cancellation and obsolescence require reasons. There is no Begin revision
  action: registered documents are `draft` when never released or when the
  current draft digest differs from the latest non-withdrawn release source
  digest, and `released` when that digest still matches.
- Audit reports deterministically serialize the selected control, workflow,
  periodic-review, release, and verification records without embedding source
  drafts or release PDFs. Report paths remain inside the edit root, never
  overwrite existing files, and are recorded in a separate canonical
  workspace-level `report_generated` evidence chain.
- `Workspace::verify_release` and `Workspace::verify_all_releases` re-read the
  recorded PDF and compare its SHA-256 digest; they never modify, repair, or
  delete release bytes.
- `Workspace::start_periodic_review` binds the current release ID, version,
  PDF digest, confidentiality snapshot, and approver; a mismatched or missing
  PDF blocks the request.
- `Workspace::complete_periodic_review` refreshes current Entra eligibility,
  requires the snapshotted eligible approver, records the result, and applies
  `ConfirmedCurrent` / `ChangesRequired` / `Obsolete` transitions
  deterministically.
- Periodic-review request, result, comment-required cancellation, and each
  reminder attempt are separate canonical workflow events. Cancellation leaves
  the release schedule unchanged; reminders neither duplicate the request nor
  change lifecycle state.
- Review permalinks resolve both content-approval requests and periodic-review
  requests so notification links never point at an unresolvable review ID.
- Canonical `dms://open` permalinks resolve a stable workspace UUID alone to its
  root Library target, or pair it with a stable document UUID. `review` and `notes`
  refine only document targets; workspace-only links reject both. Unknown extra
  parameters do not affect valid document-link resolution.
- `Workspace::backup_workspace` refuses to overwrite an existing archive,
  refuses symlinks and non-regular files, and writes a Zip archive containing
  metadata, every registered draft, every recorded release PDF, and a
  SHA-256 manifest entry per file.
- Advisory `.dms/lock` records contain local owner/process evidence. Ordinary
  acquisition refuses a current lock, stale-only takeover uses the
  workspace-configured threshold, and overriding any existing lock requires a
  separate explicit core operation.
- Restore validates the complete archive manifest, entry types, sizes, digests,
  workspace identity, destination lock, and confirmed replacement policy before
  writing only beneath existing operator-selected edit and publish roots. It
  holds an owner-recorded destination lock throughout file writes and rejects
  cross-platform path aliases before restoration.
- Claude assistance is disabled by default, permits only configured
  confidentiality type IDs, and verifies the current release before extracting
  comparison text. An oversized preview exposes every exact excerpt and its
  measured size; only an explicit operator-selected subset that fits the limit
  yields a digest-bound payload. No payload is silently truncated, and only
  explicit accepted-use evidence is recorded in lifecycle records.

## Work Guidance

- Keep public operations deterministic and explicit.
- Add migration support before increasing `SCHEMA_VERSION`.
- Schema v6 adds the default-disabled workspace Claude-assistance policy and
  optional candidate/release/workflow assistance evidence, with a
  `v5.json.bak` retained during migration.
- Schema v7 adds the workspace-level report evidence chain, with a
  `v6.json.bak` retained during migration.
- Schema v8 adds the positive per-workspace advisory-lock staleness threshold,
  with a `v7.json.bak` retained during migration.
- Schema v9 adds the optional document-control effective date, with a
  `v8.json.bak` retained during migration.
- Schema v12 separates mutable document profile, release-bound effective date
  and profile/owner snapshots, and mutable review schedule. Its v11 migration
  retains `v11.json.bak`, preserves legacy owner text without identity inference,
  maps a stored date only to the current non-withdrawn release and retained open
  candidates, and leaves older release snapshot omissions unrecorded.
- Schema v13 separates the SMTP authentication login user from its RFC 5322
  `From` mailbox. Its v12 migration copies the legacy sender to both fields and
  retains `v12.json.bak`; SMTP passwords remain outside core metadata.
- Schema v14 adds the optional Markdown Word-template record without assigning
  an implicit asset. Its v13 migration retains `v13.json.bak`.
- Schema v15 adds optional confidentiality replacement IDs. Its v14 migration
  writes `replacement_type_id: null` for every catalogue entry and retains
  `v14.json.bak`.
- Schema v16 removes notification transport and SMTP relay fields from portable
  workspace metadata. Desktop injects the current OS user's validated settings
  only for a lifecycle operation; the v15 migration retains `v15.json.bak`.
- Schema v17 adds optional first-import Office source-history records. Its v16
  migration writes `source_history: null` for every existing document and
  retains `v16.json.bak`.
- Schema v18 adds the persisted Entra tenant binding. Its v17 migration retains
  `v17.json.bak` and leaves a group-only binding explicitly unverified until an
  operator reapplies the source; it never infers tenant identity from cache,
  OS-user configuration, or historic evidence.
- Schema v19 adds optional document-type replacement IDs. Its v18 migration
  writes `replacement_type_id: null` for every catalogue entry and retains
  `v18.json.bak`. Mappings retain the current document-control ID, invalidate
  affected active candidates, and resolve only into later candidate/release
  snapshots.
- Schema v20 records the DMS-generated agent-instruction revision and SHA-256
  fingerprint. Its v19 migration preserves any pre-existing `.dms/AGENTS.md`
  as manual merge input before installing the generated original.
- Schema v21 adds optional per-document Markdown initial versions. Its v20
  migration defaults the field to `None`, retaining existing first-target and
  release history behaviour.
- Schema v22 adds `conversion_policy: {"todo_projection": "disabled"}` when
  absent during migration, retaining the original versioned backup (including
  `workspace.v21.json.bak`). Present invalid policies fail validation rather
  than being replaced with defaults. Standalone configuration reads never migrate.

## Verification

- `cargo test -p dms-core`
- `cargo test -p dms-core --test conversion_policy --test conversion --test lifecycle --test workspace --test policies --locked`
  checks policy persistence/migration, strict overrides, read-only configuration,
  effective-policy receipts, unchanged control metadata and the Release boundary.
- `cargo test -p dms-core --test conversion` checks read-only configuration,
  metadata, paths, mixed results, retained pairs, consumed-byte digests and
  failure cleanup through synthetic exporter doubles, not installed Word.
  TODO cases also verify structural DOCX projection, table NBSP/grid preservation,
  compact list references, precise mixed-entry rejection, original input hashes,
  package-part equality against an explicitly retained-source assembly, and
  synthetic PDF text transport from the retained DOCX. This is not native
  Word PDF layout or tagged-table/list structure evidence.
- `cargo test -p dms-core --test smoke_contract` pins actual parser text,
  optional document numbers, unique embedded-asset receipts and serialized
  template identity for the independent smoke verifier. Synthetic-only direct
  public API tests do not establish installed-Word fidelity.
- `cargo test -p dms-core --test markdown_template` checks multi-table cell text,
  inline styling, two/three/four/five-column layouts, native bullet/decimal list
  references, source-list starts and separation, numbering-package preservation,
  paragraph-property insertion/replacement, multi-paragraph/nested-list
  continuations, deterministic assembly,
  package XML, unchanged source/template/asset bytes, image embedding,
  in-root parent-relative resolution, escaping targets, structural corruption,
  content-type/media/relationship/drawing collisions across body/header/footer/
  footnote/comment parts, image-alt event isolation, and no-content-loss
  construct coverage. Successful image cases use real synthetic PNG/JPEG
  fixtures; header-only stubs are rejection cases only.
- `cargo test -p dms-core --test markdown_template --test lifecycle` checks the
  assembler together with the release path that supplies its asset context.

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.