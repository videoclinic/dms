# CHG-0031 — Library change-author identity

Show the local OS user whose identity DMS records for ordinary library changes
in the expanded Library sidebar foot. Preserve the workspace ID and root summary.

**Plan ID:** CHG-0031-library-change-author-identity
**Execution slot:** P0520
**Created:** 2026-08-27
**Depends on:** none
**Entry checkpoint:** Direct operator request defines the visible identity outcome.
**Context sources:** `docs/product/capabilities/CAP-0005-desktop-shell.md` (sidebar foot); `crates/dms-core/src/lib.rs` (`default_author`); `crates/dms-desktop/src/lib.rs` (`WorkspaceSummary`); `crates/dms-desktop/ui/app.mjs` (sidebar render); `crates/dms-desktop/AGENTS.md`.
**Produces:** A backend-supplied current local change author, a labelled Library sidebar identity, focused desktop/frontend tests, and current CAP-0005 wireframe evidence.
**Status:** done — all verification gates passed; this record is ready for archive.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0520` is this CHG's execution-order authority and does not change that filename convention.

| Field | Value |
| --- | --- |
| ID | CHG-0031 |
| Status | done |
| External request | Direct operator request: "Show also the user which is logged in for changes while using DMS in the sense of the library:" |
| Affected CAPs | CAP-0005 |
| Decision records | None — the sidebar presents the existing local change-author identity; it does not change workflow authentication or persistence. |

## Risk call-out

Ordinary local library mutations record DMS's local OS author. Microsoft Entra
sign-in is only approval-decision identity and must not be presented as the
actor for all changes. The sidebar must use the same local-author derivation as
DMS evidence, without exposing tokens, object IDs, or credentials.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Add the authoritative change author to the desktop workspace summary and render it in the sidebar foot | done (`cargo test -p dms-desktop`; `node --test crates/dms-desktop/ui/app.test.mjs`) | Focused Rust and app-shell tests pass |
| 2 | Synchronize CAP-0005 and its wireframe | done (`node docs/product/wireframes/generate.mjs`; Chrome CAP-0005 PNG inspected) | CAP and generated HTML/PNG show the labelled local change author |
| 3 | Run workspace gates and close the record | done (`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`, 107 passed) | Format, lint, workspace tests, frontend tests, and record/wireframe checks pass |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 1 — Render the authoritative local change author

1. Extend the desktop workspace summary with the current local change author
   derived through the same core path used in ordinary workflow evidence.
2. Render a labelled **Changes recorded as** value above the workspace ID and
   root-path summary in the expanded sidebar foot only.
3. Add focused adapter and app-shell tests for the summary and escaped footer
   markup. Do not persist this session identity in `.dms` or preferences.

Verification gates: `cargo test -p dms-desktop` and
`node --test crates/dms-desktop/ui/app.test.mjs` exit 0.

## Phase 2 — Current product evidence

1. Amend CAP-0005's sidebar-foot outcome to distinguish the local change author
   from the workspace identity and root summary.
2. Update the CAP-0005 generator screen, regenerate HTML/index/manifest, and
   export its PNG with synthetic identity data.

Verification gate: `node docs/product/wireframes/generate.mjs` exits 0 and the
CAP-0005 HTML/PNG show the same labelled value.

## Phase 3 — Verify and close

1. Run the workspace format, lint, Rust-test, and frontend-test gates.
2. Complete the DOX pass; archive this CHG and refresh the change index only
   after every gate passes.

## Out of scope

- Changing which identity DMS records for local mutations.
- Treating a cached Microsoft Entra token as the author of ordinary changes.
- Persisting an OS username in workspace metadata, preferences, or documents.
- Adding a separate DMS user roster or login flow.
