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
| [CHG-0001](active/CHG-0001-tauri-local-dms-bootstrap.md) | Bootstrap Tauri local DMS for ISO 27001 document control | in-progress | CAP-0001 … CAP-0020 |

## Archive

*(empty — no completed changes yet.)*

## Related

- Capabilities: [`../product/README.md`](../product/README.md)
- Design decisions: [`../design-decisions.md`](../design-decisions.md)
