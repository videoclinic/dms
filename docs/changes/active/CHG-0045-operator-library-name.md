# CHG-0045 — Operator-defined library name

An operator can set a custom display name on a library, change it at any time, and reuse the same name on other libraries. Identity stays the workspace ID; the edit-root path stays the locator.

**Plan ID:** CHG-0045-operator-library-name
**Created:** 2026-08-29
**Depends on:** none
**Entry checkpoint:** none
**Context sources:** `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/capabilities/CAP-0001-local-folder-dms.md` (Outcomes 2, 8–10); `docs/product/capabilities/CAP-0005-desktop-shell.md` (quoted library names); `crates/dms-core/src/lib.rs` (`SCHEMA_VERSION` 18, `Workspace`); `crates/dms-desktop/src/lib.rs` (`library_session_target`); `crates/dms-desktop/ui/app.mjs` (`recentLibraryLabel`); `crates/dms-desktop/ui/configuration.mjs` (`workspaceMarkup`)
**Produces:** `.dms` stores an optional library name; Configuration → Workspace and initialize can set it; recent libraries, shell copy, and session headings use it; duplicate names are allowed.
**Status:** pending — plan authored, not started

| Field | Value |
| --- | --- |
| ID | CHG-0045 |
| Status | pending |
| External request | Direct operator request: Allow to define a custom name for a library, and change it anytime. Names do not have to be unique. |
| Affected CAPs | CAP-0001, CAP-0005 |
| Decision records | none |

## Current state

- Display labels are the last path segment of the edit root (`library_session_target` in `crates/dms-desktop/src/lib.rs`; `recentLibraryLabel` in `crates/dms-desktop/ui/app.mjs`).
- `Workspace` has no name field. Schema is 18 (`crates/dms-core/src/lib.rs`). Identity is `workspace_id`; roots are locators (`CAP-0001` outcomes 2 and 4).
- Configuration → Workspace shows ID, edit root, publish root, and document count. It has no rename control (`workspaceMarkup` in `crates/dms-desktop/ui/configuration.mjs`).
- Recent libraries store at most ten unique edit-root paths in OS-user preferences, not `.dms` (`CAP-0001` outcome 10). Each row already shows the path under a strong label.
- Descriptive copy already quotes the library label in ASCII double quotes (`CAP-0005`).

## Risk call-out

Schema 18 → 19 rewrites `workspace.json`. Use the existing open-time migration: retain `workspace.v18.json.bak`, validate the new shape, then atomically replace. If migration fails, restore from that backup and do not retry on a half-written file.

Do not treat the name as identity, a uniqueness key, or a permalink. Do not write it into OS-user recent-library preferences as a second source of truth.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Persist library name in `.dms` | pending | `cargo test -p dms-core` exits 0 with v18→v19 migration, blank/whitespace fallback, rename, and two workspaces sharing one name |
| 2 | Desktop set, rename, and display | pending | `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs` and `cargo test -p dms-desktop --lib` exit 0 |
| 3 | CAP, wireframe, workspace gate | pending | `node docs/product/wireframes/generate.mjs`; CAP-0001 and CAP-0005 PNG renders; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check` |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, `pending` otherwise.

## Phase 1 — Persist library name in `.dms`

**Goal:** A workspace stores an optional operator library name; missing or blank after trim keeps the edit-root folder name; two workspaces may share a name.

Steps:

1. Add `library_name: Option<String>` on `Workspace`. Trim on write. Empty after trim stores `None`.
2. Bump `SCHEMA_VERSION` to 19. Migrate v18 by leaving `library_name` unset. Keep `workspace.v18.json.bak`.
3. Add `Workspace::set_library_name`. Do not scan other workspaces. Do not append workflow evidence for a display-name change.
4. Expose the effective label (stored name, else folder name, else the edit-root string) from core so CLI and desktop share one rule.

Verification gate: `cargo test -p dms-core` exits 0 with v18→v19 migration, blank/whitespace fallback, rename, and two workspaces sharing one name.

## Phase 2 — Desktop set, rename, and display

**Goal:** The operator sets or changes the name from Configuration → Workspace and optionally at initialize; every descriptive library label uses the effective name.

Steps:

1. Configuration → Workspace: editable **Library name**, Save. Show the edit-root path as identity, not as the name.
2. Initialize: optional name field. Omit it to keep the folder-name fallback.
3. Replace `library_session_target` and `recentLibraryLabel` with the effective name. Recent rows keep the path as the subtitle so duplicates stay distinguishable. Best-effort read `.dms` for each recent root; if it is unreadable, keep the folder-name fallback.
4. Quoted copy already wraps the label; keep that. Do not put the name in permalinks, `Open in DMS.lnk`, or recent-library preference keys.

Verification gate: `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/configuration.test.mjs` and `cargo test -p dms-desktop --lib` exit 0.

## Phase 3 — CAP, wireframe, workspace gate

**Goal:** CAP-0001 and CAP-0005 state the name as display-only, changeable, and non-unique.

Steps:

1. CAP-0001: optional `library_name` in `.dms`; identity remains workspace ID; blank restores the folder-name fallback.
2. CAP-0005: shell, recent libraries, and quoted copy use the effective name; duplicate names are allowed; the path remains visible on recent rows.
3. Update Configuration Workspace and recent-library wireframes. Regenerate CAP-0001 and CAP-0005 HTML/PNG.
4. Refresh `crates/dms-core/AGENTS.md` and `crates/dms-desktop/AGENTS.md` if local contracts change. Run the workspace gate. Archive this CHG only after that gate passes.

Verification gate: `node docs/product/wireframes/generate.mjs`; CAP-0001 and CAP-0005 PNG renders; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `node --test crates/dms-desktop/ui/*.test.mjs`; `git diff --check`.

## Out of scope

- Using the name as identity, a uniqueness constraint, or a permalink/shortcut target.
- Storing the name in OS-user recent-library preferences.
- Auto-renaming when the edit-root folder is renamed on disk.
- Entra group labels or document titles.
