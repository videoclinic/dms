# CHG-0019 — Library Refresh re-enumerates the current snapshot

**Plan ID:** CHG-0019-library-refresh-snapshot
**Created:** 2026-08-17
**Depends on:** CHG-0018-membership-obsolescence-independence
**Context sources:** `docs/product/capabilities/CAP-0006-library-explorer.md#outcomes` (outcome 2), `crates/dms-desktop/ui/app.mjs` (`handleLibraryClick`, `loadLibraryFolder`), `crates/dms-desktop/src/lib.rs` (`load_library`)
**Produces:** The Library Refresh control completes a fresh workspace open and one current-folder Library snapshot before it reports handled; the snapshot replaces both the visible folder contents and folder tree without changing library membership.
**Status:** done — Refresh now awaits a fresh workspace open plus current-folder snapshot; archived after workspace gate

| Field | Value |
| --- | --- |
| ID | CHG-0019 |
| Status | done |
| External request | Direct operator request: The reload button does not reload the directory content nor the folder structure |
| Affected CAPs | CAP-0006 |
| Decision records | none |

## Current state

- The Refresh control delegates a detached `loadLibraryFolder` promise and reports its click handled before the refresh operation completes.
- `load_library` creates a fresh core snapshot containing both the folder tree and current-folder entries; the desktop must bind that complete operation to the Refresh action.
- CAP-0006 already requires Refresh to re-enumerate the edit-root structure after external filesystem changes.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Focused refresh transaction and regression coverage | done (`node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs`: 55 passed) | `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs` exits 0 |
| 2 | Workspace gate and record closeout | done (`cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; frontend 96/96; link check 0; `git diff --check`) | Rust/frontend/link/diff checks exit 0; CHG archived as done |

## Phase 1 — Focused refresh transaction and regression coverage

**Goal:** Refresh reopens the active workspace and awaits the fresh current-folder snapshot so externally changed directory entries and the folder tree replace the visible snapshot together.

Steps:

1. Add a narrow frontend helper/test proving Refresh requests `open_workspace` followed by `load_library` for the active folder.
2. Make the Refresh click await that helper and apply its resulting complete snapshot.
3. Keep history, membership, and all file mutation behaviour unchanged.

**Verification gate:** `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/library.test.mjs` exits 0.

## Phase 2 — Workspace gate and record closeout

**Goal:** The CAP describes the proven behavior and the completed change becomes an archived receipt.

**Verification gate:** `cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/*.test.mjs && python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary . && git diff --check` exits 0.

## Out of scope

- Watching directories or auto-adding files.
- Changing library membership, lifecycle, filters, history, or selection semantics.
- Changing the existing Refresh control’s visual hierarchy or wireframe.
