import test from "node:test";
import assert from "node:assert/strict";

import {
  applyConfigurationSnapshot,
  configurationMarkup,
  configurationMutationRequest,
  createConfigurationState,
  selectConfigurationFolder,
  setConfigurationRoute,
} from "./configuration.mjs";

const workspace = {
  workspace_id: "workspace-1",
  edit_root: "/DMS/Edit",
  publish_root: "/DMS/Publish",
  document_count: 4,
};

const snapshot = {
  workspace,
  default_review_interval_months: 12,
  document_types: [
    { id: "policy", label: "Policy", enabled: true },
    { id: "record", label: "Record", enabled: false },
  ],
  confidentiality_types: [
    { id: "internal", label: "Internal", enabled: true },
    { id: "restricted", label: "Restricted", enabled: true },
  ],
  confidentiality_policies: [
    { folder: ".", type_id: "internal" },
    { folder: "Policies/HR", type_id: "restricted" },
  ],
  policy_folders: [
    { relative_path: "." },
    { relative_path: "Policies" },
    { relative_path: "Policies/HR" },
  ],
};

const assistancePolicy = {
  value: {
    enabled: false,
    allowed_confidentiality_type_ids: ["internal"],
    max_payload_chars: 24000,
  },
  error: "",
};

test("configuration state keeps one routed activity and a selected policy folder", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = setConfigurationRoute(state, "document-defaults");
  state = selectConfigurationFolder(state, "Policies/HR");

  assert.equal(state.route, "document-defaults");
  assert.equal(state.selected_folder, "Policies/HR");
  assert.equal(state.snapshot.workspace.workspace_id, "workspace-1");
  assert.throws(() => setConfigurationRoute(state, "unknown"), /Unknown configuration route/);
});

test("workspace route shows the persistent route navigation and supported local settings", () => {
  const state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  const markup = configurationMarkup(state, assistancePolicy);

  assert.match(markup, /data-configuration-route="workspace"[^>]*aria-current="page"/);
  assert.match(markup, /data-configuration-route="document-defaults"/);
  assert.match(markup, /data-configuration-route="workflow"[^>]*disabled/);
  assert.match(markup, /data-configuration-route="notifications"[^>]*disabled/);
  assert.match(markup, /\/DMS\/Edit/);
  assert.match(markup, /\/DMS\/Publish/);
  assert.match(markup, /data-configuration-form="review-interval"/);
  assert.match(markup, /Claude Desktop assistance policy/);
});

test("document defaults route exposes folder policy and document-type catalogue mutations", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = setConfigurationRoute(state, "document-defaults");
  state = selectConfigurationFolder(state, "Policies/HR");
  const markup = configurationMarkup(state, assistancePolicy);

  assert.match(markup, /Workspace default/);
  assert.match(markup, /Internal/);
  assert.match(markup, /data-configuration-folder="Policies\/HR"[^>]*aria-current="true"/);
  assert.match(markup, /data-configuration-form="confidentiality-policy"/);
  assert.match(markup, /data-configuration-form="remove-confidentiality-policy"/);
  assert.match(markup, /data-configuration-form="document-type"/);
  assert.match(markup, /Create document type/);
  assert.match(markup, /Manage confidentiality types/);
});

test("configuration mutations map forms to narrow desktop commands", () => {
  assert.deepEqual(
    configurationMutationRequest("review-interval", new Map([["months", "6"]]), "."),
    { command: "configure_default_review_interval", arguments: { months: 6 } },
  );
  assert.deepEqual(
    configurationMutationRequest(
      "document-type",
      new Map([["id", " procedure "], ["label", " Procedure "], ["enabled", "on"]]),
      ".",
    ),
    {
      command: "configure_document_type",
      arguments: { id: "procedure", label: "Procedure", enabled: true },
    },
  );
  assert.deepEqual(
    configurationMutationRequest(
      "confidentiality-policy",
      new Map([["typeId", "restricted"]]),
      "Policies/HR",
    ),
    {
      command: "set_confidentiality_policy",
      arguments: { folder: "Policies/HR", typeId: "restricted" },
    },
  );
  assert.deepEqual(
    configurationMutationRequest("remove-confidentiality-policy", new Map(), "Policies/HR"),
    {
      command: "remove_confidentiality_policy",
      arguments: { folder: "Policies/HR" },
    },
  );
});
