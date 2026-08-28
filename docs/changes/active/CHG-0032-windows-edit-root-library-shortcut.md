# CHG-0032 — Windows edit-root library shortcut

**Plan ID:** CHG-0032-windows-edit-root-library-shortcut
**Execution slot:** P0200
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/architecture.md` (Runtime shape, Dual-root path model, Trust and control boundary); `docs/privacy.md` (Data classes, Processing principles); `docs/product/capabilities/CAP-0001-local-folder-dms.md` (Outcomes); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes); `docs/product/capabilities/CAP-0020-document-permalinks.md` (Outcomes, Canonical form); `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/lifecycle.rs` (`Workspace::resolve_permalink`); `crates/dms-core/src/library.rs` (`Workspace::collect_library_inventory`, `Workspace::file_entry`); `crates/dms-desktop/src/lib.rs` (`initialize_workspace`, `open_workspace`, `resolve_registered_permalink_from`); `crates/dms-desktop/ui/app.mjs` (`openPermalink`, `activateWorkspace`); `docs/changes/archive/CHG-0023-os-level-dms-uri-registration.md`
**Produces:** On Windows, every DMS-created or explicitly reopened workspace has `<edit-root>/Open in DMS.lnk`. Activating it sends `dms://open?workspace=<stable-workspace-id>` through the registered handler and opens that registered accessible workspace's root Library activity without a document selection. The link contains no edit-root or publish-root path and never appears as an unsupported Library file.
**Status:** pending — ready for implementation after CHG-0025 P0100 is no longer the only active priority.

Create a Windows Shell Link named `Open in DMS.lnk` in each workspace edit root; it opens that workspace through a workspace-only `dms://` URI, not an executable path or a filesystem path.

| Field | Value |
| --- | --- |
| ID | CHG-0032 |
| Status | pending |
| External request | Direct operator request: "Create a Windows lnk file for DMS in the corresponding \"Edit root\" directory. This link should open this library in DMS; using the URI is the prefered way" |
| Affected CAPs | CAP-0001, CAP-0006, CAP-0020 |
| Decision records | ADR-0001, ADR-0006, ADR-0020, ADR-0027 remain applicable; no new ADR is required because this applies the existing local workspace and registered-URI contracts. |

## Current state

- `Workspace::resolve_permalink` accepts only `dms://open?...` with both a matching `workspace` UUID and a `document` UUID; a workspace-only URI is rejected as `InvalidPermalink` (`crates/dms-core/src/lifecycle.rs:1529-1579`).
- The desktop resolver scans only accessible edit roots in the per-user recent-library registry, then resolves the URI through `dms-core`; it deliberately does not infer an arbitrary root from a URI (`crates/dms-desktop/src/lib.rs:2429-2467`).
- Confirmed workspace initialization currently creates `.dms` through `Workspace::init` and returns a summary; it creates no Windows filesystem helper (`crates/dms-desktop/src/lib.rs:428-439`). Explicit reopening similarly only reads the workspace summary (`crates/dms-desktop/src/lib.rs:409-411`, `2479-2492`).
- Inbound document permalinks already activate a resolved workspace through the normal session/lock path before opening a document activity (`crates/dms-desktop/ui/app.mjs:726-775`), and the registered `dms://` handler ships in the Windows NSIS installer (CHG-0023).
- A root-level `.lnk` is presently counted and rendered as an unsupported Library file: `collect_library_inventory` increments `unsupported_files`, and `file_entry` returns `LibraryMembership::Unsupported` for every unsupported extension (`crates/dms-core/src/library.rs:447-515`, `621-651`).
- The repository has no Shell Link, `IShellLink`, `WScript`, `rundll32`, or `url.dll` implementation (`git grep -nE 'ShellLink|IShellLink|WScript|rundll32|url\.dll' -- crates` returns no matches). `dms-desktop` already uses a target-specific Windows dependency section (`crates/dms-desktop/Cargo.toml:38-39`).

## Risk call-out

`Open in DMS.lnk` is a visible file in an operator-controlled edit root, outside `.dms`. It must be treated as one fixed DMS helper name, never as a draft or a controlled document. Creating it must not expand URI resolution into opening arbitrary filesystem paths: the URI carries only the stable workspace UUID and resolution stays restricted to registered, accessible recent libraries.

The writer may fail on a read-only/share-unavailable root after `.dms` has already been initialized. Do not roll back the valid workspace or create metadata recovery machinery. Return the exact shortcut-write error to the initialization/open caller; opening the workspace again retries generation. The fixed filename is DMS-owned and is replaced on each successful Windows initialization or explicit open; document that reservation rather than silently using variants or searching for a user-named shortcut.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Add workspace-only URI resolution and hide the DMS helper file | pending | `cargo test -p dms-core --test library` and `node --test crates/dms-desktop/ui/app.test.mjs` exit 0, including workspace-only URI and root-Library activation cases |
| 2 | Generate and refresh the Windows Shell Link | pending | Windows `cargo test -p dms-desktop` exits 0 and a new workspace contains `Open in DMS.lnk` whose parsed target invokes the exact canonical workspace URI |
| 3 | Prove packaged activation and update current-state records | pending | On a Windows host with the NSIS installation, double-clicking the generated link opens the expected workspace root; workspace gates and the Windows `Desktop platform smoke` job pass; CAPs and CHG index agree |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Add workspace-only URI resolution and hide the DMS helper file

**Goal:** `dms://open?workspace=<workspace-id>` resolves one registered accessible workspace without a document, opens its root Library activity, and leaves document/review/note permalink semantics unchanged.

Steps:

1. In `dms-core`, add a canonical workspace-permalink constructor alongside `document_permalink`. Extend the existing permalink result model so a matching workspace URI without `document` represents a workspace target, while document targets retain their required document UUID and review targets retain their required review UUID. Reject a workspace-only URI with `target`, `review`, malformed UUIDs, or a mismatched workspace ID; continue ignoring only unknown extra parameters for valid document links.
2. Add focused core tests proving the exact canonical workspace URI, matching and mismatched UUID handling, rejection of document-only target parameters, and unchanged document/notes/review resolution. Keep the URI free of edit-root, publish-root, document, and filename fields.
3. Extend `DesktopPermalinkResolution` and `resolve_registered_permalink_from` so the resolver returns a workspace target after the same recent-library accessibility scan. Do not add filesystem-path fallbacks or auto-registration. Extend `openPermalink` / activity construction so it activates the resolved workspace through `activateWorkspace`, focuses or creates the singleton root `Library · /` activity, and does not request document selection or notes.
4. Introduce one core-owned predicate for the exact edit-root-relative `Open in DMS.lnk` helper. Apply it to library inventory enumeration and file-entry construction so the root helper neither increments unsupported-file counters nor appears in the Explorer-like Library table. Do not hide nested `.lnk` files or arbitrary shortcuts.
5. Add `dms-core` inventory tests and frontend tests for workspace-only resolution, existing-Library reuse, no selected document, and unchanged document-permalink behavior.

Verification gate: `cargo test -p dms-core --test library` and `node --test crates/dms-desktop/ui/app.test.mjs` exit 0.

## Phase 2 — Generate and refresh the Windows Shell Link

**Goal:** Successful Windows workspace initialization and explicit workspace open write exactly `<edit-root>/Open in DMS.lnk`, whose only DMS identity payload is the canonical workspace URI.

Steps:

1. Add a small Windows-only shortcut adapter under `crates/dms-desktop/src/` and declare only the needed target-specific Win32 COM/Shell dependency in `crates/dms-desktop/Cargo.toml` (with the workspace dependency pinned in root `Cargo.toml`). Keep this implementation out of `dms-core` and do not invoke PowerShell, `WScript`, or a batch file.
2. Build the shortcut with the Windows Shell Link COM interfaces. Its target is the system `rundll32.exe`; its arguments are exactly `url.dll,FileProtocolHandler "dms://open?workspace=<workspace-id>"`; its description identifies the DMS workspace opener. This deliberately activates the installed OS URI handler rather than coupling the link to an installed executable path. Use the workspace's canonical URI constructor rather than formatting a second URI by hand.
3. Call the adapter only after `Workspace::init` has persisted valid metadata and after the explicit `open_workspace` path has successfully opened a valid workspace. On non-Windows targets it is a no-op; do not create `.url` files, fake `.lnk` text files, or platform-specific artefacts under other OSes.
4. Reserve the exact root-level filename `Open in DMS.lnk`: replace that DMS helper on each successful Windows init/open, return a contextual shortcut-write error when it cannot be written, and leave `.dms` valid for a later retry. Do not touch any other file, including nested links and root links with different names.
5. Add Windows-gated adapter tests that initialize and reopen a temporary workspace, parse the saved Shell Link through the same COM API, and assert its filename, `rundll32.exe` target, exact URI argument, and absence of edit-root/publish-root strings. Keep cross-platform tests green without requiring a Windows runner.

Verification gate: on Windows, `cargo test -p dms-desktop` exits 0 and a new temporary workspace contains `Open in DMS.lnk` whose parsed target invokes the exact canonical `dms://open?workspace=<workspace-id>` URI.

## Phase 3 — Prove packaged activation and update current-state records

**Goal:** The shipped Windows app can open a real library through its edit-root link, and the CAP/architecture/privacy records describe the delivered behavior rather than this plan.

Steps:

1. On a real Windows host, install the NSIS package, initialize or explicitly reopen a test workspace, and confirm `<edit-root>/Open in DMS.lnk` appears. Close DMS, double-click the link, and record non-secret evidence that the installed handler starts/focuses DMS, opens the matching registered accessible workspace, reaches `Library · /`, and selects no document. Repeat after restarting DMS to cover single-instance activation. A copied/unregistered workspace is expected to require normal DMS open first; the URI must fail closed rather than use a path.
2. Update CAP-0001 with the Windows edit-root helper outcome, CAP-0006 with the exact helper-file exclusion, and CAP-0020 with the workspace-only URI form and its registered-accessible resolution semantics. Update `docs/architecture.md` and `docs/privacy.md` to classify `Open in DMS.lnk` as a local edit-root helper containing only a workspace ID URI. No wireframe is required: this introduces no primary app surface.
3. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs`. Confirm the Windows entry of `.github/workflows/desktop-platform-smoke.yml` passes. Then mark every phase `done (<evidence>)`, move this record to `docs/changes/archive/`, and update `docs/changes/README.md` from Active to Archive in the same change.

Verification gate: a Windows-host double-click of `Open in DMS.lnk` opens the expected workspace root through `dms://`; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; the Windows `Desktop platform smoke` job passes; CAP and CHG indexes reflect the completed implementation.

## Out of scope

- A path-bearing `dms://` URI, auto-discovery of an unregistered workspace, or opening any root based on a shortcut filesystem location.
- A `.url` companion, macOS alias, Linux desktop link, Start-menu shortcut change, or per-document `.lnk` files.
- Storing shortcut state, link paths, or an installation path in `.dms`, app preferences, audit records, or workflow evidence.
- Treating arbitrary `.lnk` files as DMS-managed or hiding any nested shortcut.
