# CHG-0003 — Retry-safe Entra identity-source application

| Field | Value |
| --- | --- |
| ID | CHG-0003 |
| Status | done |
| External requests | Direct operator request: I can now authenticate DMS and have to "Apply identity source" but get the error message "Microsoft Entra preview is no longer available; sign in and preview again". Follow-up: I got then the following error "the edit-root workflow policy must assign both editor and approver" after "Apply identity source" |
| Affected CAPs | CAP-0019, CAP-0021 |
| Decision records | none — correction to the existing one-shot preview/application boundary |

## Goal

Make **Apply identity source** consume the authenticated preview only after the
workspace binding is saved, and prevent one rendered form from dispatching the
same one-shot preview command more than once while an IPC request is active.
First setup must also collect and atomically persist the required edit-root
editor and approver from that preview.

## Root cause

`MicrosoftGraphClient::apply_identity_source_preview` removes the preview before
`mutate_workspace_configuration` opens, mutates, and saves the workspace. A
failed downstream mutation leaves the frontend's preview visible but makes its
retry invalid. The frontend also leaves the submit control active during IPC, so
a double activation dispatches the same preview ID twice; the first request
consumes it and the second reports that it is no longer available.
Saving the app-global Entra settings replaces the Graph client and therefore
invalidates every backend preview, but its frontend success branch currently
keeps the old preview rendered and applicable.

First-time setup has a circular persistence dependency: the preview supplies
the eligible people needed for root-role selection, but **Apply identity
source** persists only the binding/cache. `Workspace::save` correctly rejects
that intermediate state because a configured identity source requires both root
roles. The separate Workflow form cannot be used until the failed apply has
persisted the eligible people.

## Scope

- Retain a prepared preview when the workspace mutation or save fails.
- Consume it exactly once after successful workspace persistence.
- Suppress repeated submissions from the same live form while its IPC request is
  active and visibly disable that form's submitter.
- Clear the rendered sign-in/preview state when saving global Entra settings
  invalidates the matching backend state.
- On first setup, require an initial edit-root editor and approver selected from
  the preview and save the binding, people cache, and both roles atomically.
- On replacement, retain the existing contract: do not remap root roles to the
  new binding automatically; existing references become unresolved.
- Keep the existing explicit confirmation, preview fields, persistence shape,
  Graph permissions, and token handling unchanged.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Reproduce and record the one-shot failure paths | done (focused Rust test fails to compile against the early-consume API; focused frontend test fails because no submission lock exists) | Focused tests fail against the current early-consume and duplicate-submit behaviour |
| 2 | Make preview application retry-safe and submissions single-flight | done (focused Rust retry test and frontend submission/global-reset tests pass) | Focused Rust and frontend tests pass |
| 3 | CAP/DOX closeout and integration verification | done (`cargo fmt --all -- --check`; `CARGO_INCREMENTAL=0 cargo test --workspace` including 44 desktop Rust tests; `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`; 67 frontend tests; WSL desktop smoke exit 0 with expected EGL/Zink warnings; `git diff --check`) | `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check` |
| 4 | Reproduce first-binding root-role persistence failure | done (Rust test fails because the atomic workspace helper is absent; UI tests fail because the preview has no initial-role controls and the request drops those values) | Focused desktop/core and UI tests fail before the atomic setup slice exists |
| 5 | Persist first binding plus root roles atomically | done (focused Rust tests prove required atomic first setup and no replacement remapping; focused UI tests prove required preview selectors, replacement warning, and IPC arguments) | Focused desktop/core and UI tests pass |
| 6 | CAP/wireframe/DOX closeout and full integration verification | done (CAP-0021 HTML regenerated; 1440×1024 PNG rendered and visually audited without defects; workspace Rust tests including 46 desktop tests, clippy, 69 frontend tests, formatting, WSL desktop smoke, strict Markdown links, record/table structure, and diff checks pass) | Regenerate CAP-0021 HTML/PNG; `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `node --test crates/dms-desktop/ui/*.test.mjs`; desktop smoke; records/link/table checks; `git diff --check` |

## DOX impact

CAP-0019 and CAP-0021 carry the atomic initial-role application outcome and test
evidence; CAP-0021's primary wireframe shows the two required selections. The
desktop contract records atomic first setup and no replacement remapping. The
product/wireframe contracts distinguish desktop surfaces from headless-only
CAPs, and the Entra administrator guide describes the required initial roles.

## Result

- A failed workspace mutation retains its authenticated preview for retry; a
  successful save consumes it exactly once.
- One rendered form dispatches at most one active IPC mutation and visibly
  disables its submitter.
- First identity-source setup requires an edit-root editor and approver from the
  preview and persists the binding, people cache, and both roles together.
- Binding replacement leaves existing role references unresolved rather than
  mapping them to people in the new group.
- Saving global Entra configuration clears frontend setup state invalidated by
  the replaced Graph client.

## Verification

- `cargo fmt --all -- --check`
- `CARGO_INCREMENTAL=0 cargo test --workspace`
- `CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets -- -D warnings`
- `node --test crates/dms-desktop/ui/*.test.mjs` — 69 passed
- `DMS_DESKTOP_SMOKE=1 CARGO_INCREMENTAL=0 cargo run -p dms-desktop` — exited 0
  with expected WSL EGL/Zink warnings
- CAP-0021 HTML generation, 1440×1024 PNG rendering, structural checks, and
  visual inspection
- Strict Markdown links, record/wireframe/table structure, and `git diff --check`
