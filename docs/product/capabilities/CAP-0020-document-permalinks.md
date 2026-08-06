# CAP-0020 — Document permalinks (stable local-app URI)

| Field | Value |
| --- | --- |
| ID | CAP-0020 |
| Status | not implemented |
| Identity keys | Stable workspace ID + stable document ID (ADR-0015) |
| Tests | none |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The desktop app registers a **local-app URI scheme** used for document
   permalinks and notification deep links. The scheme is stable across
   application versions for a given product build line.
2. A **document permalink** identifies a library document by **stable workspace
   ID** and **stable document ID** only. It does **not** embed draft file name,
   relative path, version label (`VMAJOR.MINOR`), publish PDF name, or absolute
   filesystem paths.
3. Activating a document permalink on a host where the app is installed and the
   workspace is **registered and accessible**:
   - brings the app to the foreground (or starts it)
   - opens that workspace
   - jumps to the document in the library navigator (selects it and shows the
     CAP-0006 selection pane)
   - opens or focuses the matching open-activity tab (CAP-0005)
4. Optional **target** segments refine the landing surface without changing
   identity keys, for example:
   - document home / selection (default)
   - open review request / decision UI (when a review-request ID is present)
   - notes for that document
   Targets never substitute path or version for the document ID.
5. If the workspace ID is unknown, not registered, or not currently accessible,
   the app reports that condition and does **not** open an arbitrary filesystem
   path or invent a workspace. If the document ID is unknown in an accessible
   matching workspace, the app reports not found and does not guess by file name.
6. After a draft **rename**, **move**, or **reassociate** of the locator
   (CAP-0013), or after any **version** bump on release, the same permalink URI
   still resolves to the same document ID. Display labels may change; the URI
   does not.
7. Unregistered (history-retained) and obsolete documents remain resolvable by
   ID when still present in `.dms`; the UI lands on the appropriate directory,
   maintenance, or history view rather than failing silently.
8. From a single-document CAP-0006 selection, the operator can **Copy
   permalink**. The clipboard receives the canonical URI for the default
   document target. Copy does not require network access.
9. Review-request and decision-outcome notifications (CAP-0010) embed a
   permalink whose identity keys are workspace ID + document ID and whose
   target is the addressed review request. Notification bodies still exclude
   document content (privacy rules).
10. The URI cannot record a workflow decision by itself. Opening a review target
    only navigates to the decision UI; approve/reject/request-changes remain
    explicit in-app actions (CAP-0002 / CAP-0011).
11. The app exposes the canonical URI form in operator-visible UI (copy control
    and, where helpful, a read-only field). Parsing accepts only the registered
    scheme and required identity query/path parts; unknown extra parameters are
    ignored without changing resolution of workspace and document IDs.

## Canonical form (contract)

Illustrative shape (exact scheme name fixed at implementation; must match the
registered handler):

```
dms://open?workspace=<workspace-id>&document=<document-id>
dms://open?workspace=<workspace-id>&document=<document-id>&target=review&review=<review-request-id>
dms://open?workspace=<workspace-id>&document=<document-id>&target=notes
```

Equivalent path-style forms are allowed only if they carry the same identity
keys and map 1:1 to this semantics. File name and version must never be
required parameters.

## Non-goals

- Public internet URLs or a hosted web portal
- Permalinks that encode absolute disk paths as the primary key
- Sharing a permalink as proof of approval or as a substitute for access control
- Resolving by document title, document number, or draft stem alone
- QR codes or short-link redirect services in v1

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0020-document-permalinks.html`](../wireframes/html/CAP-0020-document-permalinks.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0020-document-permalinks.png`](../wireframes/exports/CAP-0020-document-permalinks.png)

- Desktop shell / activity tabs: [`CAP-0005-desktop-shell.md`](CAP-0005-desktop-shell.md)
- Library navigator: [`CAP-0006-library-explorer.md`](CAP-0006-library-explorer.md)
- Notification deep links: [`CAP-0010-notification-transport.md`](CAP-0010-notification-transport.md)
- Stable IDs: [`CAP-0001-local-folder-dms.md`](CAP-0001-local-folder-dms.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0009, ADR-0015, ADR-0020: [`../../design-decisions.md`](../../design-decisions.md)
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
