# CAP-0004 — Released PDF checksum integrity

| Field | Value |
| --- | --- |
| ID | CAP-0004 |
| Status | not implemented |
| Algorithm | SHA-256 |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. On successful release, the app computes SHA-256 over the **versioned**
   released PDF file bytes under the publish root and stores the digest in
   `.dms` with that version’s release record.
2. Operator can run verify on a released version; result is `match`,
   `mismatch`, or `missing file` — never silent success on mismatch.
3. A later release creates a new versioned PDF and a new digest entry; prior
   version records remain readable and verifiable.
4. Checksum verification does not modify PDF bytes.
5. Workspace-level or per-document **verify all releases** is available as
   described in CAP-0016 and reports per-version outcomes without stopping at
   the first failure.

## Non-goals

- Encryption at rest
- Digital signatures / certificates (distinct from content checksums)
- Checksumming draft Office files as a release gate

## Links

- ADR-0005, ADR-0007: [`../../design-decisions.md`](../../design-decisions.md)
- Publish tree: [`CAP-0016-publish-tree-maintenance.md`](CAP-0016-publish-tree-maintenance.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
