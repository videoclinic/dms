# Privacy

## Data classes

| Class | Where it lives | Notes |
| --- | --- | --- |
| Draft source documents (Office and Markdown) | Edit root (operator-controlled) | Content is business documentation; app does not upload it |
| Released versioned PDFs | Publish root (operator-controlled) | Final released artifacts (`*_VMAJOR.MINOR_<confidentiality-type-id>.pdf`); the path exposes the classification ID |
| DMS metadata, library membership, notes, approval state, checksums | `<edit-root>/.dms/` | Local only; may contain personal names in notes or approver fields |
| Microsoft Entra workflow group binding and display cache | `<edit-root>/.dms/` | Group object ID, referenced user object IDs, and cached display name/email for routing; no app-managed user accounts, tenant/client ID, or OAuth tokens |
| App-global Entra configuration | OS user app-config `global-settings.json` | Non-secret public-client ID and tenant ID shared by local libraries; non-empty `DMS_ENTRA_CLIENT_ID` / `DMS_ENTRA_TENANT_ID` process overrides are not persisted |
| Mutable document profile and workflow-role policies (title, owner reference, number, type, assigned editor/approver) | `<edit-root>/.dms/` | Local control metadata; not Office properties, Markdown front matter, or document body content. Owner/editor/approver references are tenant-scoped Entra object IDs; display name/email is refreshable and never participates in identity equality. Free-text pre-v12 owners remain display-only `legacy_owner_label` values and are never resolved by name or email. Literal `<owner>` / `<editor>` placeholders represent only a successful empty eligible-people result and carry no identity or authority |
| Candidate and release snapshots | `<edit-root>/.dms/` | Immutable requested/accepted profile, effective date, confidentiality, requester/editor/approver/owner display snapshots, and object IDs where recorded. Later profile or display-cache changes do not rewrite this evidence; pre-v12 omissions remain explicitly unrecorded |
| Mutable review schedule | `<edit-root>/.dms/` | Per-document interval, exemption reason, and next-review-due date derived from the current release's stored effective date; separate from both the mutable profile and immutable release snapshot |
| Confidentiality policy and effective document label | `<edit-root>/.dms/` | Local classification metadata; a label does not enforce access control |
| Approval-notification metadata | `<edit-root>/.dms/` | Requester/approver display name/email, approver Entra tenant/object ID on a decision, outcome, send time, and delivery-attempt result; no document content |
| Workspace root paths | Inside `.dms` | Absolute edit/publish paths on the operator machine |
| SMTP relay app password | OS credential store | Write-only Configuration input; never stored in `.dms`, app preferences, frontend state, IPC results, or errors |
| Microsoft Entra delegated-token cache | OS credential store | Interactive sign-in tokens for Microsoft Graph; never stored in `.dms` |
| Workspace advisory lock | `<edit-root>/.dms/lock` | Process id, hostname, timestamp; advisory only, never contains document content |
| Export/audit reports | `<edit-root>/.dms/exports/` (operator-chosen) | Aggregated lifecycle, approval, periodic-review, and release evidence; produced on demand. Pre-v12 release rows carry an explicit **unrecorded** date and **unresolved** owner rather than substituting the current mutable profile |
| Workspace backup archive | Operator-chosen path | Contains `.dms`, controlled source drafts, and released PDFs; not encrypted by the app |
| Optional Claude Desktop handoff | Clipboard and Claude Desktop conversation | Operator-previewed plain-text change excerpts and selected metadata; processing may leave the machine for Anthropic |
| App preferences | OS user config directory (when implemented) | Sidebar preference and saved-view targets (workspace/document IDs plus route state); no document content or workflow evidence |

## Processing principles

- No remote document store in the current architecture.
- Microsoft Graph calls resolve the workspace group's direct user members and
  verify a review-decision actor. They send no document bytes, source or publish
  paths, `.dms` metadata, or approval comments to Microsoft Graph.
- No telemetry that includes document content or paths unless a future ADR
  explicitly enables opt-in diagnostics.
- A review-request email contains only the DMS-managed document title,
  filesystem-derived edit-root-relative source path, requested action,
  requester display name, candidate target version, configured confidentiality
  label, and CAP-0020 review permalink (workspace ID + document ID + review
  target). It never attaches or uploads draft or released document content.
  Permalinks never put draft or PDF bytes in the URI.
- A decision-outcome email contains only the document display/relative path,
  decision outcome, configured confidentiality label, and local-app CAP-0020
  permalink to the review detail; it does not include document content or the
  decision comment.
- The configured confidentiality label, DMS-managed title, and candidate target
  version are rendered in the review-request subject. No generated body field
  may embed document content. The same rule and template apply to `mailto:`
  fallback drafts.
- Each released PDF filename includes its effective confidentiality type ID.
  Anyone who can list the publish tree can therefore see that classification;
  filesystem access remains the operator's responsibility.
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
- Permalinks: [`product/capabilities/CAP-0020-document-permalinks.md`](product/capabilities/CAP-0020-document-permalinks.md)
- Audit/export: [`product/capabilities/CAP-0012-audit-export.md`](product/capabilities/CAP-0012-audit-export.md)
- Library maintenance: [`product/capabilities/CAP-0013-library-maintenance.md`](product/capabilities/CAP-0013-library-maintenance.md)
- Document control data: [`product/capabilities/CAP-0015-document-control-data.md`](product/capabilities/CAP-0015-document-control-data.md)
- Workflow-role routing: [`product/capabilities/CAP-0019-inherited-workflow-role-routing.md`](product/capabilities/CAP-0019-inherited-workflow-role-routing.md)
- Publish tree: [`product/capabilities/CAP-0016-publish-tree-maintenance.md`](product/capabilities/CAP-0016-publish-tree-maintenance.md)
- Claude Desktop assistance: [`product/capabilities/CAP-0018-claude-desktop-change-assistance.md`](product/capabilities/CAP-0018-claude-desktop-change-assistance.md)
