# CHG-0032 — Windows edit-root library shortcut

**Plan ID:** CHG-0032-windows-edit-root-library-shortcut
**Execution slot:** P0200
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md` (Local Contracts, Work Guidance); `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/architecture.md` (Runtime shape, Dual-root path model, Trust and control boundary); `docs/privacy.md` (Data classes, Processing principles); `docs/product/capabilities/CAP-0001-local-folder-dms.md` (Outcomes); `docs/product/capabilities/CAP-0006-library-explorer.md` (Outcomes); `docs/product/capabilities/CAP-0020-document-permalinks.md` (Outcomes, Canonical form); `crates/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/lifecycle.rs` (`Workspace::resolve_permalink`); `crates/dms-core/src/library.rs` (`Workspace::collect_library_inventory`, `Workspace::file_entry`); `crates/dms-desktop/src/lib.rs` (`initialize_workspace`, `open_workspace`, `resolve_registered_permalink_from`); `crates/dms-desktop/ui/app.mjs` (`openPermalink`, `activateWorkspace`); `docs/changes/archive/CHG-0023-os-level-dms-uri-registration.md`
**Produces:** On Windows, every DMS-created or explicitly reopened workspace has `<edit-root>/Open in DMS.lnk`. Activating it sends `dms://open?workspace=<stable-workspace-id>` through the registered handler and opens that registered accessible workspace's root Library activity without a document selection. The link contains no edit-root or publish-root path and never appears as an unsupported Library file.
**Status:** done — closed 2026-08-28

Create a Windows Shell Link named `Open in DMS.lnk` in each workspace edit root; it opens that workspace through a workspace-only `dms://` URI, not an executable path or a filesystem path.

| Field | Value |
| --- | --- |
| ID | CHG-0032 |
| Status | done |
| External request | Direct operator request: "Create a Windows lnk file for DMS in the corresponding \"Edit root\" directory. This link should open this library in DMS; using the URI is the prefered way" |
| Affected CAPs | CAP-0001, CAP-0006, CAP-0020 |
| Decision records | ADR-0001, ADR-0006, ADR-0020, ADR-0027 remain applicable; no new ADR is required because this applies the existing local workspace and registered-URI contracts. |

## Current state

- `Workspace::workspace_permalink` creates `dms://open?workspace=<id>`. `Workspace::resolve_permalink` accepts that form and URL-serialized `dms://open/?workspace=<id>`, rejects `target`/`review` on workspace-only links, and preserves document/review/notes resolution.
- The desktop resolver scans only accessible edit roots in the per-user recent-library registry. Failures include the received URI. It does not infer a root from a filesystem path.
- On Windows, confirmed initialization and explicit workspace open write or replace only `<edit-root>/Open in DMS.lnk` after valid workspace metadata opens. The adapter uses `IShellLinkW`/`IPersistFile` with system `rundll32.exe` and the canonical workspace URI; non-Windows targets are no-ops.
- Packaged Windows activation captures `dms://` process arguments into app-owned state, emits `deep-link://new-url` on single-instance handoff, and listens with core `event.listen`. First paint does not wait on deep-link or Entra IPC.
- A workspace URI focuses the singleton `Library · /` activity with no document selection. The registered `dms://` handler ships in the Windows NSIS installer (CHG-0023).
- The exact root `Open in DMS.lnk` helper is excluded from inventory counters and Library rows; nested or differently named `.lnk` files remain unsupported Library files.
- Operator evidence 2026-08-28: installed NSIS package; `Open in DMS.lnk` for `C:\Users\Raphael_Bossek\dms-demo\edit` opens that workspace at `Library · /`.

## Risk call-out

`Open in DMS.lnk` is a visible file in an operator-controlled edit root, outside `.dms`. It must be treated as one fixed DMS helper name, never as a draft or a controlled document. Creating it must not expand URI resolution into opening arbitrary filesystem paths: the URI carries only the stable workspace UUID and resolution stays restricted to registered, accessible recent libraries.

The writer may fail on a read-only/share-unavailable root after `.dms` has already been initialized. Do not roll back the valid workspace or create metadata recovery machinery. Return the exact shortcut-write error to the initialization/open caller; opening the workspace again retries generation.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Add workspace-only URI resolution and hide the DMS helper file | done (`cargo test -p dms-core --test library`: 9 passed; `node --test crates/dms-desktop/ui/app.test.mjs`: 32 passed; `cargo test -p dms-desktop`: 70 passed) | `cargo test -p dms-core --test library` and `node --test crates/dms-desktop/ui/app.test.mjs` exit 0, including workspace-only URI and root-Library activation cases |
| 2 | Generate and refresh the Windows Shell Link | done (Windows `CARGO_INCREMENTAL=0 cargo test -p dms-desktop`: 73 passed, 1 ignored; Linux `cargo test -p dms-desktop`: 70 passed) | Windows `cargo test -p dms-desktop` exits 0 and a new workspace contains `Open in DMS.lnk` whose parsed target invokes the exact canonical workspace URI |
| 3 | Prove packaged activation and update current-state records | done (operator 2026-08-28: installed NSIS; `Open in DMS.lnk` opens `dms-demo` at `Library · /`; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP-0001/0006/0020, architecture, and privacy updated) | On a Windows host with the NSIS installation, double-clicking the generated link opens the expected workspace root; workspace gates and the Windows `Desktop platform smoke` job pass; CAPs and CHG index agree |

## Phase 1 — Add workspace-only URI resolution and hide the DMS helper file

**Goal:** `dms://open?workspace=<workspace-id>` resolves one registered accessible workspace without a document, opens its root Library activity, and leaves document/review/note permalink semantics unchanged.

Verification gate: `cargo test -p dms-core --test library` and `node --test crates/dms-desktop/ui/app.test.mjs` exit 0.

## Phase 2 — Generate and refresh the Windows Shell Link

**Goal:** Successful Windows workspace initialization and explicit workspace open write exactly `<edit-root>/Open in DMS.lnk`, whose only DMS identity payload is the canonical workspace URI.

Verification gate: on Windows, `cargo test -p dms-desktop` exits 0 and a new temporary workspace contains `Open in DMS.lnk` whose parsed target invokes the exact canonical `dms://open?workspace=<workspace-id>` URI.

## Phase 3 — Prove packaged activation and update current-state records

**Goal:** The shipped Windows app can open a real library through its edit-root link, and the CAP/architecture/privacy records describe the delivered behavior rather than this plan.

Verification gate: a Windows-host double-click of `Open in DMS.lnk` opens the expected workspace root through `dms://`; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` exit 0; CAP and CHG indexes reflect the completed implementation.

## Out of scope

- A path-bearing `dms://` URI, auto-discovery of an unregistered workspace, or opening any root based on a shortcut filesystem location.
- A `.url` companion, macOS alias, Linux desktop link, Start-menu shortcut change, or per-document `.lnk` files.
- Storing shortcut state, link paths, or an installation path in `.dms`, app preferences, audit records, or workflow evidence.
- Treating arbitrary `.lnk` files as DMS-managed or hiding any nested shortcut.
