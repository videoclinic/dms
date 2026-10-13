# CHG-0050 — UUID dependency MSRV ceiling

The workspace pins Rust 1.88, while `uuid = "1.11"` permits UUID 1.27.0 and later, which require Rust 1.89. The current lockfile resolves UUID 1.24.0 and still builds, but a routine dependency update can select an unsupported release.

**Plan ID:** CHG-0050-uuid-msrv-ceiling
**Created:** 2026-10-12
**Depends on:** none
**Context sources:** `AGENTS.md`; `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `Cargo.toml`; `Cargo.lock`; `rust-toolchain.toml`; `cargo info uuid@1.25.0`, `cargo info uuid@1.26.0`, and `cargo info uuid@1.28.0`; `cargo update -p uuid --precise 1.28.0 --dry-run`.
**Produces:** A UUID version requirement that permits Rust-1.88-compatible UUID updates but rejects UUID releases requiring a newer compiler.
**Status:** planned

| Field | Value |
| --- | --- |
| ID | CHG-0050 |
| Status | planned |
| External request | Direct operator request: Is there a problem with the dependencies? uuid@1.28.0 requires Rust 1.89 while the repository pins Rust 1.88? create or update a CHR if this need to be fixed with the phased-plan-design skill |
| Affected CAPs | none — dependency-resolution and build-compatibility maintenance only |
| Decision records | none |

## Current state

- `rust-toolchain.toml` and the workspace package both require Rust 1.88.
- The lockfile currently uses UUID 1.24.0, and `cargo check --workspace --all-targets --locked` exits 0 on Rust 1.88.
- Cargo's caret requirement for UUID 1.11 allows later 1.x releases. A dry-run update selects UUID 1.28.0, which declares Rust 1.89.0.
- UUID 1.25.0 and 1.26.0 declare Rust 1.85.0. UUID 1.27.0 begins the Rust 1.89 requirement.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Constrain UUID to the Rust-1.88-compatible release line | pending | The manifest rejects UUID 1.27.0, and `cargo check --workspace --all-targets --locked` exits 0 with Rust 1.88. |
| 2 | Run the Rust workspace gate and close the record | pending | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `git diff --check` exit 0. |

## Phase 1 — Constrain UUID to the Rust-1.88-compatible release line

**Goal:** Keep the Rust 1.88 toolchain while allowing UUID updates through the last known compatible 1.x release line.

Steps:

1. Change the workspace UUID requirement from the open caret range to `>=1.11, <1.27`, retaining the existing `serde` and `v4` features.
2. Keep the lockfile on a version satisfying that range; update it only if Cargo needs a lockfile adjustment.
3. Confirm that an attempted dry-run selection of UUID 1.27.0 is rejected by the manifest constraint, then compile all workspace targets with the locked dependency graph on Rust 1.88.

**Verification gate:** `cargo update -p uuid --precise 1.27.0 --dry-run` is rejected by dependency resolution, and `cargo check --workspace --all-targets --locked` exits 0.

## Phase 2 — Run the Rust workspace gate and close the record

**Goal:** Prove the revised dependency policy builds and test coverage remains green, then archive the completed progress authority.

Steps:

1. Run the Rust formatting, Clippy, and workspace test gates plus `git diff --check`.
2. Record the exact evidence only after every gate passes.
3. Set this CHG to done, move it to `docs/changes/archive/`, and refresh `docs/changes/README.md`. Do not change CAPs because no product behaviour changes.

**Verification gate:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `git diff --check` exit 0.

## Out of scope

- Raising the workspace MSRV or Rust toolchain.
- Updating unrelated dependencies.
- CAP, UI, persistence, or runtime-behaviour changes.
