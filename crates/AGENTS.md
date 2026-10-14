# Rust workspace

## Purpose

Own the Rust implementation shared by the headless `dms` CLI and the Tauri
desktop adapter.

## Ownership

| Path | Owns |
| --- | --- |
| `dms-core/` | Tauri-independent workspace domain and `.dms` persistence |
| `dms-cli/` | Headless `dms` command parsing and presentation |
| `dms-export/` | Shared Tauri-independent controlled-field fill and installed-Word automation |
| `dms-desktop/` | Tauri startup, WebView shell, OS preferences, and core IPC adapter |

## Local Contracts

- `dms-core` owns validation, metadata mutation, and persistence; callers do
  not reimplement domain rules.
- `dms-core` has no Tauri, WebView, Office, Entra, or notification dependency.
- `dms-cli` accepts explicit workspace/document targets. Only standalone
  conversion discovers Markdown beneath an explicit external input root, with
  recursion opt-in. It creates no library membership or release authority.
  Core may rewrite controlled Markdown frontmatter on registered library
  members when CLI invokes workspace mutations.
- `dms-export` is shared by CLI conversion and desktop release, never by core.
  Core owns conversion paths, strict metadata, manifests and output validation;
  the adapter owns Word field/TOC updates, DOCX save and PDF export.
- Metadata writes go through the core store and remain inside `<edit-root>/.dms`.
- Schema changes require migration fixtures and explicit version handling before
  changing the persisted shape.

## Work Guidance

- Keep UI/OS adapters outside `dms-core`; `dms-desktop` calls its public API just
  as `dms-cli` does.
- Return actionable errors without printing document content or credentials.
- Add focused unit or integration coverage with every material behaviour change.

## Verification

- `cargo fmt --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Child DOX Index

| Child | Owns | Read when editing… |
| --- | --- | --- |
| `dms-core/AGENTS.md` | Domain records and workspace persistence | Shared core Rust code or tests |
| `dms-cli/AGENTS.md` | Command interface and CLI integration tests | CLI Rust code or tests |
| `dms-export/AGENTS.md` | Shared placeholder and native Word adapter contracts | Word automation, retained DOCX handling, or export adapter tests |
| `dms-desktop/AGENTS.md` | Tauri adapter, local WebView shell, and OS user preferences | Desktop Rust, frontend assets, permissions, or tests |

Parent: `../AGENTS.md`.
