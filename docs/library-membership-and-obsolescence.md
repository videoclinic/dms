# Library membership and obsolescence

**Mark obsolete** and **Unregister** are independent operations. One answers
whether a controlled document is still valid. The other answers whether a file
is an active library member. They are not substitutes.

Owning contracts stay in the CAPs. This page is the operator comparison.

## Two axes

| Axis | Persisted field | Values | Question |
| --- | --- | --- | --- |
| Library membership | `source_state` | `registered`, `unregistered` | Is this file in the active library? |
| Document control | `lifecycle` | `draft`, `in_review`, `approved`, `released`, `obsolete` | May this document still be reviewed or released? |

A registered obsolete document is still **In library**. An unregistered file
is **Not in library** even if its retained record still has a lifecycle value.

## Which action

- **Mark obsolete** — withdraw a library document from further use. It stays a
  controlled member so operators and auditors can still see that it is dead.
- **Unregister** — take a file out of the active library. Use this for a
  mistaken add, a source you will not reassociate, or a file that should stop
  participating in control until explicitly added again.

Do not unregister a still-valid document to “retire” it. Do not mark a file
obsolete merely to hide it from the library.

## Comparison

| | Mark obsolete | Unregister |
| --- | --- | --- |
| Owner | [CAP-0015](product/capabilities/CAP-0015-document-control-data.md) | [CAP-0006](product/capabilities/CAP-0006-library-explorer.md) |
| Changes | `lifecycle` → `obsolete` | `source_state` → `unregistered` |
| Library row | Stays **In library** with state `obsolete` | Becomes **Not in library** (supported draft) or disappears if Lost source |
| Folder `~N` | Not counted (only exact `draft`) | Not counted (not registered) |
| Review / release | Blocked | Not a library member, so lifecycle actions do not apply |
| Open content or periodic review | Cancels an active release candidate; does not close a periodic review by itself | Not a blocker and not cancelled |
| Source file | Left in place | Left in place |
| Released PDFs | Left on disk | Left on disk |
| ID, control data, notes, releases, checksums, workflow history | Kept on the live library record | Kept as a retained `.dms` record under the same ID |
| Document number | Still reserved | Still reserved |
| Evidence | Required reason; `document_obsoleted` event | No workflow event |
| Desktop confirmation | Required checkbox | None |
| Selection | Exactly one document | Single or homogeneous multi-select |
| Lost source | Unavailable (needs draft bytes) | Available |
| Other entry | Periodic review result **obsolete** ([CAP-0017](product/capabilities/CAP-0017-periodic-document-review.md)) | Reassociate-absorb of an already-registered target ([CAP-0013](product/capabilities/CAP-0013-library-maintenance.md)) |
| Desktop placement | **Revision cycle** | **Actions** |
| CLI | No dedicated command | `dms document unregister` |

Digest-driven Draft/Released reconciliation never walks an obsolete document
back to `draft` or `released` ([ADR-0016](design-decisions.md#adr-0016-digest-driven-post-release-draft-cycle)).

Confidentiality resolution and other library-member rules apply only to
registered documents at a resolvable source path. Unregistered files are
excluded.

## Re-adding an unregistered file

Adding the same in-root path again restores `source_state` to `registered` on
the **same document ID**. It does not create a new document and does not reset
lifecycle, control data, notes, releases, or history.

So obsolete then unregister, then add-back, returns an obsolete library
document. Unregister a live document and add it back, and it returns with the
lifecycle it had when it left.

## What neither action deletes

- The source draft
- Released PDFs
- Stable document ID
- Document control data, notes, candidates, releases, checksums
- Canonical workflow history
- The document-number reservation (uniqueness spans active, unregistered, and
  obsolete records)

Closing a document-scoped tab is not unregister. Withdrawing a release is not
obsolescence.

## Permalinks and history

Stable workspace + document IDs still name both kinds of record
([CAP-0020](product/capabilities/CAP-0020-document-permalinks.md), not
implemented). Resolution should land on the directory, maintenance, or history
surface rather than fail silently.

## Runtime notes

- Unregister does not confirm in the desktop Actions control and does not
  append a workflow event.
- `mark_obsolete` does not check `source_state`. The Library pane offers it
  only for a registered document whose source is present.

## Related

- [CAP-0002](product/capabilities/CAP-0002-document-lifecycle.md) — lifecycle
  states; release refuses `obsolete`
- [CAP-0003](product/capabilities/CAP-0003-document-notes.md) — notes survive
  both actions unless deleted
- [CAP-0006](product/capabilities/CAP-0006-library-explorer.md) — add /
  unregister / explorer membership
- [CAP-0011](product/capabilities/CAP-0011-approval-evidence.md) —
  `document_obsoleted`
- [CAP-0013](product/capabilities/CAP-0013-library-maintenance.md) — Lost
  source, reassociate, absorb-unregister
- [CAP-0015](product/capabilities/CAP-0015-document-control-data.md) —
  obsolescence, cancel review, control data
- [CAP-0017](product/capabilities/CAP-0017-periodic-document-review.md) —
  periodic-review **obsolete** result
- [ADR-0015](design-decisions.md#adr-0015-stable-document-id-plus-relative-path-locator),
  [ADR-0016](design-decisions.md#adr-0016-digest-driven-post-release-draft-cycle),
  [ADR-0026](design-decisions.md#adr-0026-membership-and-obsolescence-stay-distinct)
