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
  control data**, Office document properties, and Markdown front matter as
  explicitly distinct data sources. Do not relabel document control data as
  "Master data" or imply source-metadata synchronization without changing the
  owning CAP contract.
- In wireframe tables, use **Name** only for the exact filesystem-derived source
  file name and **Title** for the DMS-managed `title` field. Do not abbreviate
  that field as "Doc" or call it "Master data".
- CAP-0005 owns open-activity naming and reuse: document panes use task +
  DMS title + optional document number, while their stable identity is workspace
  + task + document ID.
- **Publish root** and **publish tree** name the filesystem destination and its
  views only. **Release** is the sole workflow action and lifecycle transition
  that creates a released PDF; there is no `published` state or workflow.

## Work Guidance

- Material outcome change → edit CAP + proving tests + active CHG together.
- Do not churn CAPs for pure refactors with no user-visible change.
- Keep the Library surface and its wireframe folder-dominant. Follow CAP-0006's
  familiar Windows Explorer-like navigation model rather than reducing folders
  to a compact filter beside a document-first table.

## Verification

- Every `capabilities/CAP-*.md` appears in `README.md`. CAPs with a primary
  DMS Desktop surface link their HTML and PNG wireframes; headless CAPs state
  why no desktop wireframe applies.
- CAP status matches working tree reality (no false “implemented”).

## Child DOX Index

| Child | Owns | Read when editing… |
| --- | --- | --- |
| `wireframes/AGENTS.md` | CAP wireframe HTML/PNG assets and generator | Wireframe screens, exports, or CAP visual references |

Parent: `../AGENTS.md`. Progress sibling: `../changes/AGENTS.md`.
