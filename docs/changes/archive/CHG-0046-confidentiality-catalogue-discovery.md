# CHG-0046 — Confidentiality catalogue discovery

A newly initialized library has no confidentiality types or root policy. Make the
catalogue entry point prominent in Configuration → Document defaults so an
operator can create the first type, set it as the workspace default, and later
configure retained-ID future-release migrations without first locating the
Document types catalogue.

**Plan ID:** CHG-0046-confidentiality-catalogue-discovery
**Created:** 2026-08-31
**Depends on:** CHG-0030
**Entry checkpoint:** Direct operator request identifies that confidentiality
cannot be discovered or configured for a new library.
**Context sources:** `docs/product/capabilities/CAP-0008-confidentiality-classification.md`; `docs/changes/archive/CHG-0030-confidentiality-type-id-migration.md`; `crates/dms-core/src/lib.rs` (`Workspace::init`); `crates/dms-desktop/ui/configuration.mjs`; `crates/dms-desktop/ui/configuration.test.mjs`.
**Produces:** A direct Document defaults entry point to the confidentiality
catalogue, empty-catalogue guidance, focused frontend coverage, and current CAP
wording.
**Status:** done — all verification gates passed; this record is ready for archive.

| Field | Value |
| --- | --- |
| ID | CHG-0046 |
| Status | done |
| External request | Direct operator request: "With CHG-0030 we introduced also the possibility to migrate confidentiality (like for document types) but in the UI there is no ability to define Confidentiality at allo (neither to migrate them -- for a new library)" |
| Affected CAPs | CAP-0008 |
| Decision records | none |

## Current state

- `Workspace::init` creates empty confidentiality catalogues and policies.
- The desktop command and secondary catalogue already create types, select the
  first workspace default, and submit migration IDs to core.
- The sole entry point is attached to the Document types card, which makes a
  required new-library setup task look like a document-type feature and leaves
  the empty confidentiality state without guidance.

## Risk call-out

Keep ID validation, retained-reference protection, enabled-target validation,
cycle rejection, candidate invalidation, and Markdown projection in `dms-core`.
This change only makes the existing desktop workflow discoverable; it must not
introduce frontend copies of those rules or change release evidence.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Expose confidentiality setup from Document defaults | done (`node --test crates/dms-desktop/ui/configuration.test.mjs`, 23 passed) | `node --test crates/dms-desktop/ui/configuration.test.mjs` exits 0 with empty-catalogue and migration-entry coverage |
| 2 | Synchronize CAP and close | done (`node docs/product/wireframes/generate.mjs`; `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`, 129 passed; `git diff --check`) | `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check` exit 0 |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 1 — Expose confidentiality setup from Document defaults

1. Keep **Manage confidentiality types…** next to the workspace confidentiality
   default, rather than nesting it under Document types.
2. When no types exist, say that the operator must create the first type and
   make it the workspace default; the control must still open the secondary
   catalogue.
3. Retain the secondary catalogue's add, label, enable, workspace-default, and
   retained-ID migration controls unchanged.
4. Add frontend tests for the direct entry point, empty state, and existing
   migration request.

Verification gate: `node --test crates/dms-desktop/ui/configuration.test.mjs`
exits 0.

## Phase 2 — Synchronize CAP and close

1. Amend CAP-0008 to make the direct Document defaults entry point and empty
   catalogue outcome explicit.
2. Run the workspace gates and complete the DOX pass. The existing CAP-0008
   wireframe already places this entry point in the workspace-default summary;
   regenerate it to verify the generated record remains current.
3. Mark Phase 1 done only after its focused gate passes. Archive this CHG only
   after every Phase 2 gate passes.

## Out of scope

- Changing the core confidentiality catalogue, schema, or migration semantics.
- Rewriting folder policies, document overrides, candidates, releases, or
  Markdown frontmatter.
- Moving confidentiality management into the workspace-initialization dialog.
