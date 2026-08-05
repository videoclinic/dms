# Change records

## Purpose

Own implementation-progress authority (`CHG-*.md`) for material work.

## Ownership

| Path | Owns |
| --- | --- |
| `README.md` | Active/archive index and CHG rules |
| `active/CHG-*.md` | In-flight progress, phases, gates |
| `archive/CHG-*.md` | Closed implementation receipts |

## Local Contracts

- Exactly one progress authority per material request while active.
- External request field is a ticket link or verbatim `Direct operator request:`.
- Single phase `in-progress` at a time; `done (<evidence>)` only after the gate passes.
- On close: status `done`, move file to `archive/`, refresh `README.md`.
- CHG is not a CAP substitute.

## Work Guidance

- Resume work from the active CHG, not from chat memory or profile-private plans.
- Split only when scope truly forks into independent requests (new CHG IDs).

## Verification

- Active index matches files under `active/`.
- No CHG in `active/` marked `done`.
- Affected CAP IDs referenced by each CHG exist under `../product/capabilities/`.

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`. Product sibling: `../product/AGENTS.md`.
