import test from "node:test";
import assert from "node:assert/strict";

import {
  applyDocumentSelection,
  applyLibrarySnapshot,
  breadcrumbSegments,
  buildFolderTree,
  clampLibraryDetailWidth,
  createLibraryState,
  confidentialityUpdateRequest,
  documentControlUpdateRequest,
  documentReviewScheduleRequest,
  entryDocumentId,
  filterLibraryEntries,
  historyTarget,
  lifecycleActionRequest,
  libraryIcon,
  libraryMarkup,
  libraryOpenRequest,
  membershipKind,
  normalizeLibraryPath,
  paginateLibraryEntries,
  resizeLibraryDetailWidth,
  selectedEntries,
  sortLibraryEntries,
  toggleLibrarySelection,
  toggleLibraryVisibility,
  toggleTreeFolder,
} from "./library.mjs";

const file = (name, membership, document = null) => ({
  name,
  relative_path: `Policies/${name}`,
  kind: "file",
  membership,
  document,
});

function snapshot(path, entries = []) {
  return {
    tree: [
      { name: "Edit", relative_path: "." },
      { name: "Policies", relative_path: "Policies" },
    ],
    folder: {
      relative_path: path,
      parent: path === "." ? null : ".",
      entries,
    },
  };
}

test("library paths and breadcrumbs stay edit-root-relative and portable", () => {
  assert.equal(normalizeLibraryPath("\\Policies\\HR\\"), "Policies/HR");
  assert.deepEqual(breadcrumbSegments("Policies/HR", "Edit"), [
    { label: "Edit", path: "." },
    { label: "Policies", path: "Policies" },
    { label: "HR", path: "Policies/HR" },
  ]);
});

test("folder history supports push, back, and forward without creating extra library state", () => {
  let library = createLibraryState();
  library = applyLibrarySnapshot(library, snapshot("."), ".", "replace");
  library = applyLibrarySnapshot(library, snapshot("Policies"), "Policies", "push");
  library = applyLibrarySnapshot(library, snapshot("Policies/HR"), "Policies/HR", "push");
  assert.deepEqual(library.back, [".", "Policies"]);
  assert.equal(historyTarget(library, "back"), "Policies");

  library = applyLibrarySnapshot(library, snapshot("Policies"), "Policies", "back");
  assert.deepEqual(library.back, ["."]);
  assert.deepEqual(library.forward, ["Policies/HR"]);
  assert.equal(historyTarget(library, "forward"), "Policies/HR");
});

test("folder tree is hierarchical and keeps branch expansion independent from navigation", () => {
  const folders = [
    { name: "Edit", relative_path: "." },
    { name: "Policies", relative_path: "Policies" },
    { name: "HR", relative_path: "Policies/HR" },
    { name: "Archive", relative_path: "Policies/HR/Archive" },
    { name: "IT", relative_path: "Policies/IT" },
  ];
  const tree = buildFolderTree(folders);
  assert.deepEqual(tree.map((node) => node.path), ["."]);
  assert.deepEqual(tree[0].children.map((node) => node.path), ["Policies"]);
  assert.deepEqual(tree[0].children[0].children.map((node) => node.path), [
    "Policies/HR",
    "Policies/IT",
  ]);

  let library = { ...createLibraryState(), tree: folders };
  library = applyLibrarySnapshot(library, {
    tree: folders,
    folder: { relative_path: "Policies/HR", parent: "Policies", entries: [] },
  }, "Policies/HR");
  assert.deepEqual(library.expanded_folders, [".", "Policies", "Policies/HR"]);
  library = toggleTreeFolder(library, "Policies");
  assert.deepEqual(library.expanded_folders, [".", "Policies/HR"]);

  const markup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies/HR" } },
    { ...library, expanded_folders: [".", "Policies", "Policies/HR"] },
  );
  assert.match(markup, /role="tree"/);
  assert.match(markup, /role="group"/);
  assert.match(markup, /data-library-tree-toggle="Policies"[^>]*aria-expanded="true"/);
  assert.match(markup, /role="treeitem"[^>]*aria-current="page"/);
});

test("folder counters render identical escaped nonzero summaries in tree and list", () => {
  const folders = [
    { name: "Edit", relative_path: ".", counters: { draft_documents: 2, available_to_add: 1 } },
    { name: "Policies", relative_path: "Policies", counters: { draft_documents: 2, available_to_add: 1 } },
    { name: "HR <Ops>", relative_path: "Policies/HR", counters: { draft_documents: 2, available_to_add: 1 } },
  ];
  const library = {
    ...createLibraryState(),
    tree: folders,
    expanded_folders: [".", "Policies"],
    folder: {
      relative_path: "Policies",
      parent: ".",
      entries: [{
        name: "HR <Ops>",
        relative_path: "Policies/HR",
        kind: "folder",
        folder_counters: { draft_documents: 2, available_to_add: 1 },
      }],
    },
  };
  const markup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    library,
  );
  assert.equal((markup.match(/aria-label="2 draft documents">~2/g) ?? []).length, 4);
  assert.equal((markup.match(/aria-label="1 file available to add">\+1/g) ?? []).length, 4);
  assert.doesNotMatch(markup, /unsupported file/);
  assert.match(markup, /HR &lt;Ops&gt;/);
});

test("visibility controls filter files before paging and prune hidden selection without changing folders", () => {
  const entries = [
    { name: "HR", relative_path: "Policies/HR", kind: "folder" },
    file("Draft.md", { in_library: { document_id: "draft" } }, { id: "draft", lifecycle: "draft" }),
    file("Released.md", { in_library: { document_id: "released" } }, { id: "released", lifecycle: "released" }),
    file("Available.md", "not_in_library"),
    file("Unsupported.bin", "unsupported"),
    file("Unknown.bin", null),
  ];
  let library = { ...createLibraryState(), folder: snapshot("Policies", entries).folder };
  assert.deepEqual(
    [library.show_draft_documents, library.show_available_to_add, library.show_unsupported_files],
    [true, true, true],
  );
  assert.equal(filterLibraryEntries(entries, library).length, 6);
  library = { ...library, selection: ["Policies/Draft.md"], detail: { document_id: "draft" }, page: 3 };
  library = toggleLibraryVisibility(library, "show_draft_documents");
  assert.equal(library.page, 0);
  assert.deepEqual(library.selection, []);
  assert.equal(library.detail, null);
  assert.deepEqual(filterLibraryEntries(entries, library).map((entry) => entry.name), [
    "HR", "Released.md", "Available.md", "Unsupported.bin", "Unknown.bin",
  ]);
  library = toggleLibraryVisibility(library, "show_available_to_add");
  library = toggleLibraryVisibility(library, "show_unsupported_files");
  assert.deepEqual(filterLibraryEntries(entries, library).map((entry) => entry.name), [
    "HR", "Released.md", "Unknown.bin",
  ]);
  const page = paginateLibraryEntries(sortLibraryEntries(filterLibraryEntries(entries, library), "name"), 10, 0);
  assert.equal(page.total, 3);
});

test("detail width is bounded per session and inline SVG icons stay self-contained", () => {
  assert.equal(clampLibraryDetailWidth(120), 280);
  assert.equal(clampLibraryDetailWidth(900), 640);
  assert.equal(clampLibraryDetailWidth(600, 440), 440);
  assert.equal(resizeLibraryDetailWidth(420, 100, 60), 460);
  assert.equal(createLibraryState().detail_width, 420);
  for (const icon of ["folder", "file", "chevron_right", "chevron_down", "back", "forward", "up", "refresh"]) {
    assert.match(libraryIcon(icon), /^<svg class="library-icon"[^>]*aria-hidden="true"/);
  }
  assert.throws(() => libraryIcon("missing"), /Unknown Library icon/);
});

test("search results use the same visibility state instead of changing folder counters", () => {
  const folderEntry = file("Current.md", "not_in_library");
  const result = file("Result.md", { in_library: { document_id: "doc-result" } }, {
    id: "doc-result",
    lifecycle: "draft",
    control: { title: "Matched title", document_number: "DOC-7" },
  });
  const library = {
    ...createLibraryState(),
    show_draft_documents: false,
    query: "matched",
    results: [result, file("Released.md", { in_library: { document_id: "released" } }, {
      id: "released", lifecycle: "released", control: { title: "Released" },
    })],
    folder: snapshot("Policies", [folderEntry]).folder,
  };
  const markup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    library,
  );
  assert.doesNotMatch(markup, /Policies\/Result\.md/);
  assert.match(markup, /data-library-entry="Policies\/Released\.md"/);
  assert.doesNotMatch(markup, /Policies\/Current\.md/);
  assert.match(markup, /aria-pressed="false">Draft documents/);
  assert.match(markup, /data-library-splitter/);
  assert.match(markup, /class="selection-pane"[^>]*width:420px/);
});

test("multi-selection exposes homogeneous membership without losing exact identities", () => {
  const entries = [
    file("Handbook.md", "not_in_library"),
    file("Policy.docx", "not_in_library"),
  ];
  let library = { ...createLibraryState(), folder: snapshot("Policies", entries).folder };
  library = toggleLibrarySelection(library, "Policies/Handbook.md");
  library = toggleLibrarySelection(library, "Policies/Policy.docx", true);

  assert.deepEqual(selectedEntries(library).map((entry) => entry.name), ["Handbook.md", "Policy.docx"]);
  assert.ok(selectedEntries(library).every((entry) => membershipKind(entry) === "not_in_library"));
});

test("registered membership keeps the stable document ID and supports control-data sorting", () => {
  const handbook = file("Handbook.md", { in_library: { document_id: "doc-2" } }, {
    id: "doc-2",
    lifecycle: "draft",
    control: { title: "Zebra guide", document_number: "HR-2" },
  });
  const access = file("Access.md", { in_library: { document_id: "doc-1" } }, {
    id: "doc-1",
    lifecycle: "draft",
    control: { title: "Access policy", document_number: "IT-1" },
  });

  assert.equal(membershipKind(handbook), "in_library");
  assert.equal(entryDocumentId(handbook), "doc-2");
  assert.deepEqual(sortLibraryEntries([handbook, access], "title").map((entry) => entry.name), [
    "Access.md",
    "Handbook.md",
  ]);
});

test("filter results paginate after sorting with only supported page sizes", () => {
  const entries = Array.from(
    { length: 26 },
    (_, index) => file(`Policy-${String(index).padStart(2, "0")}.md`, "not_in_library"),
  );
  const first = paginateLibraryEntries(entries, 10, 0);
  const last = paginateLibraryEntries(entries, 10, 99);
  assert.equal(first.entries.length, 10);
  assert.equal(first.page_count, 3);
  assert.equal(last.page, 2);
  assert.equal(last.entries.length, 6);
  assert.equal(paginateLibraryEntries(entries, 12, 0).entries.length, 25);
});

test("mixed and unsupported selections expose no incompatible batch action", () => {
  const entries = [
    file("Draft.md", "not_in_library"),
    file("Diagram.png", "unsupported"),
  ];
  const library = {
    ...createLibraryState(),
    folder: snapshot("Policies", entries).folder,
    selection: entries.map((entry) => entry.relative_path),
  };
  const markup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    library,
  );
  assert.match(markup, /Mixed selections have no common action/);
  assert.doesNotMatch(markup, /data-library-add/);
  assert.doesNotMatch(markup, /data-library-unregister/);
});

test("library markup separates source Name from DMS Title and keeps actions in the selection pane", () => {
  const registered = file("Handbook.md", { in_library: { document_id: "doc-1" } }, {
    id: "doc-1",
    lifecycle: "draft",
    control: { title: "Employee handbook", document_number: "HR-001" },
  });
  const library = {
    ...createLibraryState(),
    tree: snapshot("Policies").tree,
    folder: snapshot("Policies", [registered]).folder,
    selection: ["Policies/Handbook.md"],
    detail: {
      document_id: "doc-1",
      source_name: "Handbook.md",
      relative_path: "Policies/Handbook.md",
      source_exists: true,
      source_state: "registered",
      lifecycle: "draft",
      control: {
        title: "Employee handbook",
        document_number: "HR-001",
        document_type: "procedure",
        owner: { kind: "entra", object_id: "owner-1", display_name: "Olivia Owner" },
      },
      current_owner: { kind: "entra", object_id: "owner-1", display_name: "Olivia Owner" },
      eligible_people: [
        { object_id: "owner-1", display_name: "Olivia Owner", email: "owner@example.test" },
        { object_id: "editor-1", display_name: "Eva Editor", email: "editor@example.test" },
      ],
      document_types: [{ id: "procedure", label: "Procedure", enabled: true }],
      confidentiality_types: [
        { id: "internal", label: "Internal", enabled: true },
        { id: "restricted", label: "Restricted", enabled: true },
      ],
      confidentiality_override: "restricted",
      effective_confidentiality: null,
      effective_workflow_roles: null,
      lifecycle_actions: {
        begin_revision: { available: false, reason: "Only a released document can begin a revision." },
        cancel_review: { available: false, reason: "Available only while a review is open." },
        mark_obsolete: { available: true, reason: null },
      },
      workflow_events: [{
        event_hash: "abc123",
        body: {
          event_id: "event-1",
          event_type: "document_control_data_changed",
          timestamp: "2026-08-11T10:00:00Z",
          predecessor_hash: null,
          operator_comment: "Safe <script>alert(1)</script>",
        },
      }],
      workflow_verification: "valid",
      current_release: {
        release_id: "release-1",
        version: "1.2",
        relative_pdf_path: "Policies/Handbook_V1.2_internal.pdf",
        pdf_exists: true,
        effective_date: "2026-08-11",
        profile: {
          title: "Employee handbook",
          document_number: "HR-001",
          document_type: "procedure",
          owner: { kind: "entra", object_id: "owner-1", display_name: "Olivia Owner" },
        },
      },
      review_schedule: {
        workspace_interval_months: 12,
        interval_months: 6,
        exemption_reason: null,
        next_due_date: "2027-02-11",
      },
      permalink: "dms://open?workspace=ws-1&document=doc-1",
    },
  };
  const markup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    library,
  );

  assert.match(markup, /<th>Name<\/th><th>Title<\/th>/);
  assert.match(markup, /Handbook\.md/);
  assert.match(markup, /Employee handbook/);
  assert.match(markup, /data-library-open-source/);
  assert.match(markup, /data-library-open-release/);
  assert.match(markup, /<option value="next_minor">Next minor<\/option><option value="next_major">/);
  assert.match(markup, /Current released PDF · V1\.2/);
  assert.match(markup, /id="library-document-control-form"/);
  assert.match(markup, /name="ownerObjectId" required/);
  assert.match(markup, /<option value="owner-1" selected>Olivia Owner/);
  assert.doesNotMatch(markup, /name="owner"/);
  assert.match(markup, /Immutable current release profile/);
  assert.match(markup, /2026-08-11/);
  assert.match(markup, /id="library-review-schedule-form"/);
  assert.match(markup, /Next review due: 2027-02-11/);
  assert.match(markup, /<option value="procedure" selected>Procedure<\/option>/);
  assert.match(markup, /id="library-confidentiality-form"/);
  assert.match(markup, /<option value="restricted" selected>Restricted<\/option>/);
  assert.match(markup, /data-library-lifecycle-action="begin_revision" disabled/);
  assert.match(markup, /Only a released document can begin a revision/);
  assert.match(markup, /data-library-lifecycle-form="mark_obsolete"/);
  assert.match(markup, /Canonical workflow evidence · valid/);
  assert.match(markup, /document control data changed/);
  assert.doesNotMatch(markup, /<script>alert/);
  library.detail.workflow_verification = { tampered_at: "event-1" };
  assert.match(
    libraryMarkup(
      { edit_root: "/srv/Edit", workspace_id: "ws-1" },
      { route_state: { folder: "Policies" } },
      library,
    ),
    /tampered at event-1/,
  );
  assert.match(markup, /data-library-open-notes/);
  assert.match(markup, /data-library-copy-permalink/);
  assert.match(markup, /data-library-unregister/);
  assert.match(markup, /id="library-reassociate-form"/);
  assert.doesNotMatch(markup, /overflow menu|hamburger/i);
});

test("library file actions map only to host-mediated document commands", () => {
  const detail = { document_id: "doc-1" };
  assert.deepEqual(libraryOpenRequest(detail, "source"), {
    command: "open_document_source",
    arguments: { documentId: "doc-1" },
  });
  assert.deepEqual(libraryOpenRequest(detail, "release"), {
    command: "open_current_release_pdf",
    arguments: { documentId: "doc-1" },
  });
  assert.throws(() => libraryOpenRequest(detail, "other"));
});

test("approver sign-in challenge opens the host browser without WebView navigation", () => {
  const registered = file("Handbook.md", { in_library: { document_id: "doc-1" } }, {
    id: "doc-1",
    lifecycle: "draft",
    control: { title: "Employee handbook", document_number: null },
  });
  const library = {
    ...createLibraryState(),
    tree: snapshot("Policies").tree,
    folder: snapshot("Policies", [registered]).folder,
    selection: ["Policies/Handbook.md"],
    approver_sign_in: {
      challenge: {
        challenge_id: "challenge-1",
        user_code: "ABCD-EFGH",
        verification_uri: "https://microsoft.com/devicelogin",
      },
    },
    detail: {
      document_id: "doc-1",
      lifecycle: "draft",
      control: { title: "Employee handbook", document_number: null },
      active_candidate: { status: "in_review", approval_required: true },
      eligible_people: [],
      lifecycle_actions: {},
      workflow_events: [],
      workflow_verification: "valid",
      document_types: [],
      confidentiality_types: [],
    },
  };

  const markup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    library,
  );
  assert.match(markup, /data-open-external="https:\/\/microsoft\.com\/devicelogin"/);
  assert.match(markup, /Open sign-in page/);
  assert.doesNotMatch(markup, /target="_blank"/);
});

test("failed approver challenge offers a same-surface restart", () => {
  const registered = file("Handbook.md", { in_library: { document_id: "doc-1" } }, {
    id: "doc-1",
    lifecycle: "draft",
    control: { title: "Employee handbook", document_number: null },
  });
  const library = {
    ...createLibraryState(),
    tree: snapshot("Policies").tree,
    folder: snapshot("Policies", [registered]).folder,
    selection: ["Policies/Handbook.md"],
    detail_error: "Microsoft Entra sign-in challenge is no longer available; start again",
    approver_sign_in: {
      challenge: {
        challenge_id: "challenge-1",
        user_code: "ABCD-EFGH",
        verification_uri: "https://microsoft.com/devicelogin",
      },
    },
    detail: {
      document_id: "doc-1",
      lifecycle: "draft",
      control: { title: "Employee handbook", document_number: null },
      active_candidate: { status: "in_review", approval_required: true },
      eligible_people: [],
      lifecycle_actions: {},
      workflow_events: [],
      workflow_verification: "valid",
      document_types: [],
      confidentiality_types: [],
    },
  };

  const failedMarkup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    library,
  );
  assert.match(failedMarkup, /Previous sign-in failed/);
  assert.match(failedMarkup, /data-library-approver-sign-in/);
  assert.match(failedMarkup, /Sign in again/);
  assert.doesNotMatch(failedMarkup, /Complete approver sign-in/);
  assert.doesNotMatch(failedMarkup, /data-library-approver-sign-in-complete/);
  assert.doesNotMatch(failedMarkup, /ABCD-EFGH/);

  const activeMarkup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    { ...library, detail_error: "" },
  );
  assert.doesNotMatch(activeMarkup, /Sign in again/);
  assert.doesNotMatch(activeMarkup, /Previous sign-in failed/);
});

test("document control and confidentiality forms map to narrow document commands", () => {
  const detail = { document_id: "doc-1" };
  const control = new FormData();
  control.set("title", " Employee handbook ");
  control.set("documentNumber", " HR-001 ");
  control.set("documentType", "procedure");
  control.set("ownerObjectId", " owner-1 ");
  assert.deepEqual(documentControlUpdateRequest(control, detail), {
    command: "update_document_control",
    arguments: {
      documentId: "doc-1",
      title: "Employee handbook",
      documentNumber: "HR-001",
      documentType: "procedure",
      ownerObjectId: "owner-1",
    },
  });
  control.delete("ownerObjectId");
  assert.throws(() => documentControlUpdateRequest(control, detail), /eligible Microsoft Entra owner/);

  const confidentiality = new FormData();
  confidentiality.set("confidentialityTypeId", " restricted ");
  assert.deepEqual(confidentialityUpdateRequest(confidentiality, detail), {
    command: "set_document_confidentiality",
    arguments: { documentId: "doc-1", confidentialityTypeId: "restricted" },
  });

  const schedule = new FormData();
  schedule.set("scheduleMode", "override");
  schedule.set("reviewIntervalMonths", "6");
  assert.deepEqual(documentReviewScheduleRequest(schedule, detail), {
    command: "update_document_review_schedule",
    arguments: {
      documentId: "doc-1",
      reviewIntervalMonths: 6,
      reviewExemptionReason: null,
    },
  });
  schedule.set("scheduleMode", "exempt");
  schedule.set("reviewExemptionReason", " Retired reference ");
  assert.deepEqual(documentReviewScheduleRequest(schedule, detail).arguments, {
    documentId: "doc-1",
    reviewIntervalMonths: null,
    reviewExemptionReason: "Retired reference",
  });
});

test("lifecycle actions require reasons and confirmations and map to narrow commands", () => {
  const detail = { document_id: "doc-1" };
  assert.deepEqual(lifecycleActionRequest("begin_revision", null, detail), {
    command: "begin_document_revision",
    arguments: { documentId: "doc-1" },
  });

  const values = new FormData();
  values.set("reason", " Superseded by the global policy ");
  assert.throws(() => lifecycleActionRequest("mark_obsolete", values, detail), /confirmation/);
  values.set("confirmed", "yes");
  assert.deepEqual(lifecycleActionRequest("mark_obsolete", values, detail), {
    command: "mark_document_obsolete",
    arguments: {
      documentId: "doc-1",
      reason: "Superseded by the global policy",
      confirmed: true,
    },
  });
  assert.equal(
    lifecycleActionRequest("cancel_review", values, detail).command,
    "cancel_document_review",
  );
  values.set("reason", "   ");
  assert.throws(() => lifecycleActionRequest("cancel_review", values, detail), /reason/);
});

test("external lifecycle forms map candidates, decisions, releases, and mail confirmations to narrow commands", () => {
  const detail = {
    document_id: "doc-1",
    retryable_decision_candidate: { id: "candidate-1" },
    retryable_minor_publication: { release_id: "release-1" },
  };
  const submit = new FormData();
  submit.set("targetMode", "manual");
  submit.set("manualMajor", "2");
  submit.set("manualMinor", "4");
  submit.set("requesterObjectId", "editor-1");
  submit.set("changelog", " Clarify escalation path ");
  submit.set("effectiveDate", "2026-08-11");
  submit.set("reviewOverrideReason", " Marker retained for review ");
  assert.deepEqual(lifecycleActionRequest("submit_candidate", submit, detail), {
    command: "submit_document_candidate",
    arguments: {
      input: {
        documentId: "doc-1",
        targetMode: "manual",
        manualMajor: 2,
        manualMinor: 4,
        changelog: "Clarify escalation path",
        effectiveDate: "2026-08-11",
        requesterObjectId: "editor-1",
        stagedOwnerObjectId: null,
        stagedEditorObjectId: null,
        reviewOverrideReason: "Marker retained for review",
        mailtoConfirmed: false,
      },
    },
  });

  const decision = new FormData();
  decision.set("decision", "approved");
  decision.set("comment", " Ready ");
  assert.deepEqual(lifecycleActionRequest("decide_review", decision, detail), {
    command: "decide_document_review",
    arguments: { documentId: "doc-1", decision: "approved", comment: "Ready", mailtoConfirmed: false },
  });

  const confirmation = new FormData();
  assert.throws(() => lifecycleActionRequest("retry_review_notification", confirmation, detail), /host mail/);
  confirmation.set("mailtoConfirmed", "yes");
  assert.deepEqual(lifecycleActionRequest("retry_decision_notification", confirmation, detail), {
    command: "retry_decision_notification",
    arguments: { documentId: "doc-1", candidateId: "candidate-1", mailtoConfirmed: true },
  });
  assert.deepEqual(lifecycleActionRequest("retry_minor_publication_notification", confirmation, detail), {
    command: "retry_minor_publication_notification",
    arguments: { documentId: "doc-1", releaseId: "release-1", mailtoConfirmed: true },
  });
});

test("successful empty identity state renders literal placeholders and blocks lifecycle transitions", () => {
  const registered = file("Handbook.md", { in_library: { document_id: "doc-1" } }, {
    id: "doc-1",
    lifecycle: "draft",
    control: { title: "Handbook" },
  });
  const base = {
    ...createLibraryState(),
    folder: snapshot("Policies", [registered]).folder,
    selection: ["Policies/Handbook.md"],
    detail: {
      document_id: "doc-1",
      source_name: "Handbook.md",
      relative_path: "Policies/Handbook.md",
      source_exists: true,
      source_state: "registered",
      lifecycle: "draft",
      control: { title: "Handbook", owner: { kind: "placeholder", label: "<owner>" } },
      current_owner: { kind: "placeholder", label: "<owner>" },
      effective_workflow_roles: { editor: { kind: "placeholder", label: "<editor>" }, approver: null },
      eligible_people: [],
      eligible_people_state: "successful_empty",
      requires_identity_handover: true,
      lifecycle_actions: {},
    },
  };
  const emptyMarkup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    base,
  );
  assert.match(emptyMarkup, /&lt;owner&gt;/);
  assert.match(emptyMarkup, /Requesting editor/);
  assert.match(emptyMarkup, /value="&lt;editor&gt;" readonly/);
  assert.match(emptyMarkup, /Candidate submission and release are blocked/);
  assert.match(emptyMarkup, /type="submit" disabled>Submit candidate/);

  const populatedMarkup = libraryMarkup(
    { edit_root: "/srv/Edit", workspace_id: "ws-1" },
    { route_state: { folder: "Policies" } },
    {
      ...base,
      detail: {
        ...base.detail,
        eligible_people_state: "populated",
        eligible_people: [
          { object_id: "owner-1", display_name: "Olivia Owner", email: "owner@example.test" },
          { object_id: "editor-1", display_name: "Eva Editor", email: "editor@example.test" },
        ],
      },
    },
  );
  assert.match(populatedMarkup, /Apply real identities with successful release/);
  assert.match(populatedMarkup, /name="stagedOwnerObjectId" required/);
  assert.match(populatedMarkup, /name="stagedEditorObjectId" required/);
  assert.match(populatedMarkup, /name="targetMode"><option value="next_minor">/);
});

test("updated document selection refreshes the detail and visible row in place", () => {
  const registered = file("Handbook.md", { in_library: { document_id: "doc-1" } }, {
    id: "doc-1",
    lifecycle: "draft",
    control: { title: "Handbook" },
  });
  const library = {
    ...createLibraryState(),
    folder: snapshot("Policies", [registered]).folder,
    results: [registered],
    detail_error: "Duplicate document number",
  };
  const detail = {
    document_id: "doc-1",
    lifecycle: "draft",
    control: { title: "Employee handbook", document_number: "HR-001" },
  };
  const updated = applyDocumentSelection(library, detail, true);
  assert.equal(updated.detail, detail);
  assert.equal(updated.detail_error, "");
  assert.equal(updated.evidence_open, true);
  assert.equal(updated.folder.entries[0].document.control.title, "Employee handbook");
  assert.equal(updated.results[0].document.control.document_number, "HR-001");
});
