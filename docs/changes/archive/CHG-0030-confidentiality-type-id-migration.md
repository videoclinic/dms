# CHG-0030 — Confidentiality type-ID migration

Provide a retained confidentiality type-ID migration: policies and document
overrides keep the old ID, while each future candidate and released version uses
the configured replacement type. Registered Markdown frontmatter changes to the
replacement ID only as the next release candidate is prepared.

**Plan ID:** CHG-0030-confidentiality-type-id-migration
**Execution slot:** P0510
**Created:** 2026-08-26
**Depends on:** none
**Entry checkpoint:** Direct operator request defines the migration outcome.
**Context sources:** `docs/product/capabilities/CAP-0008-confidentiality-classification.md`; `docs/product/capabilities/CAP-0002-document-lifecycle.md`; `crates/dms-core/src/policies.rs`; `crates/dms-core/src/lifecycle.rs`; `crates/dms-core/src/frontmatter.rs`; `crates/dms-desktop/ui/configuration.mjs`; `crates/dms-desktop/AGENTS.md`.
**Produces:** A non-cyclic replacement mapping persisted in `.dms`, release-time Markdown frontmatter projection, a Configuration → Confidentiality types control, focused executable coverage, and updated CAP/wireframe evidence.
**Status:** done — all verification gates passed; this record is ready for archive.
**Filename convention:** The repository's active-record contract requires `CHG-*.md`; `P0510` is this CHG's execution-order authority and does not change that filename convention.

| Field | Value |
| --- | --- |
| ID | CHG-0030 |
| Status | done |
| External request | Direct operator request: "Add a function to \"migrate\" old \"type-id\" to a different new \"type-id\" in the confidentiality types. The old type-id can not be deleted and for documents using the old type-id for the next release to new type-id is used instead. The frontmatter in markdown files is migrated to the new type-id during release process." |
| Affected CAPs | CAP-0002, CAP-0008, CAP-0013, CAP-0015 |
| Decision records | ADR-0010 — extend the existing catalogue decision with retained replacement mappings |

## Risk call-out

A migration must not rewrite folder policies, document overrides, candidates, or
released snapshots: they are audit evidence or current source classification.
Only a candidate created after the mapping can snapshot the replacement ID. An
open candidate whose release classification changed must be invalidated, not
released under an approval made for the old type. The mapping target must be an
enabled configured type, and cycles must be rejected so release resolution
cannot loop.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Implement persisted replacement mapping and release projection | done (`cargo test -p dms-core`) | `cargo test -p dms-core` exits 0 with migration, candidate invalidation, and Markdown-to-release coverage |
| 2 | Expose the secondary-configuration migration control | done (`cargo test -p dms-desktop`; `node --test crates/dms-desktop/ui/configuration.test.mjs`) | `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/configuration.test.mjs` exit 0 |
| 3 | Synchronize CAPs, ADR, Markdown reference, and wireframe | done (`node docs/product/wireframes/generate.mjs`; headless Chrome CAP-0008 PNG inspected) | `node docs/product/wireframes/generate.mjs` exits 0; CAP-0008 HTML/PNG show retained-ID future-release migration |
| 4 | Verify and close | done (`cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`, 106 passed) | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `node --test crates/dms-desktop/ui/*.test.mjs` all exit 0 |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate
passes, and `pending` otherwise.

## Phase 1 — Persisted mapping and release projection

1. Add a defaulted replacement type ID to the persisted confidentiality type,
   with a v14 → v15 migration and backup fixture coverage.
2. Add a core migration operation that rejects missing, disabled, identical, or
   cyclic targets; preserve the old type and all references to it.
3. Resolve the replacement only for future release candidates, invalidate any
   affected active candidate, and rewrite registered Markdown controlled
   frontmatter to the candidate's replacement ID before its content check.
4. Prove retained old references, invalid mapping rejection, mapping persistence,
   candidate invalidation, replacement release snapshot/PDF name, and Markdown
   frontmatter projection.

Verification gate: `cargo test -p dms-core` exits 0.

## Phase 2 — Secondary configuration control

1. Add a narrow desktop command that invokes the core migration operation and
   returns the refreshed configuration snapshot.
2. In **Manage confidentiality types**, show each retained source ID and let the
   operator select a different enabled type for future releases.
3. Add command and markup/request tests without duplicating core validation in
   the frontend.

Verification gates: `cargo test -p dms-desktop` and
`node --test crates/dms-desktop/ui/configuration.test.mjs` exit 0.

## Phase 3 — Current product evidence

1. Make CAP-0008's type-ID replacement, retained-reference, candidate, and
   Markdown outcomes current; adjust the related lifecycle, maintenance, and
   document-control contracts only where their current claims change.
2. Amend ADR-0010 and the Markdown frontmatter reference with the retained-ID
   versus future-release distinction.
3. Update CAP-0008's generator screen, regenerate HTML/index/manifest, and
   export its PNG.

Verification gate: product records and the CAP-0008 HTML/PNG agree with the
implemented control and `node docs/product/wireframes/generate.mjs` exits 0.

## Phase 4 — Verify and close

1. Run the workspace format, lint, Rust-test, and frontend-test gates.
2. Complete the DOX pass and update `docs/changes/README.md`.
3. Set this CHG done, archive it, and update the archive index only after all
   gates pass.

## Out of scope

- Rewriting existing folder policies, document overrides, candidate evidence, or
  historical release records.
- Renaming or deleting a confidentiality type ID.
- Bulk release of documents, rewriting unregistered Markdown files, or changing
  Office source markers outside a future release candidate.
