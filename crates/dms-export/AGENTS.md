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
- In both retained and temporary-release modes, success/error cleanup closes only
  the operation's document without saving, then quits the observed owned instance
  when no documents remain. Never quit an instance with documents remaining, and
  never change retained conversion's Word visibility or application alerts. If
  Windows cannot read the observed instance's document count, it fails safe by
  leaving it running rather than guessing that an empty instance is owned.
- On macOS, cleanup quits only when Word was initially stopped and no documents
  remain; retained conversion never changes visibility. Temporary release hides
  Word only when this operation started it. macOS converts Word's POSIX or HFS
  full-name text to an alias and binds the `open` result only after it matches the
  supplied source; an unverified result is never a save/close target. Bind
  immediately after comparison so subsequent failures close that verified document
  without saving.
- Windows does not change application-wide visibility or alerts: its observed
  process-ID difference cannot prove that the COM object is the new process.
  It records Word process IDs before COM activation and uses the observed new
  process only for empty-instance cleanup. A pre-existing operator session is
  outside that snapshot. An unrelated Word process starting in the activation
  window is bounded by the empty-document guard, not attributed with an
  unprovable process-to-COM identity. Word's `Quit` takes `SaveChanges` by
  reference, so owned cleanup passes `wdDoNotSaveChanges` as `[ref]$saveChanges`;
  a literal argument throws inside `finally` and leaks the owned instance.
  Failed COM activation grants no ownership. Preflight checks COM registration
  without activation; licensing and actual automation availability are checked by
  export, not this installation probe.

## Work Guidance

- Use synthetic OOXML and an injected Word double for automated verification.
- Bound native-platform claims honestly: script/adapter tests do not prove
  installed-Word layout or pagination.

## Verification

- `cargo test -p dms-export`
- `cargo test -p dms-export retained_cleanup_` checks static Windows finally and
  macOS success/error cleanup, close-before-quit, ownership and empty-instance
  guards. Script tests also protect retained visibility/alerts and no DOCX
  resave after PDF export; they do not execute native Word.
- `cargo test -p dms-desktop export::tests`
- `cargo test --workspace`

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
