# CHG-0013 — Lost-source library filter, counter, and reassociate audit

| Field | Value |
| --- | --- |
| ID | CHG-0013 |
| Status | done |
| External request | Direct operator request: Add another filter and type of counting for the library: "(Re-)Moved documents"; these are documents tracked within the library but the corresponding file cannot be detected where it was originally. This is where the "Reassociate source" feature came in: Select a lost file in the folder list view, where it was before (re-)moving and reassociate it with a different file within the edit-subdirectory-tree. In such a case where the document was (re-)moved most of the other actions/functionality have to be disabled until a new reassignment is done. A reassociated source is an audit log for the document (old-folder/old-name => new-folder/new-name of the file). If the reassociation is done with a document in the library, incorporate the "old" audit log to the target audit log. This is only allowed if the target file/source do not have audit log entries that are older than the previous audit log. If the new target source have audit logs that are overlapping from timestamp perspective, a reassociation is not possible. The target source file have to leave the library. Point this out to the user if the user tries to do this. Follow-up: the file list view should show (re-)moved entries e.g. italic with Lifecycle state "Lost source"; do not forget the filter. |
| CAP impact | CAP-0006, CAP-0013, CAP-0011 |

## Goal

Surface registered documents whose draft file is no longer at the stored
edit-root-relative locator as **Lost source** rows in the Library folder list
(with a dedicated session filter and recursive counter), keep non-recovery
actions disabled until reassociation, and record every reassociation in the
canonical workflow chain — including safe audit-history absorption when the
chosen path is already another library document.

## Phases

| # | Phase | Status | Gate |
| --- | --- | --- | --- |
| 1 | CAP/CHG contracts + wording | done (CAP-0006/0011/0013 + CHG index) | Lost source filter/counter/`?`, phantom rows, reassociate audit/merge rules recorded |
| 2 | Core inventory, reassociate, evidence | done (`dms-core` library tests) | Lost rows/counters, `source_reassociated` event, absorb rules |
| 3 | Desktop Library filter/UI + CLI errors | done (frontend library tests) | Filter, italic Lost source rows, gated actions, core error strings surface via adapter |
| 4 | Wireframes + workspace gates | done (HTML/PNG CAP-0006/0013; workspace fmt/test/clippy) | Regenerated screens; `cargo fmt/test/clippy` + node library tests |

## Notes

- Display label is **Lost source** (filter/counter title remains **(Re-)Moved documents**).
- Counter symbol is `?N` beside folder names; unfiltered recursive totals.
- Surviving identity on path-collision reassociate is the document being repaired;
  the registered target leaves the library after a non-overlapping, not-older
  audit merge into the surviving chain.
