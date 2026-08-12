# Repository agent skills

## Purpose

Own optional, versioned agent playbooks that help contributors apply this repository's contracts consistently.

## Ownership

| Path | Owns |
| --- | --- |
| `software-development/` | Product-record authoring and migration playbooks |

## Local Contracts

- A skill is an execution aid, not a source of product behaviour, progress, or architectural authority.
- Durable policy lives in root `AGENTS.md` and `docs/` contracts. Skills link to those sources instead of restating them.
- Skills follow the Agent Skills `SKILL.md` format and remain safe to read without a Hermes installation.
- Do not add credentials, profile-local paths, or agent-private state to repository skills.
- `software-development/application-records*` skills name `phased-plan-execution` as the only phase-sequencing engine for CHG-driven work; the `P`-prefixed slot, `Execution slot`, and `Depends on` discipline is enforced by that engine, not by this skill tree.

## Work Guidance

- Add a skill only when it provides a reusable, checkable procedure beyond the repository contracts.
- Keep repository-specific detail here; keep generic planning mechanics in shared profile skills such as `phased-plan-execution` and `phased-plan-overview`. Repository skills decide **what** each phase produces (CAP delta, CHG row update, behaviour-test evidence); the engine decides **when** a phase advances and commits.
- When a CHG closes, control returns to `phased-plan-overview`; do not let a repository skill route directly to the next CHG or a sibling file.
- When adding a skill category directory, give it an AGENTS.md if it has its own purpose or workflow; otherwise keep ownership at this level.

## Verification

- Read changed `SKILL.md` frontmatter (`name`, `description`, version) and completion criteria.
- Validate relative Markdown links from the repository root for paths that exist.
- When a package-level records check exists (for example `pnpm records:check`), run it from the repository root. No such command exists yet.

## Child DOX Index

| Child | Owns | Read when editing… |
| --- | --- | --- |
| `software-development/AGENTS.md` | Product-record workflow and migration skills | Capability/change-record workflows or their migration |

Parent contract: `../AGENTS.md`. Governing records: `../docs/product/README.md`, `../docs/changes/README.md`.
