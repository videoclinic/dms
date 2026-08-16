import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import {
  activityKey,
  applyNotesLibraryRestoration,
  applyPermalinkDocumentSelection,
  beginFormSubmission,
  closeActivity,
  closeWorkspaceSession,
  createInitialState,
  defaultPreferences,
  finishFormSubmission,
  lifecycleFailureLibraryState,
  lifecycleSuccessLibraryState,
  notesLibraryReturnTarget,
  openActivity,
  permalinkActivity,
  rememberRecentLibrary,
  removeRecentLibrary,
  savedViewId,
  setupMarkup,
  switchWorkspaceSession,
  toggleSavedView,
  workspaceSetupRequest,
} from "./app.mjs";

const workspaceId = "5ef3db10-8f6d-4ae4-9d68-ecb1eaac8235";

test("an active form submission suppresses duplicate IPC dispatch", () => {
  const form = {};
  const submitter = { disabled: false, isConnected: true };

  assert.equal(beginFormSubmission(form, submitter), true);
  assert.equal(submitter.disabled, true);
  assert.equal(beginFormSubmission(form, submitter), false);

  finishFormSubmission(form, submitter);
  assert.equal(submitter.disabled, false);
  assert.equal(beginFormSubmission(form, submitter), true);
  finishFormSubmission(form, submitter);
});

test("a failed decision consumes the one-time approver sign-in in frontend state", () => {
  const library = {
    detail_error: "",
    approver_sign_in: { actor: { tenant_id: "tenant-1", object_id: "person-1" } },
    lifecycle_drafts: {},
  };

  const failedDecision = lifecycleFailureLibraryState(
    library,
    "decide_review",
    "actor mismatch",
    { reason: "", confirmed: false },
  );
  assert.equal(failedDecision.approver_sign_in, null);
  assert.equal(failedDecision.detail_error, "actor mismatch");

  const failedRelease = lifecycleFailureLibraryState(
    library,
    "release_candidate",
    "export failed",
    { reason: "", confirmed: false },
  );
  assert.deepEqual(failedRelease.approver_sign_in, library.approver_sign_in);
});

test("a successful decision also clears the one-time approver sign-in", () => {
  const library = {
    ...createInitialState().library,
    approver_sign_in: { actor: { tenant_id: "tenant-1", object_id: "person-1" } },
  };
  const detail = { document_id: "doc-1", relative_path: "Policies/Handbook.md" };

  const decided = lifecycleSuccessLibraryState(library, "decide_review", detail);
  assert.equal(decided.approver_sign_in, null);

  const released = lifecycleSuccessLibraryState(library, "release_candidate", detail);
  assert.deepEqual(released.approver_sign_in, library.approver_sign_in);
});

function documentActivity(task) {
  return {
    workspace_id: workspaceId,
    destination: "Library",
    task,
    label: `${task} · Policy · DOC-014`,
    document_id: "80693979-420b-4766-9a86-2f6603cd52ab",
    route_state: {},
  };
}

test("workspace setup exposes existing-open and confirmed dual-root initialization", () => {
  const markup = setupMarkup("", ["/Users/name/DMS/Edit"]);

  assert.match(markup, /id="open-workspace-form"/);
  assert.match(markup, /id="initialize-workspace-form"/);
  assert.match(markup, /name="editRoot"/);
  assert.match(markup, /name="publishRoot"/);
  assert.match(markup, /name="confirmed"[^>]*required/);
  assert.match(markup, /name="takeOverStale"/);
  assert.match(markup, /name="overrideExisting"/);
  assert.match(markup, /another DMS instance may still be writing/i);
  assert.equal((markup.match(/data-directory-target=/g) ?? []).length, 3);
  assert.match(markup, /data-recent-library-open="\/Users\/name\/DMS\/Edit"/);
  assert.match(markup, /data-recent-library-remove="\/Users\/name\/DMS\/Edit"/);
});

test("recent-library open failures stay visible and retain the path for lock recovery", () => {
  const editRoot = "/Users/name/DMS/Edit";
  const markup = setupMarkup(
    "workspace lock is stale; explicit take-over is required",
    [editRoot],
    editRoot,
  );

  assert.ok(markup.indexOf('role="alert"') < markup.indexOf('class="setup-grid"'));
  assert.match(markup, /id="open-edit-root"[^>]*value="\/Users\/name\/DMS\/Edit"/);
});

test("workspace setup maps each form to its explicit desktop command", () => {
  assert.deepEqual(
    workspaceSetupRequest("open-workspace-form", { editRoot: " C:\\DMS\\Edit " }),
    {
      command: "open_workspace",
      arguments: { editRoot: "C:\\DMS\\Edit" },
      lockOptions: { takeOverStale: false, overrideExisting: false },
    },
  );
  assert.deepEqual(
    workspaceSetupRequest("open-workspace-form", {
      editRoot: "/Users/name/DMS/Edit",
      overrideExisting: "on",
    }).lockOptions,
    { takeOverStale: false, overrideExisting: true },
  );
  assert.deepEqual(
    workspaceSetupRequest("initialize-workspace-form", {
      editRoot: " /Users/name/DMS/Edit ",
      publishRoot: " /Users/name/DMS/Publish ",
      confirmed: "on",
    }),
    {
      command: "initialize_workspace",
      arguments: {
        editRoot: "/Users/name/DMS/Edit",
        publishRoot: "/Users/name/DMS/Publish",
        confirmed: true,
      },
    },
  );
});

test("clean desktop close releases the active workspace lock before destroying the window", async () => {
  const calls = [];
  let destroyed = false;
  const owner = { os_user: "operator", hostname: "host", process_id: 17, acquired_at: "now" };
  const closed = await closeWorkspaceSession(
    {
      workspace: { edit_root: "/DMS/Edit" },
      maintenance: { lock_status: { lock: owner } },
    },
    async (command, arguments_) => calls.push({ command, arguments_ }),
    async () => { destroyed = true; },
  );

  assert.equal(closed, true);
  assert.deepEqual(calls, [{
    command: "release_workspace_lock",
    arguments_: { editRoot: "/DMS/Edit", owner, confirmed: true },
  }]);
  assert.equal(destroyed, true);
});

test("main window permits the forced close used after advisory-lock release", () => {
  const capability = JSON.parse(readFileSync(new URL("../capabilities/default.json", import.meta.url)));

  assert.ok(capability.permissions.includes("core:window:allow-destroy"));
});

test("shell and Library panes contain scrolling without moving navigation", () => {
  const styles = readFileSync(new URL("./styles.css", import.meta.url), "utf8");

  assert.match(styles, /body\s*\{[^}]*overflow:\s*hidden/);
  assert.match(styles, /\.app\s*\{[^}]*height:\s*100vh[^}]*overflow:\s*hidden/);
  assert.match(styles, /\.workspace-shell\s*\{[^}]*min-height:\s*0[^}]*overflow:\s*hidden/);
  assert.match(styles, /\.main-content\s*\{[^}]*min-height:\s*0[^}]*overflow:\s*auto/);
  assert.match(styles, /#expanded-groups\s*\{[^}]*overflow-y:\s*auto/);
  assert.match(styles, /\.library-grid\s*\{[^}]*overflow:\s*hidden/);
  for (const selector of ["folder-tree", "table-scroll", "selection-pane"]) {
    assert.match(styles, new RegExp(`\\.${selector}\\s*\\{[^}]*overflow:\\s*auto`));
  }
});

test("opening a workspace acquires its lock and switches only after releasing the prior lock", async () => {
  const calls = [];
  const priorOwner = { process_id: 16 };
  const newOwner = { process_id: 17 };
  const status = { state: "current", stale_after_hours: 24, lock: newOwner };
  const result = await switchWorkspaceSession(
    { edit_root: "/DMS/Old" },
    { state: "current", stale_after_hours: 24, lock: priorOwner },
    { edit_root: "/DMS/New" },
    { takeOverStale: true, overrideExisting: true },
    async (command, arguments_) => {
      calls.push({ command, arguments_ });
      return command === "acquire_workspace_lock" ? status : null;
    },
  );

  assert.equal(result, status);
  assert.deepEqual(calls, [
    {
      command: "acquire_workspace_lock",
      arguments_: {
        editRoot: "/DMS/New",
        takeOverStale: true,
        overrideExisting: true,
      },
    },
    {
      command: "release_workspace_lock",
      arguments_: { editRoot: "/DMS/Old", owner: priorOwner, confirmed: true },
    },
  ]);
});

test("preferences start expanded and persist no session activities", () => {
  const state = createInitialState(defaultPreferences());

  assert.equal(state.preferences.sidebar_expanded, true);
  assert.deepEqual(state.preferences.saved_views, []);
  assert.deepEqual(state.preferences.recent_libraries, []);
  assert.deepEqual(state.activities, []);
});

test("recent libraries are unique, most-recent-first, capped at ten, and removable", () => {
  let preferences = {
    ...defaultPreferences(),
    recent_libraries: Array.from({ length: 10 }, (_, index) => `/libraries/${index}`),
  };

  preferences = rememberRecentLibrary(preferences, "/libraries/5");
  assert.deepEqual(preferences.recent_libraries, [
    "/libraries/5",
    "/libraries/0",
    "/libraries/1",
    "/libraries/2",
    "/libraries/3",
    "/libraries/4",
    "/libraries/6",
    "/libraries/7",
    "/libraries/8",
    "/libraries/9",
  ]);

  preferences = rememberRecentLibrary(preferences, "/libraries/new");
  assert.equal(preferences.recent_libraries.length, 10);
  assert.equal(preferences.recent_libraries[0], "/libraries/new");

  preferences = removeRecentLibrary(preferences, "/libraries/5");
  assert.equal(preferences.recent_libraries.includes("/libraries/5"), false);
  assert.equal(preferences.sidebar_expanded, true);
});

test("opening the same document task focuses one stable activity", () => {
  let state = createInitialState(defaultPreferences());
  state = openActivity(state, documentActivity("Audit"));
  state = openActivity(state, { ...documentActivity("Audit"), label: "Audit · Renamed policy · DOC-014" });

  assert.equal(state.activities.length, 1);
  assert.equal(state.activities[0].label, "Audit · Renamed policy · DOC-014");
  assert.equal(state.current_key, activityKey(documentActivity("Audit")));
});

test("permalink targets create stable document, review, and notes activities", () => {
  const resolution = {
    workspace: { workspace_id: workspaceId },
    document_id: "80693979-420b-4766-9a86-2f6603cd52ab",
    title: "Policy",
    document_number: "DOC-014",
    folder: "Policies/HR",
    target: "document",
    review_id: null,
  };
  const document = permalinkActivity(resolution);
  const notes = permalinkActivity({ ...resolution, target: "notes" });
  const review = permalinkActivity({
    ...resolution,
    target: "review",
    review_id: "5a6382c0-3078-4cf9-a64c-d194686bd1f6",
  });

  assert.equal(activityKey(document), `${workspaceId}:Library`);
  assert.equal(document.document_id, resolution.document_id);
  assert.deepEqual(document.route_state, { folder: "Policies/HR" });
  assert.equal(activityKey(notes), `${workspaceId}:Notes:document:${resolution.document_id}`);
  assert.deepEqual(notes.route_state, { folder: "Policies/HR" });
  assert.equal(activityKey(review), `${workspaceId}:Review:document:${resolution.document_id}`);
  assert.equal(review.route_state.review, "5a6382c0-3078-4cf9-a64c-d194686bd1f6");
});

test("notes return focuses the unchanged Library selection without duplicating its activity", () => {
  const libraryActivity = {
    workspace_id: workspaceId,
    destination: "Library",
    task: "Library",
    label: "Library · Policies/HR",
    document_id: null,
    route_state: { folder: "Policies/HR", sort: "title" },
  };
  const notesActivity = { ...documentActivity("Notes"), route_state: { folder: "Policies/HR" } };
  let state = openActivity(createInitialState(), libraryActivity);
  state = {
    ...state,
    library: {
      ...state.library,
      folder: {
        relative_path: "Policies/HR",
        entries: [{
          relative_path: "Policies/HR/Policy.md",
          membership: { in_library: { document_id: notesActivity.document_id } },
        }],
      },
      selection: ["Policies/HR/Policy.md"],
      detail: { document_id: notesActivity.document_id },
      query: "retained query",
      sort: "title",
      back: ["."],
    },
  };
  state = openActivity(state, notesActivity);
  const noteState = {
    compose_body: "Unsaved note",
    compose_author: "Raphael",
    editing_id: "note-1",
    editing_body: "Unsaved edit",
    delete_id: "note-2",
  };
  state = { ...state, note_documents: { [notesActivity.document_id]: noteState } };
  const originalLibrary = state.library;

  const target = notesLibraryReturnTarget(state, notesActivity);
  assert.equal(target.exact, true);
  state = openActivity(state, target.activity);

  assert.equal(state.activities.length, 2);
  assert.equal(state.current_key, activityKey(libraryActivity));
  assert.equal(state.library, originalLibrary);
  assert.equal(state.library.query, "retained query");
  assert.equal(state.library.sort, "title");
  assert.deepEqual(state.library.back, ["."]);
  assert.equal(state.note_documents[notesActivity.document_id], noteState);
});

test("notes return requires stable-ID restoration when the Library view changed or closed", () => {
  const libraryActivity = {
    workspace_id: workspaceId,
    destination: "Library",
    task: "Library",
    label: "Library · Policies/HR",
    document_id: null,
    route_state: { folder: "Policies/HR" },
  };
  const notesActivity = { ...documentActivity("Notes"), route_state: { folder: "Policies/HR" } };
  let state = openActivity(createInitialState(), libraryActivity);
  state = {
    ...state,
    library: {
      ...state.library,
      folder: { relative_path: "Policies/Finance", entries: [] },
      selection: [],
      detail: null,
    },
  };
  state = openActivity(state, notesActivity);

  assert.deepEqual(notesLibraryReturnTarget(state, notesActivity), {
    activity: { ...libraryActivity, key: activityKey(libraryActivity) },
    exact: false,
  });

  state = closeActivity(state, activityKey(libraryActivity));
  assert.deepEqual(notesLibraryReturnTarget(state, notesActivity), { activity: null, exact: false });
});

test("notes restoration follows a moved source and retains details when its row is missing", () => {
  const documentId = documentActivity("Notes").document_id;
  const libraryActivity = {
    workspace_id: workspaceId,
    destination: "Library",
    task: "Library",
    label: "Library · Policies/Old",
    document_id: null,
    route_state: { folder: "Policies/Old", sort: "title" },
  };
  const notesActivity = { ...documentActivity("Notes"), route_state: { folder: "Policies/Old" } };
  let state = openActivity(createInitialState(), libraryActivity);
  state = openActivity(state, notesActivity);

  state = applyNotesLibraryRestoration(
    state,
    notesActivity,
    state.activities.find((activity) => activity.task === "Library"),
    {
      document_id: documentId,
      folder: "Policies/Moved",
      relative_path: "Policies/Moved/Policy.md",
      source_exists: true,
    },
    {
      tree: [],
      folder: {
        relative_path: "Policies/Moved",
        entries: [{
          relative_path: "Policies/Moved/Policy.md",
          membership: { in_library: { document_id: documentId } },
        }],
      },
    },
  );

  assert.equal(state.activities.length, 2);
  assert.equal(state.activities.find((activity) => activity.task === "Library").label, "Library · Policies/Moved");
  assert.deepEqual(state.library.selection, ["Policies/Moved/Policy.md"]);
  assert.equal(state.library.detail.document_id, documentId);

  state = applyNotesLibraryRestoration(
    state,
    notesActivity,
    state.activities.find((activity) => activity.task === "Library"),
    {
      document_id: documentId,
      folder: "Policies/Moved",
      relative_path: "Policies/Moved/Policy.md",
      source_exists: false,
      source_state: "unregistered",
    },
    { tree: [], folder: { relative_path: "Policies/Moved", entries: [] } },
  );

  assert.equal(state.activities.length, 2);
  assert.deepEqual(state.library.selection, []);
  assert.equal(state.library.detail.document_id, documentId);
  assert.equal(state.library.detail.source_exists, false);
});

test("permalink document details survive a missing filesystem row", () => {
  const detail = {
    document_id: "80693979-420b-4766-9a86-2f6603cd52ab",
    source_state: "unregistered",
    control: { title: "Retained policy" },
  };
  const library = {
    folder: { entries: [] },
    selection: ["Policies/Old.md"],
    detail: null,
    detail_error: "stale",
  };

  const resolved = applyPermalinkDocumentSelection(library, detail);

  assert.deepEqual(resolved.selection, []);
  assert.equal(resolved.detail, detail);
  assert.equal(resolved.detail_error, "");
});

test("opening or focusing an activity preserves an unfolded sidebar overlay", () => {
  let state = {
    ...createInitialState({ sidebar_expanded: false, saved_views: [] }),
    sidebar_overlay: true,
    flyout: "activity",
  };
  state = openActivity(state, documentActivity("Audit"));

  assert.equal(state.sidebar_overlay, true);
  assert.equal(state.preferences.sidebar_expanded, false);
  assert.equal(state.flyout, null);
});

test("different document tasks remain separate", () => {
  let state = createInitialState(defaultPreferences());
  state = openActivity(state, documentActivity("Audit"));
  state = openActivity(state, documentActivity("Notes"));

  assert.equal(state.activities.length, 2);
  assert.notEqual(state.activities[0].key, state.activities[1].key);
});

test("library navigation updates one session pane in place", () => {
  let state = createInitialState(defaultPreferences());
  const library = {
    workspace_id: workspaceId,
    destination: "Library",
    task: "Library",
    label: "Library · /",
    document_id: null,
    route_state: { folder: "." },
  };
  state = openActivity(state, library);
  state = openActivity(state, {
    ...library,
    label: "Library · policies/HR",
    route_state: { folder: "policies/HR" },
  });

  assert.equal(state.activities.length, 1);
  assert.equal(state.activities[0].label, "Library · policies/HR");
  assert.deepEqual(state.activities[0].route_state, { folder: "policies/HR" });
});

test("configuration route changes update one stable activity", () => {
  const workspace = {
    workspace_id: workspaceId,
    destination: "Configuration",
    task: "Configuration",
    label: "Configuration · Workspace",
    document_id: null,
    route_state: { route: "workspace" },
  };
  const defaults = {
    ...workspace,
    label: "Configuration · Document defaults",
    route_state: { route: "document-defaults" },
  };
  const workflow = {
    ...workspace,
    label: "Configuration · Workflow",
    route_state: { route: "workflow" },
  };
  const notifications = {
    ...workspace,
    label: "Configuration · Notifications",
    route_state: { route: "notifications" },
  };

  const state = [workspace, defaults, workflow, notifications].reduce(openActivity, createInitialState());

  assert.equal(activityKey(workspace), activityKey(defaults));
  assert.equal(state.activities.length, 1);
  assert.equal(state.activities[0].route_state.route, "notifications");
});

test("closing the current activity keeps the app running with another or no pane", () => {
  let state = createInitialState(defaultPreferences());
  state = openActivity(state, documentActivity("Audit"));
  state = openActivity(state, documentActivity("Notes"));
  state = closeActivity(state, activityKey(documentActivity("Notes")));

  assert.equal(state.activities.length, 1);
  assert.equal(state.current_key, activityKey(documentActivity("Audit")));

  state = closeActivity(state, activityKey(documentActivity("Audit")));
  assert.equal(state.current_key, null);
});

test("saved views contain stable IDs and never copy open activities", () => {
  const activity = documentActivity("Review");
  const preferences = toggleSavedView(defaultPreferences(), activity);

  assert.equal(preferences.saved_views.length, 1);
  assert.equal(preferences.saved_views[0].workspace_id, workspaceId);
  assert.equal(preferences.saved_views[0].document_id, activity.document_id);
  assert.equal("activities" in preferences, false);

  const removed = toggleSavedView(preferences, activity);
  assert.deepEqual(removed.saved_views, []);
});

test("library saved views retain folder, sort, and stable document target without duplicating the activity", () => {
  const base = {
    workspace_id: workspaceId,
    destination: "Library",
    task: "Library",
    label: "Library · Policies/HR",
    document_id: "80693979-420b-4766-9a86-2f6603cd52ab",
    route_state: { folder: "Policies/HR", sort: "title" },
  };
  const second = {
    ...base,
    label: "Library · Policies/IT",
    document_id: null,
    route_state: { folder: "Policies/IT", sort: "name" },
  };

  assert.equal(activityKey(base), `${workspaceId}:Library`);
  assert.notEqual(savedViewId(base), savedViewId(second));
  const preferences = toggleSavedView(toggleSavedView(defaultPreferences(), base), second);
  assert.equal(preferences.saved_views.length, 2);
  assert.equal(preferences.saved_views[0].document_id, base.document_id);
  assert.deepEqual(preferences.saved_views[0].route_state, base.route_state);
});

test("Library folder activation and splitter controls stay direct and session-only", () => {
  const source = readFileSync(new URL("./app.mjs", import.meta.url), "utf8");
  const styles = readFileSync(new URL("./styles.css", import.meta.url), "utf8");

  assert.match(source, /if \(row\.dataset\.libraryKind === "folder"\) \{\s*void loadLibraryFolder\(row\.dataset\.libraryEntry\);\s*return true;/);
  assert.match(source, /row\?\.dataset\.libraryKind === "folder"/);
  assert.match(source, /libraryResize && event\.key === "Escape"/);
  assert.match(source, /grid\.clientWidth - tree\.offsetWidth - splitter\.offsetWidth - 360/);
  assert.doesNotMatch(source, /localStorage|persistLibraryDetailWidth/);
  assert.doesNotMatch(styles, /@font-face|\.woff2?|\.ttf/i);
});
