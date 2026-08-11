# CAP-0018 — Optional Claude Desktop change-comment assistance

| Field | Value |
| --- | --- |
| ID | CAP-0018 |
| Status | not implemented |
| Integration | Operator-mediated handoff to installed Claude Desktop |
| Tests | Partial phase-8/9h evidence: [`dms-core` assistance tests](../../../crates/dms-core/src/assistance.rs), [`dms-desktop` adapter tests](../../../crates/dms-desktop/src/lib.rs), and [desktop assistance UI tests](../../../crates/dms-desktop/ui/assistance.test.mjs) |

## Outcomes (contract — not yet true in runtime)

When implemented, the following must hold:

1. The core document lifecycle works without Claude Desktop. AI assistance is
   disabled by default, optional per workspace, and never required for review,
   approval, release, or periodic review.
2. **Evaluate changes with Claude** is available only when the application can
   locate Claude Desktop on the supported host OS and workspace policy permits
   the document's effective confidentiality type. Missing Claude Desktop shows
   an unavailable explanation and does not offer a cloud/API fallback.
3. The app first produces a deterministic local plain-text comparison between
   the current source draft and the latest released PDF. The release record's
   source-draft and PDF digests identify the compared revision. The app shows
   the operator the exact change excerpts and metadata that would be handed to
   Claude; raw source/PDF files, `.dms`,
   approver data, paths outside the document, and credentials are excluded.
4. Before every handoff, the app states that Claude Desktop is a local client
   but Claude model processing may send the displayed payload to Anthropic. The
   operator must explicitly confirm that handoff. Cancellation sends nothing.
5. The app launches Claude Desktop through normal OS application launching and
   copies a generated prompt to the clipboard. The operator pastes the prompt
   into Claude Desktop and pastes the response back into the DMS. The app does
   not claim a callable Claude Desktop API, delivery confirmation, or automatic
   response capture.
6. The prompt asks for two advisory outputs:
   - a suggested target-version mode (`minor version change`, `major version
     change`, or `manual version set`) under CAP-0002, including rationale;
   - a concise proposed changelog grounded only in the supplied differences.
7. Claude output is untrusted suggestion text. It cannot change lifecycle,
   choose a version, write `.dms`, approve a document, or bypass required
   fields. The editor reviews and edits any accepted text and explicitly
   chooses the target-version mode and candidate.
8. When an operator accepts all or part of a suggestion, only the accepted text
   is copied into the editable changelog field. The eventual workflow event
   records `assistance_used: true` and provider label `Claude Desktop`; prompts
   and rejected responses are not persisted by the DMS.
9. If deterministic text extraction is unsupported, incomplete, or exceeds the
   configured payload limit, the app explains the limitation and requires the
   operator to select/trim the excerpts. It never silently truncates text sent
   for evaluation.
10. The integration never reads Claude Desktop configuration, cookies, account
    data, conversation history, or credentials.

## Non-goals

- Treating Claude Desktop as a local/offline model
- Direct invocation or response automation through an undocumented interface
- Supplying an Anthropic API key or embedding an API client
- Letting AI decide approval, target-version mode, or version number
- Installing or configuring Claude Desktop for the operator
- A custom Claude Desktop MCP extension in v1

## Links
- Wireframe (HTML): [`../wireframes/html/CAP-0018-claude-desktop-change-assistance.html`](../wireframes/html/CAP-0018-claude-desktop-change-assistance.html)
- Wireframe (PNG): [`../wireframes/exports/CAP-0018-claude-desktop-change-assistance.png`](../wireframes/exports/CAP-0018-claude-desktop-change-assistance.png)

- Lifecycle and target version: [`CAP-0002-document-lifecycle.md`](CAP-0002-document-lifecycle.md)
- Privacy: [`../../privacy.md`](../../privacy.md)
- ADR-0018: [`../../design-decisions.md`](../../design-decisions.md)
- Anthropic local MCP documentation: <https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop>
- Progress: [`../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md`](../../changes/active/CHG-0001-tauri-local-dms-bootstrap.md)
