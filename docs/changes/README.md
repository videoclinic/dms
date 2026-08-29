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
| [CHG-0036](active/CHG-0036-pending-approval-status-and-review-request-resend.md) | Pending-approval status and review-request resend | in-progress | CAP-0002, CAP-0006, CAP-0010, CAP-0011 |
| [CHG-0037](active/CHG-0037-library-switcher-and-lock-owner-feedback.md) | Library switcher and lock-owner feedback | pending | CAP-0005, CAP-0014 |
| [CHG-0038](active/CHG-0038-candidate-approval-outcome-clarity.md) | Candidate, release, and Entra identity clarity | pending | CAP-0002, CAP-0006, CAP-0011, CAP-0015, CAP-0021 |
| [CHG-0039](active/CHG-0039-library-table-sort-and-user-layout.md) | Library table sort direction and user layout | pending | CAP-0005, CAP-0006 |
| [CHG-0040](active/CHG-0040-document-type-id-migration.md) | Document type-ID migration | pending | CAP-0001, CAP-0002, CAP-0013, CAP-0015 |
| [CHG-0041](active/CHG-0041-library-batch-execution-activity.md) | Library batch execution activity | pending | CAP-0005, CAP-0006 |
| [CHG-0042](active/CHG-0042-publish-root-released-document-index.md) | Publish-root released-document index | pending | CAP-0005, CAP-0016 |
| [CHG-0024](active/CHG-0024-windows-nsis-installer-release.md) | Windows NSIS installer and signed GitHub Release | in-progress | CAP-0005 |
| [CHG-0025](active/CHG-0025-windows-entra-deployment-policy.md) | Windows Entra deployment policy | in-progress | CAP-0021 |
| [CHG-0045](active/CHG-0045-operator-library-name.md) | Operator-defined library name | pending | CAP-0001, CAP-0005 |


## Archive

| ID | Title | Status | CAP impact |
| --- | --- | --- | --- |
| [CHG-0044](archive/CHG-0044-entra-device-flow-poll-and-open.md) | Entra device-flow poll, reissue, and Open library | done | CAP-0005, CAP-0021 |
| [CHG-0043](archive/CHG-0043-blocked-library-recovery-navigation.md) | Blocked-library recovery navigation | done | CAP-0005, CAP-0021 |
| [CHG-0035](archive/CHG-0035-entra-session-required-for-group-bound-library.md) | Entra session required for a group-bound library | done | CAP-0011, CAP-0021 |
| [CHG-0033](archive/CHG-0033-bounded-office-source-history-import.md) | Bounded Office source-history import | done | CAP-0006, CAP-0011, CAP-0012, CAP-0015 |
| [CHG-0032](archive/CHG-0032-windows-edit-root-library-shortcut.md) | Windows edit-root library shortcut | done | CAP-0001, CAP-0006, CAP-0020 |
| [CHG-0031](archive/CHG-0031-library-change-author-identity.md) | Library change-author identity | done | CAP-0005 |
| [CHG-0034](archive/CHG-0034-user-scoped-notification-transport.md) | User-scoped notification transport | done | CAP-0001, CAP-0002, CAP-0010, CAP-0017 |
| [CHG-0030](archive/CHG-0030-confidentiality-type-id-migration.md) | Confidentiality type-ID migration | done | CAP-0002, CAP-0008, CAP-0013, CAP-0015 |
| [CHG-0029](archive/CHG-0029-newest-history-entry-default.md) | Newest version-history entry expanded by default | done | CAP-0011 |
| [CHG-0028](archive/CHG-0028-document-pane-defaults.md) | Document-pane initial disclosure state | done | CAP-0006, CAP-0011, CAP-0015 |
| [CHG-0027](archive/CHG-0027-version-history-discoverability.md) | Discoverable version history and person-consolidated changes | done | CAP-0006, CAP-0011, CAP-0015 |
| [CHG-0026](archive/CHG-0026-person-consolidated-workflow-evidence.md) | Person-consolidated workflow evidence without digest values | done | CAP-0011 |
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
