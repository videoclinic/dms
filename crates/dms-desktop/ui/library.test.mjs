import test from "node:test";
import assert from "node:assert/strict";

import {
  applyLibrarySnapshot,
  breadcrumbSegments,
  createLibraryState,
  entryDocumentId,
  historyTarget,
  libraryMarkup,
  membershipKind,
  normalizeLibraryPath,
  paginateLibraryEntries,
  selectedEntries,
  sortLibraryEntries,
  toggleLibrarySelection,
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
      lifecycle: "draft",
      control: { title: "Employee handbook", document_number: "HR-001" },
      effective_confidentiality: null,
      effective_workflow_roles: null,
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
  assert.match(markup, /data-library-copy-permalink/);
  assert.match(markup, /data-library-unregister/);
  assert.match(markup, /id="library-reassociate-form"/);
  assert.doesNotMatch(markup, /overflow menu|hamburger/i);
});
