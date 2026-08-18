# Change records

Active implementation progress lives in `active/CHG-*.md`.
Completed changes move to `archive/` as implementation receipts.
A CHG is not a feature specification; CAPs describe current behaviour.

## Rules

- Exactly one active CHG progress authority per material request.
- Link an external ticket, or write `Direct operator request: <verbatim text>`.
  Never invent a ticket ID.
- Keep a single phase `in-progress` at a time.
- Mark a phase `done (<evidence>)` only after its verification gate passes.
- For material work, the tracked CHG is the execution plan. Do not create a
  competing profile-private plan that carries the same progress.
- On close: confirm CAPs, run the integration gate, set status `done`, move to
  `archive/`.

## Active

| ID | Title | Status | CAP impact |
| --- | --- | --- | --- |
| [CHG-0024](active/CHG-0024-windows-nsis-installer-release.md) | Windows NSIS installer and signed GitHub Release | in-progress | CAP-0005 |

## Archive

| ID | Title | Status | CAP impact |
| --- | --- | --- | --- |
| [CHG-0023](archive/CHG-0023-os-level-dms-uri-registration.md) | OS-level `dms://` URI handler registration | done | CAP-0020 |
| [CHG-0020](archive/CHG-0020-foldable-library-panes.md) | Foldable Library side panes | done | CAP-0006 |
| [CHG-0022](archive/CHG-0022-library-table-workflow-columns.md) | Workflow metadata and resizable columns in the Library table | done | CAP-0006 |
| [CHG-0021](archive/CHG-0021-html-notification-permalinks.md) | Clickable permalinks in HTML notification emails | done | CAP-0010 |
| [CHG-0019](archive/CHG-0019-library-refresh-snapshot.md) | Library Refresh re-enumerates the current snapshot | done | CAP-0006 |
| [CHG-0018](archive/CHG-0018-membership-obsolescence-independence.md) | Membership and obsolescence stay independent | done | CAP-0006, CAP-0015 |
| [CHG-0017](archive/CHG-0017-candidate-approval-copy.md) | Candidate form approval-neutral copy | done | CAP-0002, CAP-0015 |
| [CHG-0016](archive/CHG-0016-actions-footer-full-height.md) | Actions footer full height | done | CAP-0006, CAP-0015 |
| [CHG-0015](archive/CHG-0015-fixed-foldable-actions-footer.md) | Fixed foldable Actions footer | done | CAP-0006, CAP-0015 |
| [CHG-0014](archive/CHG-0014-reassociate-source-topic-visibility.md) | Pinned Actions, Lost-source reassociate, native file pick | done | CAP-0006, CAP-0013, CAP-0015 |
| [CHG-0013](archive/CHG-0013-lost-source-library-filter.md) | Lost-source library filter, counter, and reassociate audit | done | CAP-0006, CAP-0013, CAP-0011 |
| [CHG-0001](archive/CHG-0001-tauri-local-dms-bootstrap.md) | Bootstrap Tauri local DMS for ISO 27001 document control | done | CAP-0001 … CAP-0022 |
| [CHG-0002](archive/CHG-0002-entra-configuration-ux-fixes.md) | Entra configuration UX fixes | done | CAP-0021 |
| [CHG-0003](archive/CHG-0003-retry-safe-entra-identity-application.md) | Retry-safe Entra identity-source application | done | CAP-0019, CAP-0021 |
| [CHG-0004](archive/CHG-0004-markdown-word-template-release.md) | Markdown Word-template release pipeline | done | CAP-0001, CAP-0002, CAP-0005, CAP-0006, CAP-0007, CAP-0015 |
| [CHG-0005](archive/CHG-0005-document-defaults-folder-tree.md) | Document defaults folder tree | done | CAP-0008 |
| [CHG-0006](archive/CHG-0006-markdown-frontmatter-template-variables.md) | Markdown frontmatter template variables | done | CAP-0002, CAP-0007 |
| [CHG-0007](archive/CHG-0007-dms-owned-markdown-frontmatter.md) | DMS-owned Markdown frontmatter | done | CAP-0002, CAP-0007, CAP-0015, CAP-0008 |
| [CHG-0008](archive/CHG-0008-markdown-variables-confidentiality-id.md) | Markdown variables reference and confidentiality type ID | done | CAP-0002, CAP-0007, CAP-0008, CAP-0015 |
| [CHG-0009](archive/CHG-0009-selection-pane-foldable-topics.md) | Selection pane foldable topics | done | CAP-0006, CAP-0015 |
| [CHG-0010](archive/CHG-0010-selection-fold-affordance-schedule.md) | Selection fold affordance and review schedule section | done | CAP-0006, CAP-0015 |
| [CHG-0011](archive/CHG-0011-revision-cycle-candidate-clarity.md) | Revision cycle candidate clarity | done | CAP-0002, CAP-0006, CAP-0015 |
| [CHG-0012](archive/CHG-0012-digest-driven-draft-lifecycle.md) | Digest-driven draft lifecycle (drop Begin revision) | done | CAP-0002, CAP-0006, CAP-0011, CAP-0015, CAP-0017 |

## Related

- Capabilities: [`../product/README.md`](../product/README.md)
- Design decisions: [`../design-decisions.md`](../design-decisions.md)
