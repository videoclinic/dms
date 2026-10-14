# Shared local export adapter

## Purpose

Own Tauri-independent controlled OOXML fill and installed-Word automation shared
by desktop release and standalone CLI conversion.

## Ownership

- `src/lib.rs`: placeholder fill, native Windows/macOS Word adapter, test seam,
  retained-DOCX save/export ordering, host preflight and adapter tests.
- `Cargo.toml`: adapter dependencies; no Tauri, identity or notification runtime.

## Local Contracts

- Core owns conversion metadata, path selection, manifests and final pair
  validation; desktop release retains its core-owned snapshot/atomic PDF flow.
- Fill only generated/temporary copies, never the source or imported template.
  Replacements are non-recursive, including tokens split across XML text nodes.
- Both modes update body/header/footer fields and template TOC before PDF
  export. Only retained conversion explicitly repaginates, updates TOC page
  numbers and confirms a DOCX save. Never edit DOCX after PDF export.
- Shared native automation accepts explicit retained versus temporary-release
  mode; do not duplicate COM/AppleScript implementations in caller crates.
- Temporary staging is privately owned and cleaned; no predictable filename may
  overwrite operator files. Native errors are sanitized, not raw Office stderr.
- Close only the document opened by the operation. Do not close unrelated Word
  documents, alter global security settings, or add a Linux/WSL fallback.
- macOS converts Word's POSIX or HFS full-name text to an alias and binds the
  `open` result only after it matches the supplied source; an unverified result
  is never a save/close target. Bind immediately after comparison so subsequent
  failures close that verified document without saving.
- macOS temporary release hides Word and quits on success/error only if Word
  was initially stopped and no documents remain. Retained conversion never
  sets application visibility or quits Word. Installation preflight resolves
  the application identifier without launching it.
- Windows records whether Word was already running before COM activation;
  only temporary release with a newly started Word may hide it or suppress
  alerts, and may quit only after the opened document closes and none remain.
  Retained conversion never hides or quits Word or changes application alerts.
  Preflight checks COM registration without activation; licensing and actual
  automation availability are checked by export, not this installation probe.

## Work Guidance

- Use synthetic OOXML and an injected Word double for automated verification.
- Bound native-platform claims honestly: script/adapter tests do not prove
  installed-Word layout or pagination.

## Verification

- `cargo test -p dms-export`
- `cargo test -p dms-desktop export::tests`
- `cargo test --workspace`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
