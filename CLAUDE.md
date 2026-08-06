# Claude Code entry point

Claude Code joins this repository through the same project contract used by
other team agents. [`AGENTS.md`](AGENTS.md) is authoritative; read it before
making changes and follow every applicable nested `AGENTS.md` on the path to a
target file.

## Working contract

- Treat capability records (`docs/product/capabilities/CAP-*.md`) as current
  behaviour contracts and change records (`docs/changes/active/CHG-*.md`) as
  implementation-progress authority. For material behaviour work, update the
  affected CAP and active CHG with the implementation and its tests.
- Keep source-file identity, DMS-managed document-control data, and Office
  document properties distinct.
- Read relevant files, trace code before changing it, make targeted edits, and
  run the relevant verification before reporting completion.
- Complete the DOX closeout after meaningful edits: re-check the applicable
  contract chain and update owning `AGENTS.md` files when a durable structure,
  responsibility, or workflow changes.

## Safety and collaboration

- Do not read, print, commit, or alter secrets. Treat `.env` files and runtime
  credentials as private; `***` in tool output is redaction, not a literal
  value to repair.
- Do not commit, push, rewrite history, or change repository visibility unless
  the operator explicitly asks.
- Keep commits focused and use conventional commit messages. Confirm the
  working tree and staged diff immediately before committing.
- Preserve current working-tree changes that are outside the requested task.
- State uncertainty and blockers plainly; do not invent source, test results,
  or external-service outcomes.
