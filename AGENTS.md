# DOX framework

- DOX is highly performant AGENTS.md hierarchy installed here
- Agent must follow DOX instructions across any edits

## Core Contract

- AGENTS.md files are binding work contracts for their subtrees
- Work products, source materials, instructions, records, assets, and durable docs must stay understandable from the nearest applicable AGENTS.md plus every parent AGENTS.md above it

## Read Before Editing

1. Read the root AGENTS.md
2. Identify every file or folder you expect to touch
3. Walk from the repository root to each target path
4. Read every AGENTS.md found along each route
5. If a parent AGENTS.md lists a child AGENTS.md whose scope contains the path, read that child and continue from there
6. Use the nearest AGENTS.md as the local contract and parent docs for repo-wide rules
7. If docs conflict, the closer doc controls local work details, but no child doc may weaken DOX

Do not rely on memory. Re-read the applicable DOX chain in the current session before editing.

## Update After Editing

Every meaningful change requires a DOX pass before the task is done.

Update the closest owning AGENTS.md when a change affects:

- purpose, scope, ownership, or responsibilities
- durable structure, contracts, workflows, or operating rules
- required inputs, outputs, permissions, constraints, side effects, or artifacts
- user preferences about behavior, communication, process, organization, or quality
- AGENTS.md creation, deletion, move, rename, or index contents

Update parent docs when parent-level structure, ownership, workflow, or child index changes. Update child docs when parent changes alter local rules. Remove stale or contradictory text immediately. Small edits that do not change behavior or contracts may leave docs unchanged, but the DOX pass still must happen.

## Hierarchy

- Root AGENTS.md is the DOX rail: project-wide instructions, global preferences, durable workflow rules, and the top-level Child DOX Index
- Child AGENTS.md files own domain-specific instructions and their own Child DOX Index
- Each parent explains what its direct children cover and what stays owned by the parent
- The closer a doc is to the work, the more specific and practical it must be

## Child Doc Shape

- Create a child AGENTS.md when a folder becomes a durable boundary with its own purpose, rules, responsibilities, workflow, materials, or quality standards
- Work Guidance must reflect the current standards of the project or user instructions; if there are no specific standards or instructions yet, leave it empty
- Verification must reflect an existing check; if no verification framework exists yet, leave it empty and update it when one exists

Default section order:
- Purpose
- Ownership
- Local Contracts
- Work Guidance
- Verification
- Child DOX Index

## Style

- Keep docs concise, current, and operational
- Document stable contracts, not diary entries
- Put broad rules in parent docs and concrete details in child docs
- Prefer direct bullets with explicit names
- Do not duplicate rules across many files unless each scope needs a local version
- Delete stale notes instead of explaining history
- Trim obvious statements, repeated rules, misplaced detail, and warnings for risks that no longer exist
- No historical breadcrumbs in code, comments, or docs — describe the current design only

## Closeout

1. Re-check changed paths against the DOX chain
2. Update nearest owning docs and any affected parents or children
3. Refresh every affected Child DOX Index
4. Remove stale or contradictory text
5. Run existing verification when relevant
6. Report any docs intentionally left unchanged and why

## User Preferences

When the user requests a durable behavior change, record it here or in the relevant child AGENTS.md.

Contract-level rules that bind every child doc:

- Prefer concise operational prose; lead with the change, not recap labels
- Prefer explicit markers (`@file:`, `@base64:`) over implicit path auto-detection
- Drop features rather than add baroque defensive machinery
- Treat chained create/update/delete instructions as one task unless interrupted
- Git-tracked plans reference only repo-local tracked plans; do not park execution findings in `/tmp` or profile-private artifacts
- Material behaviour work uses CAP + CHG records under `docs/` (see `skills/software-development/application-records`); the active CHG is the execution plan

Project-wide style and workflow preferences also live in user memory; this section holds only rules that every child contract must honor.

## Architectural decisions

- **Agent harness protected by tirith.sh.** Reading passwords or access tokens is prohibited. Extract variables from `.env` / config files without relaying their values; use environment variables by importing them for Bash execution. `***` in output is a tirith redaction marker, not a literal value — never "fix" it to a variable ref.
- **Product shape.** A shared `dms-core` Rust library, separate headless `dms` CLI, and Tauri 2 desktop shell target Windows and macOS. The product uses operator-maintained document control; dual roots (edit + publish) with mirrored relative trees; foldable left menu (hamburger when collapsed) with session-only open-activity panes/tabs plus per-user saved views; folder-dominant, Windows Explorer-like library workspace with a persistent relative tree, Back/Forward/Up + breadcrumbs, current-folder contents, exact source-file names, and a selection pane that separates filesystem-derived Source file identity from DMS-managed document control data and document/batch actions; stable document permalinks (`workspace ID` + `document ID`); Microsoft 365 workflow routing selects eligible people from one Microsoft Entra group per workspace without an app user roster and requires interactive Entra sign-in for review decisions; Office and Markdown source drafts; editors document changes and propose a minor, major, or validated manual target version for each later review; unapproved candidates never occupy a version but remain audit evidence; app-driven PDF export commits the approved target to checksummed `*_VMAJOR.MINOR_<confidentiality-type-id>.pdf` through installed Office for Office formats and a CommonMark HTML print shell plus native WebView PDF APIs for Markdown (export chrome from `.dms`); `<edit-root>/.dms` metadata with no application database. Detail: `docs/architecture.md` and `docs/design-decisions.md`.
- **Application records.** CAP files under `docs/product/` are behaviour contracts; CHG files under `docs/changes/` are progress authority. Code and tests prove behaviour.

## Codebase Knowledge Graph (codebase-memory-mcp)

This project uses codebase-memory-mcp to maintain a knowledge graph of the codebase.
ALWAYS prefer MCP graph tools over grep/glob/file-search for code discovery.

### Priority Order
1. `search_graph` — find functions, classes, routes, variables by pattern
2. `trace_path` — trace who calls a function or what it calls
3. `get_code_snippet` — read specific function/class source code
4. `query_graph` — run Cypher queries for complex patterns
5. `get_architecture` — high-level project summary

### When to fall back to grep/glob
- Searching for string literals, error messages, config values
- Searching non-code files (Dockerfiles, shell scripts, configs)
- When MCP tools return insufficient results
- When the project has no indexed source graph yet

### Examples
- Find a handler: `search_graph(name_pattern=".*OrderHandler.*")`
- Who calls it: `trace_path(function_name="OrderHandler", direction="inbound")`
- Read source: `get_code_snippet(qualified_name="pkg/orders.OrderHandler")`

## Child DOX Index

| Child | Owns | Read when editing… |
| --- | --- | --- |
| `docs/AGENTS.md` | Architecture, privacy, ADRs, CAP/CHG product records | Product behaviour, progress records, or design docs |
| `crates/AGENTS.md` | Rust workspace, shared core, and CLI contracts | Rust source, packages, or tests |
| `skills/AGENTS.md` | Repository-local agent playbooks under `skills/` | Skill authoring, skill layout, or playbook contracts |

### Cross-cutting (root-owned until a child boundary exists)

| Path / surface | Notes |
| --- | --- |
| Root `AGENTS.md` | DOX rail, user preferences, architectural decisions, this index |
| Root `CLAUDE.md` | Versioned Claude Code entry point; directs team agents to the DOX contract and shared safety/workflow rules |
| Root `README.md` | Public project overview and current concept state; it summarizes but does not replace CAP, CHG, or architecture records |
| Root `LICENSE` | MIT license for project source and documentation |
| `Cargo.toml`, `Cargo.lock`, `crates/` | Rust workspace: shared `dms-core`, standalone `dms` CLI, and Tauri `dms-desktop` adapter |
| `.github/workflows/desktop-platform-smoke.yml` | Rust 1.88 Windows/macOS workspace gate, desktop launch smoke, and native multi-page WebView PDF export smoke |

### Index scope

- Index durable local boundaries only.
- Do not create AGENTS.md for empty planned directories, infrastructure-only folders, or files that do not yet exist.
- Skill leaf directories that hold only a `SKILL.md` are owned by `skills/software-development/AGENTS.md`; they do not need their own AGENTS.md.
