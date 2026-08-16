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
| [CHG-0014](active/CHG-0014-reassociate-source-topic-visibility.md) | Pinned Actions, Lost-source reassociate, native file pick | in-progress | CAP-0006, CAP-0013, CAP-0015 |

## Archive

| ID | Title | Status | CAP impact |
| --- | --- | --- | --- |
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
