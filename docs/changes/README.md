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

## Archive

| ID | Title | Status | CAP impact |
| --- | --- | --- | --- |
| [CHG-0001](archive/CHG-0001-tauri-local-dms-bootstrap.md) | Bootstrap Tauri local DMS for ISO 27001 document control | done | CAP-0001 … CAP-0022 |
| [CHG-0002](archive/CHG-0002-entra-configuration-ux-fixes.md) | Entra configuration UX fixes | done | CAP-0021 |
| [CHG-0003](archive/CHG-0003-retry-safe-entra-identity-application.md) | Retry-safe Entra identity-source application | done | CAP-0019, CAP-0021 |
| [CHG-0004](archive/CHG-0004-markdown-word-template-release.md) | Markdown Word-template release pipeline | done | CAP-0001, CAP-0002, CAP-0005, CAP-0006, CAP-0007, CAP-0015 |
| [CHG-0005](archive/CHG-0005-document-defaults-folder-tree.md) | Document defaults folder tree | done | CAP-0008 |
| [CHG-0006](archive/CHG-0006-markdown-frontmatter-template-variables.md) | Markdown frontmatter template variables | done | CAP-0002, CAP-0007 |
| [CHG-0007](archive/CHG-0007-dms-owned-markdown-frontmatter.md) | DMS-owned Markdown frontmatter | done | CAP-0002, CAP-0007, CAP-0015, CAP-0008 |
| [CHG-0008](archive/CHG-0008-markdown-variables-confidentiality-id.md) | Markdown variables reference and confidentiality type ID | done | CAP-0002, CAP-0007, CAP-0008, CAP-0015 |

## Related

- Capabilities: [`../product/README.md`](../product/README.md)
- Design decisions: [`../design-decisions.md`](../design-decisions.md)
