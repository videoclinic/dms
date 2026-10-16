# CAP-0022 — Headless DMS core CLI

**Primary surface:** none

| Field | Value |
| --- | --- |
| ID | CAP-0022 |
| Status | implemented |
| Executable | `dms` |
| Platforms | Windows and macOS |
| Tests | `crates/dms-core/tests/workspace.rs`; `crates/dms-core/tests/conversion.rs`; `crates/dms-cli/tests/cli.rs`; `crates/dms-export/src/lib.rs` |

## Outcomes

The CLI provides explicit, local operations over the shared `dms-core` library:

1. `dms workspace init` requires `--confirm`, creates `.dms/workspace.json`
   plus the DMS-managed `.dms/AGENTS.md`, records canonical edit and publish
   roots plus a stable workspace ID, and refuses to overwrite an existing
   workspace. The generated instructions describe JSON-oriented agent use and
   the workspace store without exposing source bytes or credentials.
2. `dms workspace status` and `dms workspace verify` reopen and validate the
   stored schema and metadata without a Tauri or WebView runtime.
3. `dms document add` accepts only regular `.md`, `.docx`, `.xlsx`, and `.pptx`
   files within the edit root; it rejects external paths, `.dms` contents,
   Office temporary files, unsupported formats, and duplicate registrations.
4. Registered documents retain a stable ID and edit-root-relative source path.
   Their DMS-managed control data has an initial title derived once from the
   source stem and remains distinct from the source name and path.
5. `dms document list`, `show`, and `update-control` expose and change only the
   stored DMS-managed title, document number, document type, and owner. A title
   cannot be empty; document numbers are case-insensitively unique when set.
6. `dms note add`, `list`, `edit`, and `remove` maintain document-scoped
   plain-text notes with stable note IDs, author, and timestamps. Listing is
   newest-first.
7. Workspace-mutating commands persist validated JSON through the workspace store.
   `--json` emits structured results suitable for automation; normal output is
   concise, human-readable command feedback.
8. `dms convert markdown --workspace-config <workspace.json> --input-root <dir>
   --output-root <new-dir> [--recursive] [--dry-run] [--without-toc]` is standalone
   conversion, never Release. Global `--json` remains available. Selection is
   explicit, deterministic and printed before execution: immediate `.md` files
   by default; recursion excludes `.dms` and `assets` subtrees. Empty selections
   fail. Referenced local images remain constrained to the input root.
9. Conversion reads a schema-18/19/current configuration projection without
   migrations, backups, saves, locks, Entra or notifications. Native-host root
   locators, actual configuration location, template contract and SHA-256 must
   agree; a symlink supplied as the configuration file is rejected before it
   is resolved. External sources supply strict flat title, numeric `MAJOR.MINOR` and
   enabled confidentiality type ID; chrome uses the configured display label.
   Missing or whitespace-only numbers stay empty; nonblank numbers retain their
   supplied value. Optional variables cannot override reserved tokens.
   When the configured template contains literal `{DATE}`, every selected source
   supplies flat `last_modified: YYYY-MM-DD` with a real calendar date. Missing,
   empty or invalid dates fail that document, including in dry-run. Without the
   token, this value is optional and is not date-validated. Assembly fills the
   token from `last_modified`, not an optional `date` key, in template XML
   including body/header/footer; it never substitutes a blank. This source-owned
   conversion alias adds no DMS-controlled key, workspace metadata, custom
   property, filesystem timestamp or release authority. Source/template bytes
   and desktop release semantics remain unchanged.
10. Input and new output roots must be disjoint from workspace roots and output
    must also be disjoint from input. Portable case-insensitive destination
    collisions and existing output roots fail preflight. The output parent must
    exist. Relative parents are mirrored into paired `<stem>.docx` / `<stem>.pdf`.
11. Windows/macOS conversion uses installed Word through a shared non-Tauri
    adapter. The retained DOCX is the controlled-field-filled document saved
    after field, TOC and pagination updates and supplied to PDF export, with no
    later DOCX edit. Default template TOC stays; `--without-toc` removes only the
    template TOC SDT, not Markdown-authored contents. Every generated table uses
    fixed layout spanning printable width. Ordered Markdown lists emit native
    decimal Word numbering with source start values and distinct list restarts;
    unordered lists emit distinct native Word bullet-numbering instances on the
    normal paragraph prototype. The template's `{BULLET LIST}` marker remains a
    validation-only contract input, not a rendered paragraph prototype. Linux
    conversion fails explicitly; dry-run remains available without Word.
12. Sequential processing continues after document-specific failures and exits
    nonzero if any fail. Success requires readable DOCX ZIP/XML and a parsed PDF
    with extractable text, not just a PDF signature. Owned incomplete artifacts
    are removed or reported as diagnostics. `manifest.json` and JSON results
    state `conversion_only`, consumed input/template/asset hashes, effective
    metadata, final saved DOCX/PDF hashes and renderer/Word version. Dry-run
    validates assembly privately without creating output or invoking Word.

Automated evidence uses synthetic fixtures and exporter doubles. Full-corpus
Windows rendering and installed-Word macOS fidelity are not established by these
tests. Markdown links preserve visible labels, not guaranteed PDF navigation.
The DATE regressions in `crates/dms-core/tests/conversion.rs` exercise ordinary
and leap dates, malformed/nonexistent/missing dates, no-token optionality,
pre-export body/header/footer replacement, reserved-token precedence, input
preservation and the unchanged release assembler through public core APIs.
`crates/dms-cli/tests/cli.rs` also checks no-token optionality through the CLI.
This command creates no library membership, approval, occupied version, release
evidence or publish-root index entry; labels and version chrome imply none.

## Non-goals

- Tauri windows, WebView IPC, or desktop activity state
- Approval/release via the conversion command, or a substitute for lifecycle
  authorization, Entra sign-in and notification workflows
- A Tauri plugin, sidecar, or a command that drives a running desktop app
- Conversion source-file mutation, implicit discovery outside its explicit
  input root, or automatic library add

## Links

- DMS Desktop wireframe: none; this capability is headless-only.
- Architecture and ADR-0023: [`../../architecture.md`](../../architecture.md)
- Workspace: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Library membership: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Notes: [`CAP-0003-document-notes.md`](CAP-0003-document-notes.md)
- Document control data: [`CAP-0015-document-control-data.md`](CAP-0015-document-control-data.md)
- Implementation receipt: [`../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/archive/CHG-0001-tauri-local-dms-bootstrap.md)
