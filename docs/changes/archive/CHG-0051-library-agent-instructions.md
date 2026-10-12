# CHG-0051 — Library agent instructions

**Plan ID:** CHG-0051-library-agent-instructions
**Created:** 2026-10-12
**Depends on:** none
**Context sources:** `AGENTS.md`; `docs/AGENTS.md`; `docs/product/AGENTS.md`; `docs/changes/AGENTS.md`; `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-cli/AGENTS.md`; CAP-0001; CAP-0022.
**Produces:** Every initialized library has DMS-managed `.dms/AGENTS.md` instructions for agentic CLI use and workspace metadata. A DMS upgrade preserves a locally modified instruction file as timestamped merge input before installing the current original.
**Status:** done

| Field | Value |
| --- | --- |
| ID | CHG-0051 |
| Status | done |
| External request | Direct operator request: For a library create an .dms/AGENTS.md file that describes how `dms` can be used from cli and how the librarys workspace.json file works in order to make agentic changes. An existing AGENTS.md file should not be overwritten (by an DMS version bump) as long it's not the orignial variant. In that case move the modified .dms/AGENTS.md to .dms/AGENTS-<STAMP>.md and write the actual .dms/AGENTS.md and let the user "merge" the changes. |
| Affected CAPs | CAP-0001, CAP-0022 |
| Decision records | none |

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Persist and synchronize DMS-managed library instructions | done (`cargo test -p dms-core`) | `cargo test -p dms-core` exits 0, including initialization, upgrade, and locally modified-file preservation coverage. |
| 2 | Prove the CLI exposes the resulting library contract | done (`cargo test -p dms-cli`) | `cargo test -p dms-cli` exits 0, including `workspace init` creation of `.dms/AGENTS.md`. |
| 3 | Integrate current-state records and close | done (`cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `git diff --check`) | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `git diff --check` exit 0. |

## Phase 1 — Persist and synchronize DMS-managed library instructions

**Goal:** Initialize a canonical agent instruction file, record its exact original fingerprint in workspace metadata, and safely replace it on a later template revision.

Steps:

1. Add the versioned instruction template and its tracked revision/digest state to `dms-core` workspace persistence.
2. Raise the schema with an explicit v19-to-v20 migration and retain the existing `workspace.v19.json.bak` receipt.
3. On initialization, write the canonical `.dms/AGENTS.md`. On open, replace an earlier original when its recorded digest still matches. If the existing file is not the recorded original, rename it to `.dms/AGENTS-<STAMP>.md`, write the canonical file, and leave the timestamped file for an operator merge.
4. Cover initialization, prior-schema migration, original replacement, and preservation of modified instructions without reading or changing user source documents.

**Verification gate:** `cargo test -p dms-core` exits 0, including initialization, upgrade, and locally modified-file preservation coverage.

## Phase 2 — Prove the CLI exposes the resulting library contract

**Goal:** Confirm `dms workspace init` makes the documented agent instructions available to headless automation.

Steps:

1. Extend the end-to-end CLI initialization test to assert the generated instruction file is present and carries the canonical CLI and metadata guidance.
2. Keep the CLI command surface unchanged; the file is a library artifact created by the shared core.

**Verification gate:** `cargo test -p dms-cli` exits 0, including `workspace init` creation of `.dms/AGENTS.md`.

## Phase 3 — Integrate current-state records and close

**Goal:** Align the capability contracts and DOX with the shipped behavior, run the workspace gate, and archive this receipt.

Steps:

1. Update CAP-0001 and CAP-0022 with the persisted agent-instruction and safe-upgrade behavior plus executable test links. No DMS Desktop interaction changes, so the existing CAP-0001 wireframe remains current.
2. Update the relevant source DOX with the new `.dms` artifact and schema-v20 migration rule.
3. Run the Rust workspace gate and `git diff --check`; then mark the CHG done, move it to `archive/`, and refresh the change index.

**Verification gate:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `git diff --check` exit 0.

## Out of scope

- Parsing, applying, or automatically merging local instruction customizations.
- Changing CLI command names or exposing a desktop interaction for this artifact.
- Storing credentials, source content, or agent conversation data in `workspace.json`.
