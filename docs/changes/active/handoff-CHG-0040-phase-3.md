# CHG-0040 Phase 3 durable handoff

**Prepared:** 2026-10-11T18:17:40+02:00
**Checkpoint:** Phase 3 integrated after all gates, visual inspection, a confirmed documentation correction, and reviewer-model clearance

## Goal and constraints

Execute CHG-0040 Phase 3 from [the task packet](task-packet-CHG-0040-phase-3.md). Publish CAP and wireframe evidence for the already-implemented document-type migration, then close and archive the CHG. Preserve the primary checkout's Phase 1/2 diff. Do not commit, push, deploy, access secrets, or make external changes.

## Workspace and base

- Primary repository: `/mnt/c/Users/RaphaelBossek/src/dms`
- Branch/HEAD: `main` / `a75e524ebb192c51026273d6405b4c427aef9203`
- Base merge point: `7740a538c3883397a5b24c90e54f13261b1472c5`
- Current tracked overlay SHA-256: `27e2fb1294e3624192f138b7d10fe102bfcdb27d536f4e5fc8465660f7898a8a`
- Current tracked paths: Phase 1/2 desktop code/tests/styles; `docs/changes/README.md`; and `docs/changes/active/CHG-0040-document-type-id-migration.md`.
- Phase 3 was marked `in-progress` in the CHG and active index. It is the sole in-progress CHG-0040 phase.

## Current evidence

- Phase 1 and Phase 2 are implemented, independently tested, and reviewer-model reviewed with `No findings`; detail is in `handoff-CHG-0040-phase-2.md`.
- `google-chrome` is available at `/home/raphael/.local/bin/google-chrome`; Node is `v26.8.2`; Cargo is Rust `1.88.0`.
- `docs/product/wireframes/generate.mjs` is the only editable source for generated CAP HTML/index/manifest. CAP-0013 currently presents a maintenance action list; CAP-0015 uses the shared detail selection pane. Generated artifacts must not be hand-edited.

## Required execution sequence

1. Create one isolated worktree from the recorded HEAD, verify the current primary overlay hash, and apply a reviewed snapshot of that overlay. Copy this task packet and handoff into the worktree for the executor.
2. Start a fresh configured `astra` implementation session after the operator selects and Herdr validates an idle executor pane. Implement the full CAP/wireframe/archive slice in the worktree.
3. Independently inspect the final worktree diff; rerun every Phase 3 command named in the task packet. Render both specified PNGs and visually inspect them before review.
4. Start a fresh configured `grok` read-only reviewer session with the task, base, changed paths, diff, gate outcomes, and image-inspection evidence. Triage any finding with a fresh `astra` correction session and final review only if the diff changes.
5. Apply only the reviewer-cleared delta to the primary checkout after an exact overlay preimage check; rerun relevant integration checks there. Update this handoff with evidence and disposition.

## Review findings and dispositions

The configured `grok` review found one medium documentation defect in CAP-0002 outcome 4: after the new document-type-migration sentences, `It may also stage` could bind Owner/Editor staging to the migration rather than the candidate. A fresh `astra` correction session changed it to `The candidate may also stage`. A fresh configured `grok` read-only review then returned `No findings`.

## Final integration evidence

- The primary overlay preimage matched `27e2fb1294e3624192f138b7d10fe102bfcdb27d536f4e5fc8465660f7898a8a`; the reviewer-cleared Phase 3 tracked delta passed `git apply --check` and was applied without overwriting Phase 1/2 changes. The archived CHG receipt was copied from the reviewed worktree.
- Primary verification passed: generator; both exact 1600×1600 Chrome screenshot commands; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` (188 passed); `node --test crates/dms-desktop/ui/*.test.mjs` (145 passed); and `git diff --check`.
- Primary PNG inspection found no material clipping or overlap: CAP-0013 migration controls are legible; CAP-0015 shows both release-boundary paragraphs and all six Actions labels fully.
- No commit, push, deployment, credential access, or external mutation occurred.

## Next bounded phase

CHG-0040 is complete and archived. Do not start another CHG without explicit operator direction.

## Open risks

- CAP-0013 remains broadly `not implemented`; only its document-type catalogue slice is documented as implemented.
- Native Windows/macOS, installed-Office, and live WebView checks remain outside this Linux/WSL checkpoint.
- Phase 2/3 sidecars remain under `active/` as handoff receipts and link to the archived CHG.
