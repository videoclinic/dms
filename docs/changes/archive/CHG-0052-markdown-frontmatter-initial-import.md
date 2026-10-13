# CHG-0052 — Markdown frontmatter initial import

**Plan ID:** CHG-0052-markdown-frontmatter-initial-import
**Created:** 2026-10-12
**Depends on:** none
**Context sources:** `AGENTS.md`; `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `docs/markdown-frontmatter-and-template-variables.md`; CAP-0002; CAP-0015; CAP-0017; CAP-0019; ADR-0008; `crates/dms-core/src/{frontmatter,lib,lifecycle,maintenance,policies}.rs`.
**Produces:** First registration of a Markdown library member imports selected scalar frontmatter into DMS control and workflow metadata exactly once, with an imported version as the first DMS release target and safe unique fuzzy matching of eligible editor/approver assignments.
**Status:** done

| Field | Value |
| --- | --- |
| ID | CHG-0052 |
| Status | done |
| External request | Direct operator request: If the front matter value for the version should be the starting point for DMS. Add also frontmatter support for review interval (months), editor & approver (parial matching algorithm/fuzzy with eligible people), document type (type id) |
| Affected CAPs | CAP-0002, CAP-0015, CAP-0017, CAP-0019 |
| Decision records | ADR-0008; source-frontmatter authority at initial registration changes while DMS remains authoritative afterwards |

## Contract

- Initial import applies only to a new registered Markdown member. Re-addition, reassociation, later source edits, and all non-Markdown sources do not import frontmatter into `.dms`.
- Supported imported keys are `title`, `document_number`, `version`, `document_type`, `review_interval_months`, `editor`, and `approver`. `confidentiality` remains a DMS policy-derived controlled key and is not an import input.
- An imported `version` is a required `MAJOR.MINOR` baseline for the first DMS release target. It does not create a release record, reserve a version, or import release evidence. The first successful DMS release commits that exact target; later targets follow the ordinary DMS lifecycle.
- `document_type` must name an enabled configured stable type ID. `review_interval_months` must be a positive integer and creates a document schedule override.
- `editor` and `approver` resolve independently against currently eligible cached Entra people. Matching is case-insensitive on normalized display name or email: exact matching wins; otherwise a substring match is accepted only when it yields exactly one eligible person. Zero or multiple matches fail the entire add with an actionable error; no arbitrary or stale identity is selected.
- DMS normalizes and writes its controlled Markdown fields after a successful import. Imported document control and workflow metadata become DMS authority; optional non-controlled frontmatter remains a Word-template-variable source only.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Specify import contract and persistence boundaries | done (CHG-0052 contract written; existing CAP/ADR conflicts identified for Phase 3) | CAP/ADR/CHG contract agrees on first-registration-only authority, baseline-version semantics, and deterministic person matching. |
| 2 | Implement atomic core import and first-release baseline | done (`cargo test -p dms-core --test workspace --test library --test lifecycle --test policies`) | `cargo test -p dms-core --test workspace --test library --test lifecycle --test policies` exits 0 with import, rejection, atomic batch, re-add, migration, and first-target coverage. |
| 3 | Publish records and primary-surface evidence | done (`node docs/product/wireframes/generate.mjs`; CAP-0002 PNG rendered and inspected) | CAP/ADR/operator reference and affected wireframe are current; generator and affected PNG render pass. |
| 4 | Run integration gates and close | done (`cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; Markdown-link check from repository root; `git diff --check`) | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, relevant desktop/frontend tests, Markdown-link check, and `git diff --check` exit 0. |

## Phase 1 — Specify import contract and persistence boundaries

**Goal:** Make the one-time source-to-DMS transfer explicit before changing the old one-way-only frontmatter contract.

Steps:

1. Identified CAP-0002, CAP-0015, CAP-0017, CAP-0019, ADR-0008, architecture, and the frontmatter operator reference as the source-authority contracts to update after implementation proof.
2. Defined the initial version as the exact first DMS release target without inventing a historical release, source archive, or version reservation.
3. Preserved the existing Entra group boundary: no user roster, no arbitrary text-to-identity assignment, and an error on fuzzy ambiguity.

**Verification gate:** CAP/ADR/CHG contract agrees on first-registration-only authority, baseline-version semantics, and deterministic person matching.

## Phase 2 — Implement atomic core import and first-release baseline

**Goal:** Register Markdown source data once through `dms-core` without partial metadata or identity writes.

Steps:

1. Parse the supported scalar keys separately from release-time controlled-frontmatter validation, retaining strict flat-YAML rules and rejecting malformed imported values.
2. Extend prepared-add data and document persistence with the initial baseline necessary to resolve the first release target. Migrate existing workspaces without changing existing documents or release history.
3. Resolve document type, schedule, and independently supplied editor/approver before committing a single add or batch add. Reuse only eligible current-binding people and reject unknown, disabled, malformed, unmatched, or ambiguous input atomically.
4. Cover single and batch import, exact/unique partial matching, ambiguity/zero-match rejection, existing-document re-add/reassociate non-import, schema migration, and the first target followed by normal later version progression.

**Verification gate:** `cargo test -p dms-core --test workspace --test library --test lifecycle --test policies` exits 0 with import, rejection, atomic batch, re-add, migration, and first-target coverage.

## Phase 3 — Publish records and primary-surface evidence

**Goal:** Explain the import boundary where operators see the resulting document control, schedule, and role values.

Steps:

1. Update product CAPs, architecture/ADR, and the frontmatter reference; remove obsolete absolute claims that Markdown frontmatter can never initially populate `.dms`.
2. Update the CAP-0002 Revision-cycle wireframe for the imported first-target baseline; retain CAP-0015 because its existing selection pane already presents the imported control, schedule, and role values.
3. Refresh applicable core and desktop DOX contracts if their durable rules change.

**Verification gate:** CAP/ADR/operator reference and affected wireframe are current; generator and affected PNG render pass.

## Phase 4 — Run integration gates and close

**Goal:** Prove the core and product records agree, then archive the completed progress authority.

Steps:

1. Focused core tests, full Rust workspace tests, Clippy, all desktop frontend tests, wireframe generation/rendering, root-scoped Markdown-link validation, and diff whitespace validation passed.
2. Preserved unrelated CHG-0050 and CAP-0006 wireframe work, then archived this record and refreshed the change index.

**Verification gate:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, relevant desktop/frontend tests, Markdown-link check, and `git diff --check` exit 0.

## Out of scope

- Importing document control, schedules, or workflow roles from Office properties.
- Continuous frontmatter-to-`.dms` synchronization after first Markdown registration.
- Creating a release, importing a PDF, reconstructing source-version history, or reserving a version from source text.
- Matching or creating people outside the workspace's eligible Entra group.
- Changing confidentiality policy authority or optional Word-template variable behaviour.
