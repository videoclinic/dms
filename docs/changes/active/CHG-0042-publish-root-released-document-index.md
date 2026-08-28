# CHG-0042 — Publish-root released-document index

**Plan ID:** CHG-0042-publish-root-released-document-index
**Execution slot:** P0830
**Created:** 2026-08-28
**Depends on:** CHG-0037-library-switcher-and-lock-owner-feedback#phase-1
**Entry checkpoint:** CHG-0037 Phase 1 is done with its active-session switching gate evidence; its switch path retains the old workspace until the new one can safely replace it.
**Context sources:** `AGENTS.md` (Architectural decisions, Application records); `docs/AGENTS.md`; `docs/changes/AGENTS.md`; `docs/product/AGENTS.md`; `docs/product/wireframes/AGENTS.md`; `docs/architecture.md` (Runtime shape, publish root); `docs/product/capabilities/CAP-0004-release-integrity.md`; `docs/product/capabilities/CAP-0005-desktop-shell.md` (Outcomes 3, 6, 16); `docs/product/capabilities/CAP-0016-publish-tree-maintenance.md`; `crates/AGENTS.md`; `crates/dms-core/AGENTS.md`; `crates/dms-desktop/AGENTS.md`; `crates/dms-core/src/maintenance.rs` (`Workspace::verify_release`, `Workspace::verify_all_releases`); `crates/dms-desktop/src/lib.rs` (`release_maintenance`); `crates/dms-desktop/ui/maintenance.mjs`; `crates/dms-desktop/ui/app.mjs` (`switchWorkspaceSession`, `closeWorkspaceSession`, `registerWindowCloseHandler`); `crates/dms-desktop/ui/maintenance.test.mjs`; `crates/dms-desktop/ui/app.test.mjs`; `docs/product/wireframes/generate.mjs`.
**Produces:** Each publish root contains a DMS-owned, self-contained `index.html` that presents one current released PDF per document in its mirrored folder hierarchy, opens that PDF by a relative link, searches and filters only release metadata, defaults to newest release date, and supports alternate sorting. DMS can refresh it on demand and refreshes it before a workspace is cleanly closed or switched when its deterministic contents differ.
**Status:** pending — implementation has not begun.

DMS will generate the publish root’s `index.html` from immutable release records without replacing the existing Releases / Release history & integrity interface. The generated library lists only each document’s latest non-withdrawn released PDF; older and withdrawn records remain discoverable and verifiable only through the existing DMS release-history and integrity surfaces.

| Field | Value |
| --- | --- |
| ID | CHG-0042 |
| Status | pending |
| External request | Direct operator request: "Create for the \"publish\" directory a self contained \"index.html\" file like a document library for the released documents only using the same directory structure. A search bar allows to look for these documents. It's a transformation of \"Release history and integrity\" but not a replacement. Integrate this \"index.html\" in DMS. The \"index.html\" should be updated on request or if the library is closed (DMS is exited or library switching CHG-0037) and new releases appeared in the meantime. The user should be able to open the documents shown there. The default sort order of the documents should be the last release date. The user have the ability to change the sort order e.g. by document name. Also filters for \"type-id\" (e.g. on/off) should also be possible" |
| Affected CAPs | CAP-0005, CAP-0016 |
| Decision records | No new ADR. The index is a deterministic derived artifact in the existing publish root; `.dms` remains the release-record authority and the existing history/integrity workflow remains unchanged. |

## Current state

- `release_maintenance` already projects every recorded release into `ReleaseRow`, including immutable title, version, relative PDF path, type ID, release timestamp, withdrawal state, and integrity verdict; it sorts those rows newest-first (`crates/dms-desktop/src/lib.rs:2881-2929`).
- The current `ReleaseMaintenance` projection verifies each record while listing it, so it cannot be reused as the close-time generator without making normal close/switch dependent on integrity scans (`crates/dms-desktop/src/lib.rs:2887-2890`).
- `Workspace::verify_release` and `Workspace::verify_all_releases` re-read PDF bytes but never repair them; integrity verification is a distinct core operation (`crates/dms-core/AGENTS.md:108-110`; `crates/dms-core/src/maintenance.rs:205-211`).
- Existing CAP-0016 defines per-document released-version history, host opening, title filtering, pagination, verification, orphan handling, and immutable releases; it does not define a portable publish-root catalog (`docs/product/capabilities/CAP-0016-publish-tree-maintenance.md:14-49`).
- The desktop switch transaction acquires the destination lock then releases the old lock before returning (`crates/dms-desktop/ui/app.mjs:685-724`), while clean window close releases the active lock then destroys the window (`crates/dms-desktop/ui/app.mjs:2712-2743`). Both need an index-sync call before the active lock/session is released.
- The publish root mirrors the edit-relative tree and is a storage destination rather than a lifecycle state (`docs/architecture.md:34-60`; `docs/product/AGENTS.md:32-34`).

## Risk call-out

`<publish-root>/index.html` is a user-visible derived artifact beside controlled PDFs. DMS must never overwrite an unrelated pre-existing `index.html`: a file without the exact DMS ownership marker is a refusal with an actionable error. The generator writes only a fully rendered replacement through an atomic same-directory rename, leaves the prior DMS-generated index intact if rendering or writing fails, and never changes `.dms`, release records, or PDF bytes.

A failed automatic refresh must not silently leave a stale index behind. Before window close or a different-library handoff, DMS refreshes the still-active source workspace; a failure reports the path-specific error and aborts closing/switching while retaining the current session and owner-matched lock. A no-change refresh avoids replacing the file. The generated page never loads remote code or assets and only emits encoded relative links for current release paths that remain inside the publish root.

## Phases

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Build deterministic core index generation and ownership-safe writes | pending | `cargo test -p dms-core publish_index` exits 0 with current-release selection, mirrored-tree rows, deterministic newest-first source order, escaped/encoded output, type-ID data, no-change detection, atomic replacement, and foreign-index refusal coverage. |
| 2 | Integrate explicit refresh and close/switch refresh in DMS Desktop | pending | `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/maintenance.test.mjs` exit 0 with manual refresh, host-open, no-change refresh, close ordering, successful switch ordering, and failed-sync retention coverage. |
| 3 | Publish contracts, wireframe evidence, and close the change | pending | `node docs/product/wireframes/generate.mjs`, CAP-0005/CAP-0016 PNG renders, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, the Markdown-link check, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree. |

Mark a phase `in-progress` while running it, `done (<evidence>)` once its gate passes, and `pending` otherwise.

## Phase 1 — Build deterministic core index generation and ownership-safe writes

**Goal:** `dms-core` can derive and atomically write an owned self-contained publish-root catalog without scanning source drafts, mutating release state, or performing checksum verification.

Steps:

1. Add a core-owned publish-index projection and refresh operation beside the existing release-maintenance domain. Select one row per document: its latest non-withdrawn release, including immutable captured title (use a truthful legacy-unrecorded label), document number when captured, version, release timestamp, relative PDF path, and confidentiality type ID. Exclude documents with no current release, missing current PDF, withdrawn historical releases, draft files, source paths, and orphaned historical records.
2. Build the index hierarchy solely from each selected release’s recorded relative PDF parent path. Render an accessible folder tree that filters the document table to its subtree, plus a table/card row that opens the selected PDF through a URI-encoded relative `href`. Do not scan the publish filesystem to discover arbitrary PDFs or infer release records.
3. Render one standalone HTML document with inline CSS and JavaScript, a DMS ownership marker, and no network URLs, runtime dependencies, or remote assets. The initial client state sorts by release timestamp descending with deterministic name/path tie-breakers. Provide a case-insensitive search across displayed release metadata, a type-ID filter with explicit on/off controls for every present type ID plus All, and a sort control for last release date and document name in both directions. Apply search, type filter, folder selection, and sort before rendering the result count/empty state.
4. Treat page data as untrusted display text: HTML-escape every label, never interpolate a raw release path into markup or script literals, and URI-encode path segments for anchors. Verify each emitted path remains a normalized relative descendant of `publish_root`; refuse an invalid record rather than linking outside the tree.
5. Add `Workspace::refresh_publish_index` (or equivalently named explicit public API) that writes only `<publish-root>/index.html`. If the output bytes equal an existing DMS-owned index, return unchanged. If an existing file lacks the ownership marker, refuse without replacement. Otherwise write a temporary sibling, flush it, atomically rename it into place, and clean up a failed temporary write. Do not add `.dms` schema state merely to remember refresh time.
6. Add focused core tests with temporary dual roots: nested current releases form the expected tree and relative links; a later non-withdrawn release replaces a document’s catalog row; withdrawn and historical releases remain absent; metadata text and path punctuation cannot inject markup; search/sort/type data are present; unchanged output avoids a replacement; a foreign `index.html` remains byte-identical after refusal; and a failed write preserves the prior DMS-owned index.

**Verification gate:** `cargo test -p dms-core publish_index` exits 0 with current-release selection, mirrored-tree rows, deterministic newest-first source order, escaped/encoded output, type-ID data, no-change detection, atomic replacement, and foreign-index refusal coverage.

## Phase 2 — Integrate explicit refresh and close/switch refresh in DMS Desktop

**Goal:** An operator can explicitly refresh/open the generated library, and DMS never releases the active workspace session before synchronizing a changed catalog.

Steps:

1. Expose narrow desktop adapter commands for refreshing the active workspace’s publish index and opening its existing generated index through the host handler. The open command refuses a missing index with a clear instruction to refresh it first; it never opens a source draft or an arbitrary path.
2. Add an explicit **Update released document index** action to the existing Releases / Release history & integrity surface, followed by an **Open released document index** action when the owned index exists. Present whether the request wrote a new file or was already current, and surface a foreign-file/write error in that surface without claiming refresh success. Do not remove, rename, or weaken existing release history, integrity-verification, per-version Open PDF, pagination, or filtering controls.
3. Before `closeWorkspaceSession` releases the active lock, invoke the refresh command for its active workspace. On refresh success or no-change, continue the existing lock-release/destroy flow. On refresh failure, retain the app window, workspace, activity state, and lock; present the failure so the operator can retry manually or resolve the publish-root condition.
4. Update the activation/switch flow after CHG-0037 Phase 1 so it refreshes the currently active workspace before calling `switchWorkspaceSession`. A no-change/current result proceeds to the existing acquire-destination → release-source transaction. A failure does not acquire the destination or discard/reset the old session. Do not run the destination index refresh during source-session close; it will be refreshed when that workspace later closes or on explicit request.
5. Add adapter and frontend tests that prove the manual action invokes only the active workspace; open is host-mediated and requires the owned file; unchanged refresh retains the old file; window close calls refresh before `release_workspace_lock`; a successful switch refreshes the old workspace before acquisition/release/reset; and every refresh failure leaves the old session lock/activity state intact. Preserve CHG-0037’s blocked-destination and old-lock-release rollback coverage.

**Verification gate:** `cargo test -p dms-desktop` and `node --test crates/dms-desktop/ui/app.test.mjs crates/dms-desktop/ui/maintenance.test.mjs` exit 0 with manual refresh, host-open, no-change refresh, close ordering, successful switch ordering, and failed-sync retention coverage.

## Phase 3 — Publish contracts, wireframe evidence, and close the change

**Goal:** Product records, generated review evidence, code, and tests distinguish the portable current-release catalog from DMS’s release history and integrity workflows.

Steps:

1. Amend CAP-0016 to state the generated publish-root `index.html` contract: one current non-withdrawn released PDF per document; hierarchy derived from recorded relative release paths; encoded relative PDF opening; inline/no-network implementation; newest-release-date default sort; document-name alternative sort; metadata search; type-ID filters; explicit refresh; and clean close/switch refresh only when deterministic output differs. State that historical, withdrawn, orphaned, and integrity-verification records stay in the DMS maintenance surface.
2. Amend CAP-0005 only as needed to name the Releases-surface refresh/open actions and the close/switch failure boundary. Keep the shell’s established host-mediated file-open and session-only activity contracts intact.
3. Update `crates/dms-core/AGENTS.md` with the durable derived-artifact boundary and `crates/dms-desktop/AGENTS.md` with refresh-before-lock-release behavior. Leave parent AGENTS files unchanged unless the DOX pass finds a changed ownership/index contract.
4. Update the CAP-0016 screen definition in `docs/product/wireframes/generate.mjs` with synthetic nested folders, release-date default sorting, searchable document rows, type-ID on/off filters, the update/open index controls, and a visible distinction from history/integrity actions. Regenerate HTML, `index.html`, and `manifest.json`; render and visually inspect the CAP-0016 PNG. Do not create a runtime index mock in product wireframe output or hand-edit generated wireframes.
5. Update CAP test links/status to match the implemented evidence, then record passing phase evidence, mark phases done, archive this CHG, and update `docs/changes/README.md` so CHG-0042 appears only in Archive. Do not modify unrelated active records.

**Verification gate:** `node docs/product/wireframes/generate.mjs`, `(cd docs/product/wireframes && google-chrome --headless=new --hide-scrollbars --window-size=1600,1600 --screenshot=exports/CAP-0016-publish-tree-maintenance.png "file://$PWD/html/CAP-0016-publish-tree-maintenance.html" && test -s exports/CAP-0016-publish-tree-maintenance.png)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `node --test crates/dms-desktop/ui/*.test.mjs`, `python3 "$HOME/.hermes/profiles/hermes-vc/skills/software-development/check-md-links/scripts/check-md-links.py" --format summary docs`, and `git diff --check` exit 0; CAP/CHG indexes and DOX contracts agree.

## Out of scope

- Replacing or hiding the Releases / Release history & integrity surface, per-version history, checksum verification, orphan handling, or existing PDF-opening actions.
- Indexing drafts, unregistered files, manually copied PDFs, every historical version, or withdrawn releases in the publish-root catalog.
- Adding a publish-root watcher, background job, database, per-user index preferences, remote assets, or a new CLI command.
- Overwriting, deleting, renaming, or taking ownership of a pre-existing non-DMS `index.html`.
