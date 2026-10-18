import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import {
  activityKey,
  activityMarkup,
  applyLibrarySort,
  applyNotesLibraryRestoration,
  applyPermalinkDocumentSelection,
  beginFormSubmission,
  bookmarkActivity,
  closeActivity,
  closeWorkspaceSession,
  createInitialState,
  currentDeepLinks,
  defaultPreferences,
  executeLibraryBatch,
  executeLibrarySessionAuthorizationPoll,
  executePublishIndexAction,
  finishFormSubmission,
  finishLibrarySessionActivation,
  lifecycleFailureLibraryState,
  lifecycleSuccessLibraryState,
  selectionFeedbackLibraryState,
  notesLibraryReturnTarget,
  openActivity,
  permalinkActivity,
  refreshLibrarySnapshot,
  rememberRecentLibrary,
  removeRecentLibrary,
  resetLibraryTableLayout,
  savedViewId,
  setupMarkup,
  librarySwitcherMarkup,
  openLibrarySwitcher,
  dismissLibrarySwitcher,
  isAdvisoryLockRefusal,
  libraryLockOwnerFeedback,
  switchWorkspaceSession,
  pollWorkspaceSessionAuthorization,
  toggleSavedView,
  workspaceSetupRequest,
  applyStartupAuthorization,
  applyLibrarySessionAuthorization,
  applyLibrarySessionPollFailure,
  applyLibrarySessionIdentityRecovery,
  cancelStartupAuthorizationPoll,
  dismissLibrarySessionAuthorization,
  identitySourcePollDelayMs,
  librarySessionAuthorizationMarkup,
  librarySessionAuthorizationPollDelayMs,
  librarySessionHeading,
  scheduleStartupAuthorizationPoll,
  shouldPollLibrarySessionAuthorization,
  shouldPollIdentitySource,
  shouldPollStartupAuthorization,
  startupAuthorizationMarkup,
  startupAuthorizationPollDelayMs,
  workspaceFootMarkup,
  workspaceActivationState,
  PublishIndexRefreshError,
} from "./app.mjs";

const workspaceId = "5ef3db10-8f6d-4ae4-9d68-ecb1eaac8235";

function batchHarness(action = "add") {
  const entries = ["Policy.md", "Guide.docx"].map((name, index) => ({
    name, kind: "file", relative_path: `Policies/${name}`,
    membership: action === "add" ? "not_in_library" : {
      [index ? "lost_source" : "in_library"]: { document_id: `doc-${index}` },
    },
    content: "SYNTHETIC SOURCE CONTENT",
    credential: "SYNTHETIC CREDENTIAL",
  }));
  const activity = {
    workspace_id: workspaceId, destination: "Library", task: "Library",
    label: "Library · Policies", route_state: { folder: "Policies" },
  };
  let state = createInitialState();
  state = openActivity({
    ...state,
    workspace: { workspace_id: workspaceId, edit_root: "/private/Edit" },
    library: {
      ...state.library,
      folder: { relative_path: "Policies", parent: ".", entries },
      selection: entries.map((entry) => entry.relative_path),
    },
  }, activity);
  const calls = [];
  const renders = [];
  return {
    activity, entries, calls, renders,
    getState: () => state,
    setState: (next) => {
      state = next;
      renders.push(activityMarkup(state, state.activities.find((item) => item.key === state.current_key)));
    },
    invoke: async (command, arguments_) => {
      calls.push({ command, arguments_ });
      if (command === "open_workspace") return state.workspace;
      if (command === "load_library") return { tree: [], folder: state.library.folder };
      return [];
    },
  };
}

test("batch add captures identities, dispatches once, refreshes, and reuses one unsaveable activity", async () => {
  const harness = batchHarness();
  const initial = harness.getState();
  const pending = executeLibraryBatch("add", harness);
  assert.equal(harness.getState().library, initial.library);
  assert.equal(harness.getState().current_key, `${workspaceId}:Batch execution`);
  harness.entries[0].name = "Changed after dispatch.md";
  await pending;
  assert.deepEqual(harness.calls, [
    { command: "add_library_documents", arguments_: { editRoot: "/private/Edit", paths: ["Policies/Policy.md", "Policies/Guide.docx"] } },
    { command: "open_workspace", arguments_: { editRoot: "/private/Edit" } },
    { command: "load_library", arguments_: { editRoot: "/private/Edit", folder: "Policies" } },
  ]);
  const result = harness.getState();
  assert.deepEqual(result.batch_execution.entries.map((entry) => entry.type), ["success"]);
  assert.equal(result.batch_execution.entries[0].action, "Add 2 documents to library");
  assert.deepEqual(result.batch_execution.entries[0].targets, [
    { name: "Policy.md", document_id: null }, { name: "Guide.docx", document_id: null },
  ]);
  assert.deepEqual(result.library.selection, []);
  assert.equal(result.current_key, `${workspaceId}:Batch execution`);
  assert.equal(bookmarkActivity(result), null);
  assert.equal(toggleSavedView(result.preferences, result.activities.at(-1)), result.preferences);
  assert.deepEqual(result.preferences, initial.preferences);
  assert.doesNotMatch(harness.renders.join(""), /SYNTHETIC SOURCE CONTENT|SYNTHETIC CREDENTIAL|\/private\/Edit/);
  assert.match(harness.renders.at(-1), /Batch execution messages/);
  harness.setState(openActivity({ ...result, library: initial.library }, harness.activity));
  await executeLibraryBatch("add", harness);
  assert.equal(harness.getState().activities.filter((item) => item.task === "Batch execution").length, 1);
  assert.deepEqual(harness.getState().batch_execution.entries.map((entry) => entry.id), [1, 2]);
  assert.equal(activityKey({ workspace_id: workspaceId, task: "Batch execution", document_id: "ignored", route_state: { folder: "Other" } }), `${workspaceId}:Batch execution`);
});

test("unregister renders its preservation warning before dispatch and logs one operation success", async () => {
  const harness = batchHarness("unregister");
  harness.getState().batch_execution.filter = "success";
  const invoke = harness.invoke;
  harness.invoke = async (command, arguments_) => {
    if (command === "unregister_library_documents") {
      assert.deepEqual(harness.getState().batch_execution.entries.map((entry) => entry.type), ["warning"]);
      assert.equal(harness.getState().batch_execution.filter, "all");
      assert.match(harness.renders.at(-1), /does not delete source files, stable IDs, document control data, workflow\/release history, or checksums/);
      assert.deepEqual(arguments_.documentIds, ["doc-0", "doc-1"]);
      harness.entries[0].membership.in_library.document_id = "changed-after-dispatch";
    }
    return invoke(command, arguments_);
  };
  await executeLibraryBatch("unregister", harness);
  assert.deepEqual(harness.getState().batch_execution.entries.map((entry) => entry.type), ["warning", "success"]);
  assert.equal(harness.getState().batch_execution.entries[1].targets[0].document_id, "doc-0");
  assert.equal(harness.calls.filter((call) => call.command === "unregister_library_documents").length, 1);
  assert.equal(harness.calls.length, 3);
});

test("rejected batches retain selection, emit no success or refresh, and never expose raw failures", async () => {
  for (const action of ["add", "unregister"]) {
    const harness = batchHarness(action);
    const initial = harness.getState();
    harness.invoke = async (command) => {
      harness.calls.push(command);
      throw new Error("/private/Edit/Policy.md C:\\Private\\Edit SYNTHETIC SOURCE CONTENT SYNTHETIC CREDENTIAL");
    };
    await executeLibraryBatch(action, harness);
    assert.deepEqual(harness.calls, [action === "add" ? "add_library_documents" : "unregister_library_documents"]);
    assert.equal(harness.getState().library, initial.library);
    assert.deepEqual(harness.getState().batch_execution.entries.map((entry) => entry.type), action === "add" ? ["error"] : ["warning", "error"]);
    const markup = harness.renders.at(-1);
    assert.match(markup, /success was not confirmed/);
    assert.doesNotMatch(markup, /SYNTHETIC|\/private\/Edit|C:|complete selected batch succeeded/);
    assert.equal(harness.getState().error, "");
  }
});

test("inapplicable batch actions never open an activity or dispatch IPC", async () => {
  for (const variant of ["mixed", "unsupported", "folder", "stale", "single", "wrong-action", "unknown-action", "no-workspace"]) {
    const harness = batchHarness();
    const initial = harness.getState();
    if (variant === "mixed") harness.entries[1].membership = { in_library: { document_id: "doc-x" } };
    if (variant === "unsupported") harness.entries[1].membership = "unsupported";
    if (variant === "folder") harness.entries[1].kind = "folder";
    if (variant === "stale") initial.library.selection.push("Missing.md");
    if (variant === "single") initial.library.selection.pop();
    if (variant === "no-workspace") initial.workspace = null;
    const action = variant === "wrong-action" ? "unregister" : variant === "unknown-action" ? "verify" : "add";
    assert.equal(await executeLibraryBatch(action, harness), false, variant);
    assert.equal(harness.getState(), initial);
    assert.deepEqual(harness.calls, []);
    assert.deepEqual(harness.renders, []);
  }
});

test("activity close discards only the batch log, while workspace activation discards the entire session", async () => {
  const harness = batchHarness();
  await executeLibraryBatch("add", harness);
  const logged = harness.getState();
  const libraryClosed = closeActivity(logged, activityKey(harness.activity));
  assert.equal(libraryClosed.batch_execution, logged.batch_execution);
  const batchClosed = closeActivity(logged, `${workspaceId}:Batch execution`);
  assert.deepEqual(batchClosed.batch_execution.entries, []);
  assert.equal(batchClosed.batch_execution.filter, "all");
  assert.equal(batchClosed.library, logged.library);
  assert.notEqual(batchClosed.batch_execution.session, logged.batch_execution.session);
  const reopened = openActivity(batchClosed, logged.activities.at(-1));
  assert.deepEqual(reopened.batch_execution.entries, []);
  for (const id of [workspaceId, "other-workspace"]) {
    const activated = workspaceActivationState(logged, {
      workspace: { workspace_id: id, edit_root: "/other/Edit" }, lock_status: { lock: {} },
    });
    assert.deepEqual(activated.batch_execution.entries, []);
    assert.deepEqual(activated.activities, []);
    assert.notEqual(activated.workspace_session, logged.workspace_session);
  }
  assert.deepEqual(createInitialState(logged.preferences).batch_execution.entries, []);
});

test("closing the workspace clears batch state before destroying the window, even on lock-release failure", async () => {
  const harness = batchHarness();
  await executeLibraryBatch("add", harness);
  const state = harness.getState();
  const session = state.workspace_session;
  state.maintenance.lock_status = { lock: { process_id: 1 } };
  let destroyed = false;
  await assert.rejects(closeWorkspaceSession(state, async (command) => {
    if (command === "release_workspace_lock") throw new Error("release failed");
    return { status: "unchanged" };
  }, async () => {
    assert.deepEqual(state.batch_execution.entries, []);
    assert.notEqual(state.workspace_session, session);
    destroyed = true;
  }), /release failed/);
  assert.equal(destroyed, true);
});

test("late batch results cannot repopulate a closed activity or switched workspace", async () => {
  for (const boundary of ["activity", "workspace", "close"]) {
    for (const succeeds of [true, false]) {
      const harness = batchHarness();
      let settle;
      const invoke = harness.invoke;
      harness.invoke = (command, arguments_) => command === "add_library_documents"
        ? new Promise((resolve, reject) => { settle = () => succeeds ? resolve([]) : reject(new Error("rejected")); })
        : invoke(command, arguments_);
      const pending = executeLibraryBatch("add", harness);
      if (boundary === "activity") {
        harness.setState(closeActivity(harness.getState(), `${workspaceId}:Batch execution`));
      } else if (boundary === "workspace") {
        harness.setState(workspaceActivationState(harness.getState(), {
          workspace: { workspace_id: "new", edit_root: "/new/Edit" }, lock_status: null,
        }));
      } else {
        await closeWorkspaceSession(harness.getState(), async () => {}, async () => {});
      }
      settle();
      await pending;
      assert.deepEqual(harness.getState().batch_execution.entries, []);
      assert.equal(harness.calls.length, boundary === "activity" && succeeds ? 2 : 0);
      if (boundary === "workspace") assert.equal(harness.getState().workspace.workspace_id, "new");
    }
  }
});

test("refresh failure remains distinct from a successful mutation and leaks no backend error", async () => {
  const harness = batchHarness();
  const invoke = harness.invoke;
  harness.invoke = (command, arguments_) => {
    if (command === "open_workspace") throw new Error("/private/Edit SYNTHETIC CREDENTIAL");
    return invoke(command, arguments_);
  };
  await executeLibraryBatch("add", harness);
  assert.deepEqual(harness.getState().batch_execution.entries.map((entry) => entry.type), ["success", "error"]);
  assert.match(harness.renders.at(-1), /batch succeeded, but the Library could not refresh/);
  assert.doesNotMatch(harness.renders.at(-1), /SYNTHETIC CREDENTIAL|\/private\/Edit/);
});

test("batch controls and modifier selection are wired into the existing Library click path", () => {
  const application = readFileSync(new URL("./app.mjs", import.meta.url), "utf8");
  assert.match(application, /toggleLibrarySelection\(\s+appState\.library,\s+row\.dataset\.libraryEntry,\s+event\.ctrlKey \|\| event\.metaKey,/);
  assert.match(application, /dataset\.libraryBatch;\s+if \(batchAction\) \{\s+await executeLibraryBatch\(batchAction/);
  assert.match(application, /batchExecutionClick\(appState\.batch_execution, event\.target\)/);
  assert.match(application, /if \(batch === appState\.batch_execution\) return false/);
  assert.match(application, /bookmark\.hidden = !bookmarkTarget/);
});

test("a refresh completing after workspace activation cannot restore the old workspace or Library", async () => {
  const harness = batchHarness();
  let resolveSnapshot;
  let startedRefresh;
  const refreshing = new Promise((resolve) => { startedRefresh = resolve; });
  const invoke = harness.invoke;
  harness.invoke = (command, arguments_) => {
    if (command === "load_library") {
      startedRefresh();
      return new Promise((resolve) => { resolveSnapshot = resolve; });
    }
    return invoke(command, arguments_);
  };
  const pending = executeLibraryBatch("add", harness);
  await refreshing;
  const next = workspaceActivationState(harness.getState(), {
    workspace: { workspace_id: "new", edit_root: "/new/Edit" }, lock_status: null,
  });
  harness.setState(next);
  resolveSnapshot({ tree: [], folder: { relative_path: "Old", entries: [] } });
  await pending;
  assert.equal(harness.getState(), next);
  assert.deepEqual(harness.getState().batch_execution.entries, []);
});

test("application module does not import a classic deep-link IIFE", () => {
  const shell = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  const application = readFileSync(new URL("./app.mjs", import.meta.url), "utf8");

  assert.doesNotMatch(shell, /tauri-plugin-deep-link-api-iife/);
  assert.doesNotMatch(application, /tauri-plugin-deep-link-api-iife/);
});

test("startup deep links use the app-owned command through core IPC", async () => {
  const calls = [];
  const urls = await currentDeepLinks(async (command, arguments_) => {
    calls.push({ command, arguments_ });
    return ["dms://open?workspace=5ef3db10-8f6d-4ae4-9d68-ecb1eaac8235"];
  });

  assert.deepEqual(calls, [{ command: "startup_deep_links", arguments_: {} }]);
  assert.deepEqual(urls, ["dms://open?workspace=5ef3db10-8f6d-4ae4-9d68-ecb1eaac8235"]);
});

test("startup deep-link failures leave an empty list instead of throwing", async () => {
  await assert.doesNotReject(async () => {
    const urls = await currentDeepLinks(async () => {
      throw new Error("plugin unavailable");
    });
    assert.deepEqual(urls, []);
  });
});

test("first paint is not gated on deep-link registration", () => {
  const application = readFileSync(new URL("./app.mjs", import.meta.url), "utf8");
  const start = application.slice(application.indexOf("async function start()"));
  const renderIndex = start.indexOf("\n  render(appState);");
  const registerIndex = start.indexOf("registerDeepLinkHandler();");
  assert.ok(renderIndex > 0 && registerIndex > renderIndex);
});

test("live deep-link events use the core event listener", () => {
  const application = readFileSync(new URL("./app.mjs", import.meta.url), "utf8");
  assert.match(application, /__TAURI__\?\.event\?\.listen/);
  assert.match(application, /deep-link:\/\/new-url/);
});

test("Library Refresh opens the workspace and replaces one current-folder snapshot", async () => {
  const calls = [];
  const workspace = { workspace_id: workspaceId, edit_root: "/DMS/Edit" };
  const snapshot = {
    tree: [{ name: "Edit", relative_path: "." }, { name: "New", relative_path: "New" }],
    folder: { relative_path: "Policies", parent: ".", entries: [{ name: "Added.md" }] },
  };

  const refreshed = await refreshLibrarySnapshot(
    async (command, arguments_) => {
      calls.push({ command, arguments_ });
      return command === "open_workspace" ? workspace : snapshot;
    },
    "/DMS/Edit",
    "Policies",
  );

  assert.deepEqual(calls, [
    { command: "open_workspace", arguments_: { editRoot: "/DMS/Edit" } },
    { command: "load_library", arguments_: { editRoot: "/DMS/Edit", folder: "Policies" } },
  ]);
  assert.deepEqual(refreshed, { workspace, snapshot });
});

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
  assert.equal(failedDecision.selection_open.revision, true);

  const failedRelease = lifecycleFailureLibraryState(
    library,
    "release_candidate",
    "export failed",
    { reason: "", confirmed: false },
  );
  assert.deepEqual(failedRelease.approver_sign_in, library.approver_sign_in);
});

test("selected-document feedback opens its target section", () => {
  const library = createInitialState().library;
  for (const section of ["control", "schedule", "revision", "actions"]) {
    const updated = selectionFeedbackLibraryState(library, section, "configuration field document type cannot be empty");
    assert.equal(updated.detail_error, "configuration field document type cannot be empty");
    assert.equal(updated.lifecycle_notice, "");
    assert.equal(updated.selection_open[section], true);
  }
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

test("submit_candidate success writes a revision-cycle notice naming the candidate outcome", () => {
  const library = createInitialState().library;

  const approvedRequired = lifecycleSuccessLibraryState(
    library,
    "submit_candidate",
    {
      document_id: "doc-1",
      active_candidate: {
        approval_required: true,
        status: "in_review",
        version: { major: 1, minor: 0 },
      },
    },
  );
  assert.equal(
    approvedRequired.lifecycle_notice,
    "Release candidate V1.0 created and submitted for approval. The document is now Pending approval.",
  );
  assert.equal(approvedRequired.selection_open.revision, true);

  const deliveryFailed = lifecycleSuccessLibraryState(
    library,
    "submit_candidate",
    {
      document_id: "doc-1",
      active_candidate: {
        approval_required: true,
        status: "review_delivery_failed",
        version: { major: 1, minor: 0 },
      },
    },
  );
  assert.match(deliveryFailed.lifecycle_notice, /Confirm review request delivery/);

  const minor = lifecycleSuccessLibraryState(
    library,
    "submit_candidate",
    {
      document_id: "doc-1",
      active_candidate: {
        approval_required: false,
        status: "draft",
        version: { major: 1, minor: 4 },
      },
    },
  );
  assert.match(minor.lifecycle_notice, /export and release/);

  const decided = lifecycleSuccessLibraryState(
    library,
    "decide_review",
    { document_id: "doc-1", active_candidate: { status: "approved" } },
  );
  assert.equal(decided.lifecycle_notice, "");
});

test("lifecycleFailureLibraryState clears any prior success notice", () => {
  const library = {
    ...createInitialState().library,
    lifecycle_notice: "Release candidate V1.0 created and submitted for approval.",
  };
  const failed = lifecycleFailureLibraryState(
    library,
    "submit_candidate",
    "actor mismatch",
    { reason: "", confirmed: false },
  );
  assert.equal(failed.lifecycle_notice, "");
  assert.equal(failed.detail_error, "actor mismatch");
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

  assert.ok(markup.indexOf('role="alert"') < markup.indexOf('id="recent-libraries-heading"'));
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

test("clean desktop close refreshes before releasing the active lock and destroying the window", async () => {
  for (const status of ["written", "unchanged"]) {
    const calls = [];
    const owner = { os_user: "operator", hostname: "host", process_id: 17, acquired_at: "now" };
    const closed = await closeWorkspaceSession(
      {
        workspace: { edit_root: "/DMS/Edit" },
        maintenance: { lock_status: { lock: owner } },
      },
      async (command, arguments_) => {
        calls.push({ command, arguments_ });
        return { status };
      },
      async () => { calls.push({ command: "destroy" }); },
    );

    assert.equal(closed, true);
    assert.deepEqual(calls, [{
      command: "refresh_publish_index",
      arguments_: { editRoot: "/DMS/Edit" },
    }, {
      command: "release_workspace_lock",
      arguments_: { editRoot: "/DMS/Edit", owner, confirmed: true },
    }, { command: "destroy" }]);
  }
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
  for (const selector of ["folder-tree", "table-scroll"]) {
    assert.match(styles, new RegExp(`\\.${selector}\\s*\\{[^}]*overflow:\\s*auto`));
  }
  assert.match(
    styles,
    /\.selection-pane\s*\{[^}]*display:\s*flex[^}]*flex-direction:\s*column[^}]*overflow:\s*hidden/,
  );
  assert.doesNotMatch(styles, /\.selection-pane\s*\{[^}]*overflow:\s*auto/);
  assert.match(styles, /\.selection-scroll\s*\{[^}]*flex:\s*1 1 auto[^}]*overflow:\s*auto/);
  assert.match(styles, /\.selection-actions-footer\s*\{[^}]*min-height:\s*2\.75rem/);
  assert.match(styles, /\.selection-actions-footer\[open\]\s*\{[^}]*flex:\s*0 0 auto/);
  assert.doesNotMatch(styles, /\.selection-actions-footer[^{]*\{[^}]*max-height:\s*50%/);
  assert.doesNotMatch(styles, /\.selection-actions-footer\[open\][^{]*\{[^}]*max-height:\s*(?:50%|18rem)/);
  assert.doesNotMatch(styles, /\.selection-actions-footer\s*\{[^}]*position:\s*(?:sticky|fixed)/);
  assert.match(
    styles,
    /\.selection-actions-footer[^{]*\{[^}]*overflow:\s*hidden/,
  );
  assert.match(
    styles,
    /\.selection-actions-footer[^{]*>\s*summary\s*\{[^}]*flex:\s*0 0 auto/,
  );
  assert.match(
    styles,
    /\.selection-actions-footer[^{]*\.selection-section-body\s*\{[^}]*overflow:\s*auto/,
  );
});

test("opening a workspace revalidates it before acquiring its lock and switches only after releasing the prior lock", async () => {
  const calls = [];
  const priorOwner = { process_id: 16 };
  const newOwner = { process_id: 17 };
  const status = { state: "current", stale_after_hours: 24, lock: newOwner };
  const activation = {
    workspace: { edit_root: "/DMS/New" },
    lock_status: status,
  };
  const result = await switchWorkspaceSession(
    { edit_root: "/DMS/Old" },
    { state: "current", stale_after_hours: 24, lock: priorOwner },
    { edit_root: "/DMS/New" },
    { takeOverStale: true, overrideExisting: true },
    async (command, arguments_) => {
      calls.push({ command, arguments_ });
      return command === "activate_workspace_session" ? activation : null;
    },
  );

  assert.equal(result.kind, "activated");
  assert.equal(result.activation, activation);
  assert.deepEqual(calls, [
    { command: "refresh_publish_index", arguments_: { editRoot: "/DMS/Old" } },
    {
      command: "activate_workspace_session",
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

test("a failed bound open does not acquire a destination lock", async () => {
  const calls = [];
  await assert.rejects(
    () => switchWorkspaceSession(
      { edit_root: "/DMS/Old" },
      { state: "current", stale_after_hours: 24, lock: { process_id: 16 } },
      { edit_root: "/DMS/New" },
      {},
      async (command, arguments_) => {
        calls.push({ command, arguments_ });
        if (command === "activate_workspace_session") {
          throw new Error("the signed-in Microsoft Entra user is not an enabled direct member of this library group");
        }
        return null;
      },
    ),
    /enabled direct member/,
  );
  assert.deepEqual(calls, [
    { command: "refresh_publish_index", arguments_: { editRoot: "/DMS/Old" } },
    {
      command: "activate_workspace_session",
      arguments_: { editRoot: "/DMS/New", takeOverStale: false, overrideExisting: false },
    },
  ]);
});

test("an authorization-required workspace open returns its sign-in state without a lock handoff", async () => {
  const calls = [];
  const authorization = {
    kind: "pending",
    edit_root: "/DMS/New",
    library_label: "New",
    group_label: "Quality workflow",
    user_code: "ABCD-EFGH",
    verification_uri: "https://microsoft.com/devicelogin",
  };

  const result = await switchWorkspaceSession(
    { edit_root: "/DMS/Old" },
    { state: "current", stale_after_hours: 24, lock: { process_id: 16 } },
    { edit_root: "/DMS/New" },
    {},
    async (command, arguments_) => {
      calls.push({ command, arguments_ });
      return { kind: "authorization_required", authorization };
    },
  );

  assert.deepEqual(result, { kind: "authorization_required", authorization });
  assert.deepEqual(calls, [{
    command: "refresh_publish_index",
    arguments_: { editRoot: "/DMS/Old" },
  }, {
    command: "activate_workspace_session",
    arguments_: { editRoot: "/DMS/New", takeOverStale: false, overrideExisting: false },
  }]);
});

test("same-library reuse performs no lock handoff", async () => {
  const calls = [];
  const result = await switchWorkspaceSession(
    { edit_root: "/DMS/Edit" },
    { state: "current", lock: { process_id: 16 } },
    { edit_root: "/DMS/Edit" },
    {},
    async (command, arguments_) => {
      calls.push({ command, arguments_ });
      return null;
    },
  );
  assert.equal(result, null);
  assert.deepEqual(calls, []);
});

test("a failed prior-lock release rolls back the destination lock", async () => {
  const calls = [];
  const priorOwner = { process_id: 16 };
  const newOwner = { process_id: 17 };
  await assert.rejects(
    () => switchWorkspaceSession(
      { edit_root: "/DMS/Old" },
      { state: "current", lock: priorOwner },
      { edit_root: "/DMS/New" },
      {},
      async (command, arguments_) => {
        calls.push({ command, arguments_ });
        if (command === "activate_workspace_session") {
          return {
            workspace: { edit_root: "/DMS/New" },
            lock_status: { state: "current", lock: newOwner },
          };
        }
        if (command === "release_workspace_lock" && arguments_.editRoot === "/DMS/Old") {
          throw new Error("workspace lock owner changed before release");
        }
        return null;
      },
    ),
    /owner changed/,
  );
  assert.deepEqual(calls, [
    { command: "refresh_publish_index", arguments_: { editRoot: "/DMS/Old" } },
    {
      command: "activate_workspace_session",
      arguments_: { editRoot: "/DMS/New", takeOverStale: false, overrideExisting: false },
    },
    {
      command: "release_workspace_lock",
      arguments_: { editRoot: "/DMS/Old", owner: priorOwner, confirmed: true },
    },
    {
      command: "release_workspace_lock",
      arguments_: { editRoot: "/DMS/New", owner: newOwner, confirmed: true },
    },
  ]);
});

test("publish index actions target only the active workspace and retain release history state", async () => {
  for (const [action, status, notice] of [
    ["update", "written", "Released document index updated."],
    ["update", "unchanged", "Released document index is already current."],
    ["open", null, "Released document index opened."],
  ]) {
    let state = createInitialState();
    state.workspace = { workspace_id: workspaceId, edit_root: "/DMS/Active" };
    state.session_error = "Previous close refresh failed";
    state.releases = { ...state.releases, query: "Policy", page: 2, page_size: 10, rows: [{ release_id: "retained" }] };
    const initial = state;
    const calls = [];
    assert.equal(await executePublishIndexAction(action, {
      getState: () => state,
      setState: (next) => { state = next; },
      invoke: async (command, arguments_) => {
        calls.push({ command, arguments_ });
        assert.equal(state.releases.index_busy, true);
        return status ? { status } : null;
      },
    }), true);
    assert.deepEqual(calls, [{ command: action === "update" ? "refresh_publish_index" : "open_publish_index", arguments_: { editRoot: "/DMS/Active" } }]);
    assert.equal(state.releases.index_notice, notice);
    assert.equal(state.session_error, action === "update" ? "" : initial.session_error);
    assert.equal(state.releases.index_error, "");
    assert.equal(state.releases.index_busy, false);
    assert.equal(state.releases.publish_index_available, true);
    assert.equal(state.releases.rows, initial.releases.rows);
    for (const key of ["query", "page", "page_size"]) assert.equal(state.releases[key], initial.releases[key]);
    assert.equal(state.workspace, initial.workspace);
    assert.equal(state.activities, initial.activities);
    assert.equal(state.maintenance, initial.maintenance);
  }
});

test("publish index actions require an active workspace and suppress duplicate dispatch", async () => {
  for (const variant of ["no-workspace", "busy", "unknown-action"]) {
    const state = createInitialState();
    if (variant !== "no-workspace") state.workspace = { edit_root: "/DMS/Active" };
    state.releases.index_busy = variant === "busy";
    assert.equal(await executePublishIndexAction(variant === "unknown-action" ? "delete" : "update", {
      getState: () => state,
      setState: () => assert.fail("no state change expected"),
      invoke: () => assert.fail("no command expected"),
    }), false);
  }
});

test("publish index controls and close failure feedback are wired into the shell", () => {
  const source = readFileSync(new URL("./app.mjs", import.meta.url), "utf8");
  const shell = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  assert.match(source, /closest\("\[data-publish-index-update\]"\)/);
  assert.match(source, /closest\("\[data-publish-index-open\]"\)/);
  assert.match(source, /await executePublishIndexAction\(indexAction,/);
  assert.match(source, /event\.preventDefault\(\);\s*try\s*\{\s*await closeWorkspaceSession/);
  assert.match(shell, /id="workspace-session-error"[^>]*role="alert"/);
  assert.match(source, /sessionError\.textContent = state\.session_error/);
});

test("publish index errors replace stale success notices without resetting history or session", async () => {
  for (const action of ["update", "open"]) {
    let state = createInitialState();
    state.workspace = { edit_root: "/DMS/Active" };
    state.releases = {
      ...state.releases,
      query: "Policy",
      page: 3,
      index_notice: "Previous success",
      publish_index_available: true,
    };
    const initial = state;
    const error = action === "open"
      ? "no owned publish index exists yet; refresh the publish index first"
      : "publish index at /Publish/index.html is not a regular DMS-owned file; move it aside before refreshing";
    assert.equal(await executePublishIndexAction(action, {
      getState: () => state,
      setState: (next) => { state = next; },
      invoke: async () => { throw new Error(error); },
    }), false);
    assert.equal(state.releases.index_notice, "");
    assert.equal(state.releases.index_error, `Error: ${error}`);
    assert.equal(state.releases.index_busy, false);
    assert.equal(state.releases.publish_index_available, false);
    assert.equal(state.releases.query, "Policy");
    assert.equal(state.releases.page, 3);
    assert.equal(state.activities, initial.activities);
    assert.equal(state.workspace_session, initial.workspace_session);
  }
});

test("publish index ownership failures clear availability for update and open", async () => {
  for (const action of ["update", "open"]) {
    for (const message of [
      "no owned publish index exists yet; refresh the publish index first",
      "publish index at /Publish/index.html is not a regular DMS-owned file; move it aside before refreshing",
    ]) {
      for (const error of [message, new Error(message)]) {
        let state = createInitialState();
        state.workspace = { edit_root: "/DMS/Active" };
        state.releases.publish_index_available = true;
        assert.equal(await executePublishIndexAction(action, {
          getState: () => state,
          setState: (next) => { state = next; },
          invoke: async () => { throw error; },
        }), false);
        assert.equal(state.releases.publish_index_available, false);
        assert.equal(state.releases.index_error, String(error));
        assert.equal(state.releases.index_busy, false);
      }
    }
  }
});

test("generic index write and host-open failures preserve previous availability", async () => {
  for (const [action, message] of [
    ["update", "I/O error at /Publish/index.html: Permission denied"],
    ["open", "cannot open /Publish/index.html: Permission denied"],
    ["update", "I/O error at /DMS-owned/index.html: Permission denied"],
    ["open", "cannot open /DMS-owned/index.html: Permission denied"],
  ]) {
    for (const available of [true, false]) {
      for (const error of [message, new Error(message)]) {
        let state = createInitialState();
        state.workspace = { edit_root: "/DMS/Active" };
        state.releases = { ...state.releases, publish_index_available: available, index_notice: "Previous success" };
        assert.equal(await executePublishIndexAction(action, {
          getState: () => state,
          setState: (next) => { state = next; },
          invoke: async () => { throw error; },
        }), false);
        assert.equal(state.releases.publish_index_available, available, `${action}: ${message}`);
        assert.equal(state.releases.index_error, String(error));
        assert.equal(state.releases.index_notice, "");
        assert.equal(state.releases.index_busy, false);
      }
    }
  }
});

test("late publish index outcomes cannot update a replacement session", async () => {
  let state = createInitialState();
  state.workspace = { edit_root: "/DMS/Old" };
  let settle;
  const pending = executePublishIndexAction("update", {
    getState: () => state,
    setState: (next) => { state = next; },
    invoke: () => new Promise((resolve) => { settle = resolve; }),
  });
  state = workspaceActivationState(state, { workspace: { edit_root: "/DMS/New" }, lock_status: {} });
  const replacement = state;
  settle({ status: "written" });
  await pending;
  assert.equal(state, replacement);
});

test("failed close refresh retains window, lock, activities, drafts, and batch session for retry", async () => {
  const harness = batchHarness();
  await executeLibraryBatch("add", harness);
  const state = harness.getState();
  state.maintenance.lock_status = { lock: { process_id: 17 } };
  const before = { ...state };
  const calls = [];
  await assert.rejects(closeWorkspaceSession(state, async (command, arguments_) => {
    calls.push({ command, arguments_ });
    throw new Error("/Publish/index.html is not DMS-owned");
  }, async () => { assert.fail("window must remain open"); }), /index\.html is not DMS-owned/);
  assert.deepEqual(calls, [{ command: "refresh_publish_index", arguments_: { editRoot: "/private/Edit" } }]);
  assert.deepEqual({ ...state, session_error: undefined }, { ...before, session_error: undefined });
  assert.match(state.session_error, /current library remains open/);
  assert.match(state.session_error, /\/Publish\/index\.html/);
  const retryCalls = [];
  await closeWorkspaceSession(state, async (command) => {
    retryCalls.push(command);
    if (command === "refresh_publish_index") {
      assert.equal(state.batch_execution, before.batch_execution);
      assert.equal(state.workspace_session, before.workspace_session);
      return { status: "unchanged" };
    }
  }, async () => { retryCalls.push("destroy"); });
  assert.deepEqual(retryCalls, ["refresh_publish_index", "release_workspace_lock", "destroy"]);
  assert.equal(state.session_error, "");
});

test("source refresh failure prevents destination acquisition and session reset", async () => {
  const harness = batchHarness();
  await executeLibraryBatch("add", harness);
  let state = harness.getState();
  state.maintenance.lock_status = { lock: { process_id: 17 } };
  const before = state;
  const calls = [];
  await assert.rejects(async () => {
    const attempt = await switchWorkspaceSession(state.workspace, state.maintenance.lock_status,
      { edit_root: "/DMS/New" }, {}, async (command, arguments_) => {
        calls.push({ command, arguments_ });
        throw new Error("cannot write /Publish/index.html");
      });
    state = workspaceActivationState(state, attempt.activation);
  }, /cannot write \/Publish\/index\.html/);
  assert.deepEqual(calls, [{ command: "refresh_publish_index", arguments_: { editRoot: "/private/Edit" } }]);
  assert.equal(state, before);
  assert.equal(state.batch_execution.entries.length, 1);
});

test("written and unchanged source refresh precede switch acquisition, release, and reset", async () => {
  for (const status of ["written", "unchanged"]) {
    const harness = batchHarness();
    await executeLibraryBatch("add", harness);
    let state = harness.getState();
    state.maintenance.lock_status = { lock: { process_id: 17 } };
    const before = state;
    const calls = [];
    const activation = { workspace: { edit_root: "/DMS/New" }, lock_status: { lock: { process_id: 18 } } };
    const result = await switchWorkspaceSession(state.workspace, state.maintenance.lock_status, activation.workspace, {},
      async (command, arguments_) => {
        calls.push([command, arguments_.editRoot]);
        assert.equal(state, before);
        return command === "activate_workspace_session" ? activation : { status };
      });
    state = workspaceActivationState(state, result.activation);
    calls.push(["reset", state.workspace.edit_root]);
    assert.deepEqual(calls, [["refresh_publish_index", "/private/Edit"], ["activate_workspace_session", "/DMS/New"],
      ["release_workspace_lock", "/private/Edit"], ["reset", "/DMS/New"]]);
    assert.deepEqual(state.activities, []);
    assert.deepEqual(state.batch_execution.entries, []);
  }
});

test("blocked destination still retains the source session after a successful refresh", async () => {
  const harness = batchHarness();
  const state = harness.getState();
  const calls = [];
  await assert.rejects(switchWorkspaceSession(state.workspace, { lock: { process_id: 17 } },
    { edit_root: "/DMS/Blocked" }, {}, async (command) => {
      calls.push(command);
      if (command === "activate_workspace_session") throw new Error("workspace advisory lock is held");
      return { status: "unchanged" };
    }), /advisory lock/);
  assert.deepEqual(calls, ["refresh_publish_index", "activate_workspace_session"]);
  assert.equal(harness.getState(), state);
});

test("delayed Entra activation refreshes the source before every poll that can acquire the destination", async () => {
  for (const refreshStatus of ["written", "unchanged"]) {
    const calls = [];
    let polls = 0;
    const pending = { kind: "pending", edit_root: "/DMS/New", next_poll_delay_ms: 5000 };
    const activated = {
      kind: "valid",
      edit_root: "/DMS/New",
      activation: { workspace: { edit_root: "/DMS/New" }, lock_status: { lock: { process_id: 18 } } },
    };
    const lockOptions = { takeOverStale: true, overrideExisting: false };
    const invoke = async (command, arguments_) => {
      calls.push({ command, arguments_ });
      if (command === "refresh_publish_index") return { status: refreshStatus };
      assert.equal(command, "poll_library_session_authorization");
      polls += 1;
      return polls === 1 ? pending : activated;
    };
    let state = createInitialState();
    state.workspace = { edit_root: "/DMS/Old" };
    const first = await pollWorkspaceSessionAuthorization(
      state.workspace, { edit_root: "/DMS/New", lock_options: lockOptions }, invoke,
    );
    assert.deepEqual(first, pending);
    state = applyLibrarySessionAuthorization(state, first, lockOptions);
    const second = await pollWorkspaceSessionAuthorization(state.workspace, state.library_session_authorization, invoke);
    assert.deepEqual(second, activated);
    assert.deepEqual(calls, [
      { command: "refresh_publish_index", arguments_: { editRoot: "/DMS/Old" } },
      { command: "poll_library_session_authorization", arguments_: { editRoot: "/DMS/New", ...lockOptions } },
      { command: "refresh_publish_index", arguments_: { editRoot: "/DMS/Old" } },
      { command: "poll_library_session_authorization", arguments_: { editRoot: "/DMS/New", ...lockOptions } },
    ]);
  }
});

test("late library authorization activation preserves the replacement session and cleans up only its stale destination lock", async () => {
  for (const change of ["workspace", "session", "workspace-only", "active-destination", "active-destination-owned"]) {
    let state = createInitialState();
    state.workspace = { workspace_id: workspaceId, edit_root: "/DMS/Old" };
    state.library_session_authorization = { kind: "pending", edit_root: "/DMS/Stale", next_poll_delay_ms: 5000 };
    const originalSession = state.workspace_session;
    const staleOwner = { os_user: "operator", hostname: "host", process_id: 17, acquired_at: "stale" };
    const replacementOwner = { ...staleOwner, acquired_at: change === "active-destination-owned" ? "stale" : "replacement" };
    const calls = [];
    let activations = 0;
    let settle;
    let pollStarted;
    const started = new Promise((resolve) => { pollStarted = resolve; });
    const pending = executeLibrarySessionAuthorizationPoll({
      getState: () => state,
      setState: (next) => { state = next; },
      invoke: async (command, arguments_) => {
        calls.push({ command, arguments_ });
        if (command === "refresh_publish_index") return { status: "unchanged" };
        if (command === "poll_library_session_authorization") {
          return new Promise((resolve) => { settle = resolve; pollStarted(); });
        }
        if (command === "workspace_lock_status") return { lock: staleOwner };
        assert.equal(command, "release_workspace_lock");
        assert.notDeepEqual(arguments_.owner, state.maintenance.lock_status.lock, "replacement owner must not be released");
        assert.deepEqual(arguments_, { editRoot: "/DMS/Stale", owner: staleOwner, confirmed: true });
      },
      finishActivation: async () => { activations += 1; },
    });
    await started;
    const editRoot = change === "session" ? "/DMS/Old"
      : change.startsWith("active-destination") ? "/DMS/Stale" : "/DMS/Replacement";
    state = workspaceActivationState(state, {
      workspace: { workspace_id: workspaceId, edit_root: editRoot },
      lock_status: { state: "current", lock: replacementOwner },
    });
    if (change === "workspace-only") state.workspace_session = originalSession;
    state = openActivity(state, {
      workspace_id: workspaceId, destination: "Library", task: "Library", label: "Replacement Library", route_state: {},
    });
    state.session_error = "Replacement feedback";
    const replacement = state;
    settle({
      kind: "valid", edit_root: "/DMS/Stale",
      activation: { workspace: { edit_root: "/DMS/Stale" }, lock_status: { lock: staleOwner } },
    });
    await pending;
    assert.equal(activations, 0, "stale activation must not finish against the replacement source");
    assert.equal(state, replacement, change);
    assert.equal(state.activities, replacement.activities);
    assert.equal(state.workspace_session, replacement.workspace_session);
    assert.equal(state.maintenance.lock_status.lock, replacementOwner);
    assert.deepEqual(calls, [
      { command: "refresh_publish_index", arguments_: { editRoot: "/DMS/Old" } },
      { command: "poll_library_session_authorization", arguments_: { editRoot: "/DMS/Stale", takeOverStale: false, overrideExisting: false } },
      ...(change === "active-destination-owned" ? [] : [
        { command: "workspace_lock_status", arguments_: { editRoot: "/DMS/Stale" } },
        { command: "release_workspace_lock", arguments_: { editRoot: "/DMS/Stale", owner: staleOwner, confirmed: true } },
      ]),
    ]);
  }
});

for (const pause of ["source-release", "source-release-failure", "persist", "missing-owner-cleanup", "rollback-cleanup"]) {
  test(`library activation finishing preserves a replacement during ${pause}`, async () => {
    for (const replacementKind of ["different-root", "source-root-new-session", "same-root-new-owner", "same-root-same-owner"]) {
      let state = createInitialState();
      state.workspace = { workspace_id: workspaceId, edit_root: "/DMS/Old" };
      const sourceOwner = { os_user: "operator", hostname: "host", process_id: 17, acquired_at: "source" };
      const destinationOwner = { ...sourceOwner, acquired_at: "destination" };
      state.maintenance.lock_status = { lock: pause === "missing-owner-cleanup" ? null : sourceOwner };
      const activation = {
        workspace: { workspace_id: "destination", edit_root: "/DMS/New" },
        lock_status: { lock: destinationOwner },
      };
      const calls = [];
      let settle;
      let notifyStarted;
      const started = new Promise((resolve) => { notifyStarted = resolve; });
      const delay = () => new Promise((resolve, reject) => {
        settle = () => pause === "source-release-failure" ? reject(new Error("source release failed")) : resolve();
        notifyStarted();
      });
      const pending = finishLibrarySessionActivation(activation, {
        getState: () => state,
        setState: (next) => { state = next; },
        invoke: async (command, arguments_) => {
          calls.push({ command, arguments_ });
          if (command === "workspace_lock_status") {
            assert.deepEqual(arguments_, { editRoot: "/DMS/New" });
            if (pause.endsWith("cleanup")) await delay();
            return { lock: destinationOwner };
          }
          assert.equal(command, "release_workspace_lock");
          if (arguments_.editRoot === "/DMS/Old") {
            assert.equal(arguments_.owner, sourceOwner);
            if (pause === "rollback-cleanup") throw new Error("source release failed");
            if (pause.startsWith("source-release")) return delay();
          } else {
            assert.deepEqual(arguments_, { editRoot: "/DMS/New", owner: destinationOwner, confirmed: true });
          }
        },
        persist: async () => {
          assert.equal(pause, "persist", "stale activation must not apply or persist");
          calls.push({ command: "persist" });
          return delay();
        },
        openLibrary: () => { assert.fail("stale activation must not open Library on the replacement"); },
      });
      await started;
      const replacementOwner = {
        ...destinationOwner,
        acquired_at: replacementKind === "same-root-same-owner" ? "destination" : "replacement",
      };
      state = workspaceActivationState(state, {
        workspace: replacementKind === "source-root-new-session"
          ? { workspace_id: workspaceId, edit_root: "/DMS/Old" }
          : { workspace_id: "destination", edit_root: replacementKind === "different-root" ? "/DMS/Replacement" : "/DMS/New" },
        lock_status: { lock: replacementOwner },
      });
      state = openActivity(state, {
        workspace_id: state.workspace.workspace_id, destination: "Library", task: "Library", label: "Replacement Library", route_state: {},
      });
      state.session_error = "Replacement feedback";
      const replacement = state;
      settle();
      await pending;
      assert.equal(state, replacement);
      assert.equal(state.activities, replacement.activities);
      assert.equal(state.maintenance.lock_status.lock, replacementOwner);
      assert.deepEqual(calls, [
        ...(pause === "missing-owner-cleanup" ? [] : [{
          command: "release_workspace_lock", arguments_: { editRoot: "/DMS/Old", owner: sourceOwner, confirmed: true },
        }]),
        ...(pause === "persist" ? [{ command: "persist" }] : []),
        ...(pause.endsWith("cleanup") || replacementKind !== "same-root-same-owner" ? [{
          command: "workspace_lock_status", arguments_: { editRoot: "/DMS/New" },
        }] : []),
        ...(replacementKind !== "same-root-same-owner" ? [{
          command: "release_workspace_lock", arguments_: { editRoot: "/DMS/New", owner: destinationOwner, confirmed: true },
        }] : []),
      ], replacementKind);
    }
  });
}

for (const cleanup of ["missing-owner", "source-release-failure"]) {
  test(`${cleanup} rollback preserves replacement locks across destination IPC`, async (t) => {
    const warnings = [];
    t.mock.method(console, "warn", (...args) => warnings.push(args));
    for (const change of ["before-status", "during-release", "different-root-same-owner", "unlocked"]) {
      let state = createInitialState();
      state.workspace = { workspace_id: workspaceId, edit_root: "/DMS/Old" };
      const sourceOwner = { os_user: "operator", hostname: "host", process_id: 17, acquired_at: "source" };
      const staleOwner = { ...sourceOwner, acquired_at: "stale" };
      const replacementOwner = { ...staleOwner, acquired_at: change === "different-root-same-owner" ? "stale" : "replacement" };
      state.maintenance.lock_status = { lock: cleanup === "missing-owner" ? null : sourceOwner };
      const activation = { workspace: { edit_root: "/DMS/New" }, lock_status: { lock: staleOwner } };
      const locks = new Map([["/DMS/New", staleOwner]]);
      let settle;
      let notifyStarted;
      const started = new Promise((resolve) => { notifyStarted = resolve; });
      const pause = () => new Promise((resolve) => { settle = resolve; notifyStarted(); });
      const calls = [];
      const pending = finishLibrarySessionActivation(activation, {
        getState: () => state,
        setState: () => assert.fail("rollback cannot activate a workspace"),
        invoke: async (command, arguments_) => {
          calls.push({ command, arguments_ });
          if (arguments_.editRoot === "/DMS/Old") throw new Error("source release failed");
          if (command === "workspace_lock_status") {
            if (change !== "during-release") await pause();
            return { lock: locks.get(arguments_.editRoot) ?? null };
          }
          assert.equal(command, "release_workspace_lock");
          assert.deepEqual(arguments_, { editRoot: "/DMS/New", owner: staleOwner, confirmed: true });
          if (change === "during-release") await pause();
          // Model the real IPC's owner check at execution, not at dispatch.
          if (JSON.stringify(locks.get(arguments_.editRoot)) !== JSON.stringify(arguments_.owner)) {
            throw new Error("workspace lock ownership changed");
          }
          locks.delete(arguments_.editRoot);
        },
        persist: () => assert.fail("rollback cannot persist"),
        openLibrary: () => assert.fail("rollback cannot open Library"),
      });
      await started;
      const replacementRoot = change === "different-root-same-owner" ? "/DMS/Replacement" : "/DMS/New";
      locks.set(replacementRoot, replacementOwner);
      if (change === "unlocked") locks.delete("/DMS/New");
      state = workspaceActivationState(state, {
        workspace: { workspace_id: "replacement", edit_root: replacementRoot },
        lock_status: { lock: replacementOwner },
      });
      state = openActivity(state, {
        workspace_id: "replacement", destination: "Library", task: "Library", label: "Replacement Library", route_state: {},
      });
      state.session_error = "Replacement feedback";
      const replacement = state;
      settle();
      await pending;
      assert.equal(state, replacement, change);
      assert.equal(state.activities, replacement.activities);
      assert.equal(state.maintenance.lock_status.lock, replacementOwner);
      const released = change === "during-release" || change === "different-root-same-owner";
      assert.deepEqual(calls, [
        ...(cleanup === "missing-owner" ? [] : [{ command: "release_workspace_lock", arguments_: { editRoot: "/DMS/Old", owner: sourceOwner, confirmed: true } }]),
        { command: "workspace_lock_status", arguments_: { editRoot: "/DMS/New" } },
        ...(released ? [{ command: "release_workspace_lock", arguments_: { editRoot: "/DMS/New", owner: staleOwner, confirmed: true } }] : []),
      ], change);
      assert.equal(locks.get(replacementRoot), change === "unlocked" ? undefined : replacementOwner, change);
      if (change === "different-root-same-owner") assert.equal(locks.has("/DMS/New"), false);
    }
    assert.equal(warnings.length, 1, "only the in-flight owner change rejects release");
  });
}

test("current activation finishing releases the source, applies, persists, and opens Library", async () => {
  let state = createInitialState();
  state.workspace = { edit_root: "/DMS/Old" };
  const sourceOwner = { process_id: 17 };
  state.maintenance.lock_status = { lock: sourceOwner };
  const activation = { workspace: { edit_root: "/DMS/New" }, lock_status: { lock: { process_id: 18 } } };
  const calls = [];
  await finishLibrarySessionActivation(activation, {
    getState: () => state,
    setState: (next) => { state = next; calls.push("apply"); },
    invoke: async (command, arguments_) => {
      assert.equal(command, "release_workspace_lock");
      assert.deepEqual(arguments_, { editRoot: "/DMS/Old", owner: sourceOwner, confirmed: true });
      calls.push("release");
    },
    persist: async (current) => { assert.equal(current, state); calls.push("persist"); },
    openLibrary: () => { assert.equal(state.workspace, activation.workspace); calls.push("open"); },
  });
  assert.deepEqual(calls, ["release", "apply", "persist", "open"]);
});

test("current activation finishing rolls back only its destination when source release cannot complete", async () => {
  for (const missingOwner of [true, false]) {
    let state = createInitialState();
    state.workspace = { edit_root: "/DMS/Old" };
    const sourceOwner = { process_id: 17 };
    state.maintenance.lock_status = { lock: missingOwner ? null : sourceOwner };
    const initial = state;
    const destinationOwner = { process_id: 18 };
    const activation = { workspace: { edit_root: "/DMS/New" }, lock_status: { lock: destinationOwner } };
    const calls = [];
    await assert.rejects(finishLibrarySessionActivation(activation, {
      getState: () => state,
      setState: (next) => { state = next; },
      invoke: async (command, arguments_) => {
        calls.push({ command, arguments_ });
        if (arguments_.editRoot === "/DMS/Old") throw new Error("source release failed");
        if (command === "workspace_lock_status") return { lock: destinationOwner };
      },
      persist: async () => { assert.fail("failed activation must not persist"); },
      openLibrary: () => { assert.fail("failed activation must not open Library"); },
    }), missingOwner ? /lock owner is unavailable/ : /source release failed/);
    assert.equal(state, initial);
    assert.deepEqual(calls, [
      ...(missingOwner ? [] : [{ command: "release_workspace_lock", arguments_: { editRoot: "/DMS/Old", owner: sourceOwner, confirmed: true } }]),
      { command: "workspace_lock_status", arguments_: { editRoot: "/DMS/New" } },
      { command: "release_workspace_lock", arguments_: { editRoot: "/DMS/New", owner: destinationOwner, confirmed: true } },
    ]);
  }
});

test("current library authorization activation still finishes after refreshing its source", async () => {
  let state = createInitialState();
  state.workspace = { edit_root: "/DMS/Old" };
  state.library_session_authorization = { kind: "pending", edit_root: "/DMS/New", next_poll_delay_ms: 5000 };
  state.session_error = "Previous refresh failure";
  const activation = { workspace: { edit_root: "/DMS/New" }, lock_status: { lock: { process_id: 17 } } };
  const calls = [];
  await executeLibrarySessionAuthorizationPoll({
    getState: () => state,
    setState: (next) => { state = next; },
    invoke: async (command) => {
      calls.push(command);
      return command === "refresh_publish_index" ? { status: "unchanged" }
        : { kind: "valid", edit_root: "/DMS/New", activation };
    },
    finishActivation: async (result) => {
      assert.equal(result, activation);
      assert.equal(state.library_session_authorization.kind, "valid");
      assert.equal(state.session_error, "");
      calls.push("finish");
      state = workspaceActivationState(state, result);
    },
  });
  assert.deepEqual(calls, ["refresh_publish_index", "poll_library_session_authorization", "finish"]);
  assert.equal(state.workspace, activation.workspace);
});

test("late library authorization pending and failure results leave replacement state untouched", async () => {
  for (const outcome of ["pending", "poll-failure", "refresh-failure"]) {
    let state = createInitialState();
    state.workspace = { edit_root: "/DMS/Old" };
    state.library_session_authorization = { kind: "pending", edit_root: "/DMS/Stale", next_poll_delay_ms: 5000 };
    let settle;
    let pollStarted;
    const started = new Promise((resolve) => { pollStarted = resolve; });
    const pending = executeLibrarySessionAuthorizationPoll({
      getState: () => state,
      setState: (next) => { state = next; },
      invoke: async (command) => {
        if (command === "refresh_publish_index" && outcome !== "refresh-failure") return { status: "unchanged" };
        return new Promise((resolve, reject) => {
          settle = () => outcome === "pending" ? resolve({ kind: "pending", edit_root: "/DMS/Stale" }) : reject(new Error("late failure"));
          pollStarted();
        });
      },
      finishActivation: async () => { assert.fail("no activation expected"); },
    });
    await started;
    state = workspaceActivationState(state, { workspace: { edit_root: "/DMS/Replacement" }, lock_status: { lock: { process_id: 18 } } });
    const replacement = state;
    settle();
    await pending;
    assert.equal(state, replacement, outcome);
  }
});

test("a failed source refresh on a later poll blocks the poll without acquiring or releasing a lock", async () => {
  const calls = [];
  let refreshes = 0;
  let polls = 0;
  const invoke = async (command, arguments_) => {
    calls.push([command, arguments_.editRoot]);
    if (command === "refresh_publish_index") {
      refreshes += 1;
      if (refreshes === 1) return { status: "unchanged" };
      throw new Error("cannot write /Publish/index.html");
    }
    assert.equal(command, "poll_library_session_authorization");
    polls += 1;
    assert.equal(polls, 1, "a second poll would acquire the destination lock");
    return { kind: "pending", edit_root: "/DMS/New" };
  };
  const first = await pollWorkspaceSessionAuthorization(
    { edit_root: "/DMS/Old" }, { edit_root: "/DMS/New" }, invoke,
  );
  assert.equal(first.kind, "pending");
  await assert.rejects(
    pollWorkspaceSessionAuthorization({ edit_root: "/DMS/Old" }, first, invoke),
    /current library remains open/,
  );
  assert.deepEqual(calls, [
    ["refresh_publish_index", "/DMS/Old"],
    ["poll_library_session_authorization", "/DMS/New"],
    ["refresh_publish_index", "/DMS/Old"],
  ]);
  assert.equal(polls, 1);
});

test("delayed Entra activation retains its pending sign-in state when source refresh fails", () => {
  let state = createInitialState();
  state.library_session_authorization = {
    kind: "pending",
    edit_root: "/DMS/New",
    next_poll_delay_ms: 5000,
    lock_options: {},
  };
  const retained = applyLibrarySessionPollFailure(
    state,
    state.library_session_authorization,
    new PublishIndexRefreshError("Released document index refresh failed; the current library remains open."),
  );
  assert.equal(retained.library_session_authorization, state.library_session_authorization);
  assert.equal(retained.session_error, "Released document index refresh failed; the current library remains open.");
  assert.equal(shouldPollLibrarySessionAuthorization(retained.library_session_authorization), true);
  const failed = applyLibrarySessionPollFailure(state, state.library_session_authorization, new Error("Entra poll failed"));
  assert.equal(failed.library_session_authorization.kind, "failed");
});

test("Open library switcher opens and dismisses without replacing the workspace", () => {
  const workspace = { workspace_id: workspaceId, edit_root: "/DMS/Edit" };
  const opened = openActivity(createInitialState(), {
    workspace_id: workspaceId,
    destination: "Library",
    task: "Library",
    label: "Library · /",
    document_id: null,
    route_state: { folder: "." },
  });
  opened.workspace = workspace;
  const currentKey = opened.current_key;
  const switching = openLibrarySwitcher(opened);
  assert.equal(switching.library_switcher.open, true);
  assert.equal(switching.workspace, workspace);
  assert.equal(switching.current_key, currentKey);
  const dismissed = dismissLibrarySwitcher(switching);
  assert.equal(dismissed.library_switcher.open, false);
  assert.equal(dismissed.workspace, workspace);
  assert.equal(dismissed.current_key, currentKey);
  assert.equal(dismissed.activities.length, 1);
});

test("active switcher reuses startup open and initialize controls", () => {
  const markup = librarySwitcherMarkup("", ["/Users/name/DMS/Edit"], "/Users/name/DMS/Edit");
  assert.match(markup, /aria-label="Open library"/);
  assert.match(markup, /data-library-switcher-dismiss/);
  assert.match(markup, /id="open-workspace-form"/);
  assert.match(markup, /id="initialize-workspace-form"/);
  assert.match(markup, /name="takeOverStale"/);
  assert.match(markup, /name="overrideExisting"/);
  assert.match(markup, /name="confirmed"[^>]*required/);
  assert.match(markup, /data-recent-library-open="\/Users\/name\/DMS\/Edit"/);
  assert.doesNotMatch(markup, /process_id|process ID/i);
});

test("blocked current and stale locks name the recorded OS user and host", () => {
  const current = libraryLockOwnerFeedback("workspace has a current advisory lock", {
    state: "current",
    lock: {
      os_user: "anna",
      hostname: "desk-1",
      process_id: 99,
      acquired_at: "2026-08-29T10:00:00Z",
    },
  });
  const stale = libraryLockOwnerFeedback("workspace lock is stale; explicit take-over is required", {
    state: "stale",
    lock: {
      os_user: "lukas",
      hostname: "desk-2",
      process_id: 42,
      acquired_at: "2026-08-28T09:00:00Z",
    },
  });
  assert.equal(isAdvisoryLockRefusal("workspace has a current advisory lock"), true);
  assert.equal(isAdvisoryLockRefusal("workspace lock is stale; explicit take-over is required"), true);
  const currentMarkup = setupMarkup("workspace has a current advisory lock", [], "", current);
  const staleMarkup = librarySwitcherMarkup(
    "workspace lock is stale; explicit take-over is required",
    [],
    "",
    stale,
  );
  assert.match(currentMarkup, /Held by anna on desk-1 \(current, acquired 2026-08-29T10:00:00Z\)/);
  assert.match(staleMarkup, /Held by lukas on desk-2 \(stale, acquired 2026-08-28T09:00:00Z\)/);
  assert.doesNotMatch(currentMarkup, /99/);
  assert.doesNotMatch(staleMarkup, /42/);
});

test("a lock-status query failure keeps the original refusal without an owner claim", () => {
  assert.equal(
    libraryLockOwnerFeedback("workspace has a current advisory lock", { state: "unlocked", lock: null }),
    null,
  );
  const markup = librarySwitcherMarkup("workspace has a current advisory lock", [], "/DMS/Blocked", null);
  assert.match(markup, /workspace has a current advisory lock/);
  assert.doesNotMatch(markup, /Held by/);
});

test("the Open library control is present in the shell chrome", () => {
  const shell = readFileSync(new URL("./index.html", import.meta.url), "utf8");
  assert.match(shell, /id="open-library"[^>]*>Open library…/);
});

test("preferences start expanded and persist no session activities", () => {
  const state = createInitialState(defaultPreferences());

  assert.equal(state.preferences.sidebar_expanded, true);
  assert.deepEqual(state.preferences.saved_views, []);
  assert.deepEqual(state.preferences.recent_libraries, []);
  assert.deepEqual(state.preferences.library_table_column_widths, {});
  assert.deepEqual(state.activities, []);
});

test("Library width preferences hydrate across sessions and reset only that personal layout", () => {
  const preferences = {
    ...defaultPreferences(),
    sidebar_expanded: false,
    saved_views: [{ id: "saved", workspace_id: workspaceId }],
    recent_libraries: ["/DMS/Edit"],
    library_table_column_widths: { "col-name": 260, unknown: 999 },
  };
  let state = createInitialState(preferences);
  state = {
    ...state,
    library: applyLibrarySort({ ...state.library, detail_width: 512 }, "title", "descending"),
  };
  const activity = {
    workspace_id: workspaceId,
    destination: "Library",
    task: "Library",
    label: "Library · Policies",
    document_id: null,
    route_state: { folder: "Policies" },
  };
  state = openActivity(state, activity);

  assert.deepEqual(state.preferences.library_table_column_widths, { "col-name": 260 });
  assert.deepEqual(state.library.column_widths, { "col-name": 260 });
  assert.equal(bookmarkActivity(state).route_state.sort_direction, "descending");
  assert.equal(
    savedViewId({ ...activity, route_state: { folder: "Policies", sort: "title", sort_direction: "ascending" } }),
    `${workspaceId}:saved:Library:Policies:title:none`,
  );
  assert.notEqual(
    savedViewId({ ...activity, route_state: { folder: "Policies", sort: "title", sort_direction: "ascending" } }),
    savedViewId({ ...activity, route_state: { folder: "Policies", sort: "title", sort_direction: "descending" } }),
  );

  state = resetLibraryTableLayout(state);
  assert.deepEqual(state.preferences.library_table_column_widths, {});
  assert.deepEqual(state.library.column_widths, {});
  assert.equal(state.library.detail_width, 512);
  assert.equal(state.preferences.sidebar_expanded, false);
  assert.deepEqual(state.preferences.saved_views, preferences.saved_views);
  assert.deepEqual(state.preferences.recent_libraries, preferences.recent_libraries);
});

test("the expanded sidebar identifies the local author recorded for workspace changes", () => {
  const markup = workspaceFootMarkup({
    workspace_id: workspaceId,
    change_author: "Ada <operator>",
    edit_root: "/DMS/Edit",
    publish_root: "/DMS/Publish",
  });

  assert.match(markup, /Changes recorded as/);
  assert.match(markup, /Ada &lt;operator&gt;/);
  assert.match(markup, new RegExp(workspaceId));
  assert.match(markup, /edit: \/DMS\/Edit/);
  assert.match(markup, /publish: \/DMS\/Publish/);
  assert.equal(workspaceFootMarkup(null), "No workspace open");
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
  const workspace = permalinkActivity({
    workspace: resolution.workspace,
    document_id: null,
    title: null,
    document_number: null,
    folder: ".",
    target: "workspace",
    review_id: null,
  });

  assert.equal(activityKey(document), `${workspaceId}:Library`);
  assert.equal(document.document_id, resolution.document_id);
  assert.deepEqual(document.route_state, { folder: "Policies/HR" });
  assert.equal(activityKey(notes), `${workspaceId}:Notes:document:${resolution.document_id}`);
  assert.deepEqual(notes.route_state, { folder: "Policies/HR" });
  assert.equal(activityKey(review), `${workspaceId}:Review:document:${resolution.document_id}`);
  assert.equal(review.route_state.review, "5a6382c0-3078-4cf9-a64c-d194686bd1f6");
  assert.equal(activityKey(workspace), `${workspaceId}:Library`);
  assert.equal(workspace.label, "Library · /");
  assert.equal(workspace.document_id, null);
  assert.deepEqual(workspace.route_state, { folder: "." });

  const focused = openActivity(openActivity(createInitialState(), document), workspace);
  assert.equal(focused.activities.length, 1);
  assert.equal(focused.current_key, `${workspaceId}:Library`);
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

test("left menu destination icons match the wireframe navigation", () => {
  const source = readFileSync(new URL("./app.mjs", import.meta.url), "utf8");
  const generator = readFileSync(
    new URL("../../../docs/product/wireframes/generate.mjs", import.meta.url),
    "utf8",
  );
  const nav = [...generator.matchAll(/\{ id: "[a-z]+", label: "(.+?)", icon: "(.*?)" \}/gu)];
  assert.ok(nav.length >= 5);
  for (const [, label, icon] of nav) {
    assert.match(source, new RegExp(`\\["${label.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\$&")}", "${icon.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\$&")}"\\]`));
  }
});

test("app-shell startup authorization is visible without a workspace and on later routes", () => {
  const pending = {
    kind: "pending",
    user_code: "ABCD-EFGH",
    verification_uri: "https://microsoft.com/devicelogin",
    message: "Enter the code to sign in.",
    expires_in_seconds: 900,
    next_poll_delay_ms: 5000,
  };
  const setup = applyStartupAuthorization(createInitialState(), pending);
  const withWorkspace = applyStartupAuthorization({
    ...createInitialState(),
    workspace: { workspace_id: workspaceId, edit_root: "/DMS/Edit" },
  }, pending);
  const markup = startupAuthorizationMarkup(setup.startup_authorization);

  assert.equal(setup.workspace, null);
  assert.ok(withWorkspace.workspace);
  assert.match(markup, /data-startup-authorization="pending"/);
  assert.match(markup, /ABCD-EFGH/);
  assert.match(markup, /Open sign-in page/);
  assert.match(markup, /data-open-external="https:\/\/microsoft.com\/devicelogin"/);
  assert.doesNotMatch(markup, /device_code|access_token|refresh_token/);
  assert.equal(startupAuthorizationMarkup({ kind: "inactive" }), "");
  assert.match(startupAuthorizationMarkup({ kind: "valid", message: "Signed in to Microsoft Entra." }), /data-startup-authorization="valid"/);
});

test("startup authorization polls only while pending and cancels the previous timer", () => {
  const calls = [];
  const timers = {
    setTimeout(poll, delay) {
      calls.push(["set", delay]);
      return calls.length;
    },
    clearTimeout(id) {
      calls.push(["clear", id]);
    },
  };
  const pending = { kind: "pending", next_poll_delay_ms: 4000 };
  const first = scheduleStartupAuthorizationPoll(pending, () => {}, timers);
  cancelStartupAuthorizationPoll(first, timers);
  const terminal = scheduleStartupAuthorizationPoll({ kind: "expired" }, () => {}, timers);

  assert.equal(shouldPollStartupAuthorization(pending), true);
  assert.equal(startupAuthorizationPollDelayMs(pending), 4000);
  assert.equal(shouldPollStartupAuthorization({ kind: "declined" }), false);
  assert.equal(startupAuthorizationPollDelayMs({ kind: "declined" }), null);
  assert.equal(shouldPollStartupAuthorization({ kind: "pending", next_poll_delay_ms: null }), false);
  assert.deepEqual(calls, [["set", 4000], ["clear", 1]]);
  assert.equal(terminal, null);
});

test("terminal startup authorization offers explicit reissue and unavailable does not", () => {
  const expired = startupAuthorizationMarkup({
    kind: "expired",
    message: "Microsoft Entra sign-in expired.",
  });
  const declined = startupAuthorizationMarkup({
    kind: "declined",
    message: "Microsoft Entra sign-in was declined.",
  });
  const unavailable = startupAuthorizationMarkup({
    kind: "unavailable",
    message: "cannot access the OS credential store",
  });

  assert.match(expired, /Reissue code/);
  assert.match(expired, /data-startup-reissue/);
  assert.match(declined, /Reissue code/);
  assert.doesNotMatch(unavailable, /Reissue code/);
  assert.doesNotMatch(unavailable, /data-startup-reissue/);
});

test("bound-library authorization blocks destinations without exposing credentials", () => {
  const pending = {
    kind: "pending",
    edit_root: "/DMS/Quality",
    library_label: "Quality",
    group_label: "Quality workflow",
    user_code: "ABCD-EFGH",
    verification_uri: "https://microsoft.com/devicelogin",
    message: "Enter the code to sign in.",
    expires_in_seconds: 900,
    next_poll_delay_ms: 5000,
  };
  const state = applyLibrarySessionAuthorization(createInitialState(), pending, {
    takeOverStale: true,
  });
  const markup = librarySessionAuthorizationMarkup(state.library_session_authorization);

  assert.match(markup, /data-library-session-authorization="pending"/);
  assert.match(markup, /Sign in to open "Quality"/);
  assert.match(markup, /Quality workflow/);
  assert.match(markup, /ABCD-EFGH/);
  assert.match(markup, /Open sign-in page/);
  assert.match(markup, /Choose another library/);
  assert.match(markup, /data-library-session-dismiss/);
  assert.doesNotMatch(markup, /device_code|access_token|refresh_token/);
  assert.equal(shouldPollLibrarySessionAuthorization(state.library_session_authorization), true);
  assert.equal(librarySessionAuthorizationPollDelayMs(state.library_session_authorization), 5000);
  assert.equal(state.library_session_authorization.lock_options.takeOverStale, true);
  assert.equal(librarySessionHeading(state.library_session_authorization), 'Sign in to open "Quality"');
});

test("terminal bound-library authorization reissues only when the service is reachable", () => {
  const expired = librarySessionAuthorizationMarkup({
    kind: "expired",
    library_label: "Quality",
    group_label: "Quality workflow",
    message: "Microsoft Entra sign-in expired.",
  });
  const unavailable = librarySessionAuthorizationMarkup({
    kind: "unavailable",
    library_label: "Quality",
    message: "cannot access the OS credential store",
  });

  assert.match(expired, /data-library-session-reissue/);
  assert.match(expired, /Reissue code/);
  assert.match(expired, /Choose another library/);
  assert.doesNotMatch(unavailable, /data-library-session-reissue/);
  assert.match(unavailable, /Choose another library/);
  assert.doesNotMatch(unavailable, /Reapply identity source/);
});

test("unverified identity source explains reapply and returns to setup", () => {
  const unavailable = librarySessionAuthorizationMarkup({
    kind: "unavailable",
    edit_root: "/DMS/edit",
    library_label: "edit",
    group_label: "DMS Workflow Users",
    message: "this library's Microsoft Entra identity source has no verified tenant; reapply the identity source before signing in",
    recovery: "reapply_identity_source",
  });

  assert.match(unavailable, /Cannot open "edit"/);
  assert.match(unavailable, /has not recorded a verified tenant ID/);
  assert.match(unavailable, /writes the verified tenant with that group/);
  assert.match(unavailable, /data-library-session-reapply/);
  assert.match(unavailable, /Reapply identity source/);
  assert.match(unavailable, /Choose another library/);
  assert.doesNotMatch(unavailable, /reapply it before signing in/);
  assert.equal(librarySessionHeading({
    kind: "unavailable",
    library_label: "edit",
    recovery: "reapply_identity_source",
  }), 'Cannot open "edit"');

  const dismissed = dismissLibrarySessionAuthorization(applyLibrarySessionAuthorization(createInitialState(), {
    kind: "unavailable",
    edit_root: "/DMS/edit",
    library_label: "edit",
    recovery: "reapply_identity_source",
  }));
  assert.equal(dismissed.library_session_authorization.kind, "inactive");
  assert.equal(dismissed.setup_edit_root, "/DMS/edit");
  assert.equal(dismissed.current_key, null);
  assert.equal(dismissed.workspace, null);
});

test("identity-source recovery does not activate the blocked library", () => {
  const blocked = applyLibrarySessionAuthorization(createInitialState(), {
    kind: "unavailable",
    edit_root: "/DMS/edit",
    library_label: "edit",
    group_label: "DMS Workflow Users",
    recovery: "reapply_identity_source",
  });
  const recovered = applyLibrarySessionIdentityRecovery(blocked, {
    workspace: {
      workspace_id: workspaceId,
      edit_root: "/DMS/edit",
      publish_root: "/DMS/publish",
      document_count: 0,
    },
    identity_source: {
      binding_id: "binding-1",
      group_id: "group-1",
      group_label: "DMS Workflow Users",
    },
    eligible_people: [],
    global_entra_configuration: {
      client_id: "client-1",
      tenant_id: "tenant-1",
      client_id_source: "saved",
      tenant_id_source: "saved",
    },
    policy_folders: [{ relative_path: "." }],
    workflow_policies: [],
    document_types: [],
    confidentiality_types: [],
    confidentiality_policies: [],
  });
  const markup = librarySessionAuthorizationMarkup(
    recovered.library_session_authorization,
    recovered.configuration,
  );

  assert.equal(recovered.workspace, null);
  assert.equal(recovered.library_session_authorization.recovery_surface, "identity-source");
  assert.match(markup, /data-library-session-authorization="recovery"/);
  assert.match(markup, /Reapply identity source for "edit"/);
  assert.doesNotMatch(markup, /Cannot open edit/);
  assert.match(markup, /Unverified — reapply this identity source/);
  assert.match(markup, /value="group-1"/);
  assert.match(markup, /Choose another library/);
  assert.doesNotMatch(markup, /Back to Workflow/);
  assert.equal(librarySessionHeading(recovered.library_session_authorization, recovered.configuration), 'Reapply identity source for "edit"');

  const withPreview = {
    ...recovered,
    configuration: {
      ...recovered.configuration,
      identity_setup: {
        preview: {
          preview_id: "preview-1",
          tenant_display: "Example Healthcare GmbH",
          group_label: "DMS Workflow Users",
          eligible_people: [],
        },
      },
    },
  };
  const previewMarkup = librarySessionAuthorizationMarkup(
    withPreview.library_session_authorization,
    withPreview.configuration,
  );
  assert.match(previewMarkup, /Preview ready/);
  assert.match(previewMarkup, /Confirm identity source for "edit"/);
  assert.match(previewMarkup, /then Open "edit"/);
  assert.doesNotMatch(previewMarkup, /Cannot open edit/);
  assert.doesNotMatch(previewMarkup, /has not recorded a verified tenant ID/);
  assert.doesNotMatch(previewMarkup, /DMS will then try to open/);
  assert.equal(
    librarySessionHeading(withPreview.library_session_authorization, withPreview.configuration),
    'Confirm identity source for "edit"',
  );

  const applied = {
    ...recovered,
    library_session_authorization: {
      ...recovered.library_session_authorization,
      recovery_surface: "applied",
    },
  };
  const appliedMarkup = librarySessionAuthorizationMarkup(
    applied.library_session_authorization,
    applied.configuration,
  );
  assert.match(appliedMarkup, /Identity source applied/);
  assert.match(appliedMarkup, /data-library-session-open/);
  assert.match(appliedMarkup, />Open "edit"</);
  assert.doesNotMatch(appliedMarkup, /DMS will then try to open/);
  assert.equal(librarySessionHeading(applied.library_session_authorization), 'Open "edit"');
});

test("identity-source device-flow polls until preview and stops after a terminal result", () => {
  const pending = {
    challenge: {
      challenge_id: "challenge-1",
      user_code: "ABCD-EFGH",
      verification_uri: "https://microsoft.com/devicelogin",
      poll_interval_seconds: 5,
    },
    last_group_id: "group-1",
    next_poll_delay_ms: 1500,
  };
  assert.equal(shouldPollIdentitySource(pending), true);
  assert.equal(identitySourcePollDelayMs(pending), 1500);
  assert.equal(shouldPollIdentitySource({ ...pending, preview: { preview_id: "p1" } }), false);
  assert.equal(shouldPollIdentitySource({ ...pending, terminal: "expired", next_poll_delay_ms: null }), false);
});
