# Wireframes

## Purpose

Own static capability wireframes (HTML + PNG exports) that illustrate CAP
contracts using the shadcn-admin 2.2.0 visual base.

## Ownership

| Path | Owns |
| --- | --- |
| `generate.mjs` | Generator for HTML screens + index/manifest |
| `html/CAP-*.html` | Per-capability wireframe pages |
| `exports/CAP-*.png` | Raster exports for CAP reference links |
| `index.html` | Index of all screens |
| `manifest.json` | Machine-readable screen inventory |

## Local Contracts

- One wireframe screen per CAP ID (`CAP-NNNN`).
- Visual base is shadcn-admin 2.2.0 (`../../../../rb/shadcn-admin-2.2.0`):
  sidebar shell, Inter, oklch token palette, badge/button/card/table patterns.
- Wireframes are design references, not runtime UI. CAPs remain the behaviour
  contract; PNGs/HTML do not claim implementation.
- Regenerate with `node generate.mjs`, then headless Chrome screenshots into
  `exports/`.

## Work Guidance

- Material CAP outcome change that alters a primary operator surface → update
  the matching screen in `generate.mjs`, regenerate HTML + PNG, keep CAP link.
- Do not embed secrets or real credentials in sample content.

## Verification

- `manifest.json` lists every `html/CAP-*.html` and `exports/CAP-*.png`.
- Every CAP under `../capabilities/` links its wireframe HTML and PNG.
- PNG count equals CAP count (20).

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
