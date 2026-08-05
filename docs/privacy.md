# Privacy

## Data classes

| Class | Where it lives | Notes |
| --- | --- | --- |
| Draft Office documents | Edit root (operator-controlled) | Content is business documentation; app does not upload it |
| Released versioned PDFs | Publish root (operator-controlled) | Final published artifacts (`*_VMAJOR.MINOR.pdf`) |
| DMS metadata, library membership, notes, approval state, checksums | `<edit-root>/.dms/` | Local only; may contain personal names in notes or approver fields |
| Document master data (title, owner, number, type, review dates) | `<edit-root>/.dms/` | Local control metadata; not document body content |
| Confidentiality policy and effective document label | `<edit-root>/.dms/` | Local classification metadata; a label does not enforce access control |
| Approval-notification metadata | `<edit-root>/.dms/` | Approver display name/email, send time, and delivery-attempt result; no document content |
| Workspace root paths | Inside `.dms` | Absolute edit/publish paths on the operator machine |
| SMTP relay password | OS credential store (when implemented) | Never stored in `.dms`; relay settings contain no password |
| Workspace advisory lock | `<edit-root>/.dms/lock` | Process id, hostname, timestamp; advisory only, never contains document content |
| Export/audit reports | `<edit-root>/.dms/exports/` (operator-chosen) | Aggregated lifecycle, approval, periodic-review, and release evidence; produced on demand |
| Workspace backup archive | Operator-chosen path | Contains `.dms`, controlled Office drafts, and released PDFs; not encrypted by the app |
| Optional Claude Desktop handoff | Clipboard and Claude Desktop conversation | Operator-previewed plain-text change excerpts and selected metadata; processing may leave the machine for Anthropic |
| App preferences | OS user config directory (when implemented) | No document content |

## Processing principles

- No remote document store in the current architecture.
- No telemetry that includes document content or paths unless a future ADR
  explicitly enables opt-in diagnostics.
- Approval email contains only the document display/relative path, requested
  action, configured confidentiality label, and local-app deep link; it never
  attaches or uploads draft or released document content.
- The configured confidentiality label is rendered in the notification subject
  but no body field that could embed document content is generated. The same
  rule applies to `mailto:` fallback drafts.
- Audit/export reports contain approver display names, comments, and revision
  digests; they do not embed draft or released document bytes.
- Claude Desktop assistance is disabled by default and allowed only for
  operator-selected confidentiality types. Every handoff previews the exact
  payload and requires confirmation; the desktop client is not represented as
  local/offline model processing.
- Checksums are cryptographic digests of released PDF bytes; they are not
  encryption and do not protect confidentiality by themselves.
- Operator is responsible for filesystem ACLs, backups, and organizational
  retention under ISO 27001 / company policy on both roots.
- Operator-initiated full workspace backups contain controlled content from
  both edit and publish roots plus `.dms`; the chosen backup location requires
  equivalent organizational access controls. The app adds no encryption or
  expiry.

## Related

- Architecture: [`architecture.md`](architecture.md)
- Decisions: [`design-decisions.md`](design-decisions.md)
- Releases: [`architecture.md`](architecture.md)
- Audit/export: [`product/capabilities/CAP-0012-audit-export.md`](product/capabilities/CAP-0012-audit-export.md)
- Library maintenance: [`product/capabilities/CAP-0013-library-maintenance.md`](product/capabilities/CAP-0013-library-maintenance.md)
- Master data: [`product/capabilities/CAP-0015-document-master-data.md`](product/capabilities/CAP-0015-document-master-data.md)
- Publish tree: [`product/capabilities/CAP-0016-publish-tree-maintenance.md`](product/capabilities/CAP-0016-publish-tree-maintenance.md)
- Claude Desktop assistance: [`product/capabilities/CAP-0018-claude-desktop-change-assistance.md`](product/capabilities/CAP-0018-claude-desktop-change-assistance.md)
