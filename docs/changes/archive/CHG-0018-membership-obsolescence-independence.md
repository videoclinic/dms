# CHG-0018 — Membership and obsolescence stay independent

**Plan ID:** CHG-0018-membership-obsolescence-independence
**Created:** 2026-08-17
**Depends on:** CHG-0017-candidate-approval-copy
**Entry checkpoint:** CHG-0017 is archived as done; `2a96b1c` is the current `main` tip. Leftover membership-vs-obsolescence docs are already in the working tree.
**Context sources:** `docs/library-membership-and-obsolescence.md`, `docs/product/capabilities/CAP-0006-library-explorer.md#outcomes` (outcome 4), `docs/product/capabilities/CAP-0015-document-control-data.md#outcomes` (outcome 6), `crates/dms-core/src/library.rs` (`unregister_document`), `crates/dms-core/src/lib.rs` (`add_document_inner`)
**Produces:** Proven independence of `source_state` and `lifecycle`: unregister never requires an idle lifecycle and never cancels an open content or periodic review. Operator comparison, ADR-0026, and focused tests ship with the leftover docs.
**Status:** done — runtime already matched; tests and leftover contracts shipped; archived after workspace gate

| Field | Value |
| --- | --- |
| ID | CHG-0018 |
| Status | done |
| External request | Direct operator request: open a CHG and either prove the runtime already matches and implement it with tests |
| Affected CAPs | CAP-0006, CAP-0015 |
| Decision records | ADR-0026 |

## Current state

- Core `unregister_document` only sets `source_state = unregistered`. It does not inspect lifecycle, cancel a candidate, close a periodic review, delete files, or append a workflow event.
- `add_document_inner` re-registers the same path on the same ID and leaves lifecycle unchanged.
- HEAD CAP-0006 still said unregister required an idle review. The working-tree leftover already states the runtime rule.
- Existing tests cover identity retention, not open-review or obsolete add-back.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Prove runtime, add tests, keep leftover contracts | done (`cargo test -p dms-core --test library --test lifecycle`; frontend 28/28 then 95/95) | `cargo test -p dms-core --test library --test lifecycle` and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0 |
| 2 | Workspace gate and change-record closeout | done (`cargo test --workspace`; `clippy -D warnings`; frontend 95/95; link check 0) | Workspace Rust/frontend/link/diff checks exit 0; CHG-0018 archived as done |

## Phase 1 — Prove runtime, add tests, keep leftover contracts

**Goal:** CAP-0006 outcome 4, CAP-0015 obsolescence, ADR-0026, and the operator comparison describe current runtime. Tests prove unregister of obsolete and in-review documents, and of a released document with an open periodic review.

Steps:

1. Keep the leftover comparison page, ADR-0026, CAP-0006 outcome 4, and index links.
2. Add a core library test: obsolete then unregister then add-back keeps the same ID and `obsolete`, leaves the source file, and appends no unregister event.
3. Add a core lifecycle test: unregister during an open content review and during an open periodic review leaves those reviews and the lifecycle in place; add-back restores `registered` on the same ID.
4. Assert the desktop Actions footer still offers Unregister for an `in_review` document.

**Verification gate:** `cargo test -p dms-core --test library --test lifecycle` and `node --test crates/dms-desktop/ui/library.test.mjs` exit 0.

## Phase 2 — Workspace gate and change-record closeout

**Goal:** Repository-wide checks pass and the completed behaviour lives only in current CAPs plus an archived CHG receipt.

**Verification gate:** `cargo fmt --all -- --check && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && node --test crates/dms-desktop/ui/*.test.mjs && python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary . && git diff --check` exits 0.

## Out of scope

- Changing unregister to confirm, emit a workflow event, or cancel reviews.
- Changing mark-obsolete preconditions or periodic-review result **obsolete**.
- Adding a CLI `mark-obsolete` command.
- Changing wireframe interaction (Unregister already exists on Actions).
