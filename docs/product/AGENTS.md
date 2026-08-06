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
- Keep filesystem-derived **Source file** name/path, DMS-managed **Document
  control data**, and Office document properties as explicitly distinct data
  sources. Do not relabel document control data as "Master data" or imply Office
  property synchronization without changing the owning CAP contract.

## Work Guidance

- Material outcome change → edit CAP + proving tests + active CHG together.
- Do not churn CAPs for pure refactors with no user-visible change.
- Keep the Library surface and its wireframe folder-dominant. Follow CAP-0006's
  familiar Windows Explorer-like navigation model rather than reducing folders
  to a compact filter beside a document-first table.

## Verification

- Every `capabilities/CAP-*.md` appears in `README.md`.
- CAP status matches working tree reality (no false “implemented”).

## Child DOX Index

| Child | Owns | Read when editing… |
| --- | --- | --- |
| `wireframes/AGENTS.md` | CAP wireframe HTML/PNG assets and generator | Wireframe screens, exports, or CAP visual references |

Parent: `../AGENTS.md`. Progress sibling: `../changes/AGENTS.md`.
