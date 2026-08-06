# Documentation

## Purpose

Own product behaviour contracts, change-progress records, architecture,
privacy, and design decisions for this repository.

## Ownership

| Path | Owns |
| --- | --- |
| `architecture.md` | Runtime shape, trust boundary, out-of-scope platform choices |
| `privacy.md` | Data classes and local-processing principles |
| `design-decisions.md` | Cross-cutting ADRs |
| `product/` | CAP index, capability contracts, wireframe references |
| `changes/` | CHG lifecycle (active/archive) |

## Local Contracts

- CAP files state current behaviour only. `Status: not implemented` means no
  runtime claim; target outcomes are contracts for active CHGs.
- CHG files are progress authority, not feature specs. Exactly one active CHG
  per material request.
- Code and tests prove CAP outcomes; do not mark a CAP implemented without
  linked executable evidence.
- Do not invent external ticket IDs; use `Direct operator request:` when none.

## Work Guidance

- Material behaviour change: update affected CAPs + active CHG in the same
  vertical slice as implementation (see `skills/software-development/application-records`).
- New cross-cutting fork: add ADR in `design-decisions.md` and adjust
  architecture/privacy when those surfaces change.
- When source packages land, add their AGENTS.md under the code tree; keep
  product truth here.

## Verification

- Indexes in `product/README.md` and `changes/README.md` list every CAP/CHG file.
- Relative links between docs resolve.
- When a `records:check` (or equivalent) script exists at repo root, run it.
  None exists yet.

## Child DOX Index

| Child | Owns | Read when editing… |
| --- | --- | --- |
| `product/AGENTS.md` | Capability contracts | CAP files or product index |
| `changes/AGENTS.md` | Change-progress records | CHG files or changes index |

Parent: `../AGENTS.md`.
