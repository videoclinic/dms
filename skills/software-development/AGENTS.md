# Product-record development skills

## Purpose

Own optional agent workflows for maintaining and migrating this repository's product capability and change records.

## Ownership

| Path | Owns |
| --- | --- |
| `application-records/SKILL.md` | Normal material-change lifecycle for this repository |
| `application-records-migration/SKILL.md` | Audit-first migration from legacy documentation/plan structures |

## Local Contracts

- Both skills defer to `docs/product/README.md` for current-behaviour records and `docs/changes/README.md` for progress records.
- `application-records-migration` never treats legacy documentation, a plan checkbox, or tracker state as proof of implemented behaviour.
- Generic phase sequencing remains owned by shared `phased-plan-*` skills; these repository skills decide which records a material change must maintain.
- Leaf skill directories contain only `SKILL.md`; no nested AGENTS.md under each skill folder.

## Work Guidance

- Keep the two skills complementary: normal work uses `application-records`; structural adoption of a legacy documentation set uses `application-records-migration` first.
- When CAP/CHG layout changes, update skill links and this contract in the same change, and refresh parent indexes up to the root.

## Verification

- Read changed SKILL frontmatter and completion criteria.
- Validate Markdown links that target paths present in the tree.
- When `pnpm records:check` or an equivalent records gate exists at the repository root, run it after skill or record changes. No records gate exists yet.

## Child DOX Index

No child DOX documents.

Parent contract: `../AGENTS.md`. Governing records: `../../docs/product/README.md`, `../../docs/changes/README.md`.
