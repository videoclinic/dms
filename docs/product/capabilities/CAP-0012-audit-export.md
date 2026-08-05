# CAP-0012 — Audit and report export

| Field | Value |
| --- | --- |
| ID | CAP-0012 |
| Status | not implemented |
| Storage | `<edit-root>/.dms/exports/` (operator-chosen) |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. An operator can generate an audit report covering the entire workspace or a
   filtered subset by document, approver, confidentiality type, and date
   range.
2. The report is a structured file (PDF for human reading, CSV for tooling)
   written to an operator-chosen path inside the edit root, default
   `<edit-root>/.dms/exports/`. The report contains the workflow history
   (events, hashes, timestamps, comments), the release history (version,
   relative path, confidentiality, checksum, approval-chain head), and the
   current classification summary.
   Periodic-review requests/results and approved major/minor change class are
   included for each applicable release cycle.
3. Reports do not embed draft Office content or released PDF bytes. They
   carry identifying metadata and SHA-256 digests so a separate copy of the
   document can be matched against a recorded checksum.
4. Generating a report is itself a workflow event of type `report_generated`
   recorded with the same canonical event body, including the report path,
   filter parameters, and the local OS user.
5. The **Verify workflow** routine from CAP-0011 is runnable as part of the
   export and produces a `valid` / `tampered` / `missing` verdict alongside
   the report contents.
6. A report is reproducible: re-running the same filter produces the same
   content as long as the underlying events have not changed. The export
   also records a SHA-256 over the report file so an external reader can
   confirm integrity.

## Non-goals

- Periodic scheduled exports (operator-triggered only in v1)
- Emailing reports to external recipients
- Compressing or encrypting reports

## Links

- Evidence: [`CAP-0011-approval-evidence.md`](CAP-0011-approval-evidence.md)
- Integrity: [`CAP-0004-release-integrity.md`](CAP-0004-release-integrity.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0013, ADR-0014: [`../../design-decisions.md`](../../design-decisions.md)
- Periodic review: [`CAP-0017-periodic-document-review.md`](CAP-0017-periodic-document-review.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
