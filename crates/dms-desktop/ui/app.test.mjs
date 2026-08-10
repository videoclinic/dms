import test from "node:test";
import assert from "node:assert/strict";

import {
  activityKey,
  closeActivity,
  createInitialState,
  defaultPreferences,
  openActivity,
  savedViewId,
  setupMarkup,
  toggleSavedView,
  workspaceSetupRequest,
} from "./app.mjs";

const workspaceId = "5ef3db10-8f6d-4ae4-9d68-ecb1eaac8235";

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
  const markup = setupMarkup("");

  assert.match(markup, /id="open-workspace-form"/);
  assert.match(markup, /id="initialize-workspace-form"/);
  assert.match(markup, /name="editRoot"/);
  assert.match(markup, /name="publishRoot"/);
  assert.match(markup, /name="confirmed"[^>]*required/);
});

test("workspace setup maps each form to its explicit desktop command", () => {
  assert.deepEqual(
    workspaceSetupRequest("open-workspace-form", { editRoot: " C:\\DMS\\Edit " }),
    { command: "open_workspace", arguments: { editRoot: "C:\\DMS\\Edit" } },
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

test("preferences start expanded and persist no session activities", () => {
  const state = createInitialState(defaultPreferences());

  assert.equal(state.preferences.sidebar_expanded, true);
  assert.deepEqual(state.preferences.saved_views, []);
  assert.deepEqual(state.activities, []);
});

test("opening the same document task focuses one stable activity", () => {
  let state = createInitialState(defaultPreferences());
  state = openActivity(state, documentActivity("Audit"));
  state = openActivity(state, { ...documentActivity("Audit"), label: "Audit · Renamed policy · DOC-014" });

  assert.equal(state.activities.length, 1);
  assert.equal(state.activities[0].label, "Audit · Renamed policy · DOC-014");
  assert.equal(state.current_key, activityKey(documentActivity("Audit")));
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
