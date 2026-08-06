# Product capabilities

## Purpose

Own falsifiable capability contracts (`CAP-*.md`) and the product index.

## Ownership

| Path | Owns |
| --- | --- |
| `README.md` | CAP index and product-record rules |
| `capabilities/CAP-*.md` | Per-capability outcomes, status, test links |
| `wireframes/` | Static CAP wireframes (HTML + PNG), shadcn-admin visual base |

## Local Contracts

- One CAP per capability boundary; IDs are stable (`CAP-NNNN`).
- Present-tense outcomes only when implemented; otherwise `Status: not implemented`.
- Link ADRs/architecture/privacy; do not duplicate them.
- Update `README.md` index when adding/removing a CAP.

## Work Guidance

- Material outcome change → edit CAP + proving tests + active CHG together.
- Do not churn CAPs for pure refactors with no user-visible change.

## Verification

- Every `capabilities/CAP-*.md` appears in `README.md`.
- CAP status matches working tree reality (no false “implemented”).

## Child DOX Index

| Child | Owns | Read when editing… |
| --- | --- | --- |
| `wireframes/AGENTS.md` | CAP wireframe HTML/PNG assets and generator | Wireframe screens, exports, or CAP visual references |

Parent: `../AGENTS.md`. Progress sibling: `../changes/AGENTS.md`.
