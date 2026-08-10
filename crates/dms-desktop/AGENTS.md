# dms-desktop

## Purpose

Provide the Tauri 2 desktop adapter and local WebView shell for Windows and
macOS.

## Ownership

| Path | Owns |
| --- | --- |
| `src/` | Tauri startup, IPC commands, `dms-core` adapter, OS user preferences |
| `ui/app.mjs` | Static shell, session activities, saved-view persistence, and IPC orchestration |
| `ui/library.mjs` | Folder-first explorer state, markup, selection rules, search, and sorting |
| `ui/*.test.mjs` | Framework-free shell and Library interaction tests |
| `capabilities/` | Tauri window permissions |
| `icons/` | SVG source and derived PNG/Windows application icons |
| `tauri.conf.json` | Desktop window, local frontend, security, and bundle configuration |
| `tests/` | Desktop integration coverage when separate fixtures are needed |

## Local Contracts

- Call `dms-core` for workspace domain behaviour; do not duplicate its rules.
- Store sidebar and saved-view preferences in the OS user app-config directory,
  never under `<edit-root>/.dms`.
- Keep open activities in frontend session state only.
- Saved document targets use workspace ID + document ID, never source paths.
- Library navigation keeps one session activity while saved Library views retain
  edit-root-relative folder, sort, and at most one stable document target.
- The Library selection pane owns add/unregister/reassociate/permalink controls;
  file rows remain selection-only and preserve exact source names.
- Load only app-local frontend assets; do not add remote runtime dependencies.

## Work Guidance

- Keep the shell usable without a frontend package manager until a compiled UI
  framework provides a concrete benefit.
- Preserve accessible names for icon-only controls and elided activity labels.

## Verification

- `cargo test -p dms-desktop`
- `node --test crates/dms-desktop/ui/*.test.mjs`
- `DMS_DESKTOP_SMOKE=1 cargo run -p dms-desktop`
- `.github/workflows/desktop-platform-smoke.yml` on Windows and macOS

## Child DOX Index

No nested AGENTS.md. Parent: `../AGENTS.md`.
