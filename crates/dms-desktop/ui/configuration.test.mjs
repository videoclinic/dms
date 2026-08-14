import test from "node:test";
import assert from "node:assert/strict";

import {
  applyConfigurationSnapshot,
  applyGlobalEntraConfiguration,
  closeConfigurationSecondary,
  configurationMarkup,
  configurationMutationRequest,
  createConfigurationState,
  openConfigurationSecondary,
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
  identity_source: {
    binding_id: "binding-1",
    tenant_id: "tenant-1",
    tenant_display: "Example tenant",
    group_id: "group-1",
    group_label: "DMS workflow",
    last_refreshed_at: "2026-08-11T12:00:00Z",
  },
  eligible_people: [
    { object_id: "editor-1", display_name: "Lukas Roth", email: "lukas@example.test", account_enabled: true },
    { object_id: "approver-1", display_name: "Anna Berg", email: "anna@example.test", account_enabled: true },
  ],
  workflow_policies: [
    { folder: ".", editor: { binding_id: "binding-1", object_id: "editor-1" }, approver: { binding_id: "binding-1", object_id: "approver-1" } },
  ],
  notification_settings: {
    transport: "smtp",
    smtp: { relay_host: "smtp.example.test", relay_port: 587, sender: "dms@example.test" },
  },
  global_entra_configuration: {
    client_id: "client-1",
    tenant_id: "tenant-1",
    client_id_environment_managed: false,
    tenant_id_environment_managed: true,
  },
  smtp_credential_configured: true,
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

test("configuration snapshot rejects a global Entra result without a workspace snapshot", () => {
  const savedGlobalEntra = {
    client_id: "client-2",
    tenant_id: "tenant-2",
    client_id_environment_managed: false,
    tenant_id_environment_managed: false,
  };
  const initial = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  const saved = applyConfigurationSnapshot(
    initial,
    { ...snapshot, global_entra_configuration: savedGlobalEntra },
    "Application Entra configuration saved.",
  );

  assert.equal(saved.notice, "Application Entra configuration saved.");
  assert.deepEqual(saved.snapshot.workspace, workspace);
  assert.deepEqual(saved.snapshot.global_entra_configuration, savedGlobalEntra);

  const rawResult = applyConfigurationSnapshot(
    saved,
    savedGlobalEntra,
    "Application Entra configuration saved.",
  );
  assert.deepEqual(rawResult.snapshot, saved.snapshot);
  assert.equal(rawResult.notice, "");
  assert.equal(rawResult.error, "");
});

test("saving global Entra configuration clears backend-invalidated setup state", () => {
  const savedGlobalEntra = {
    client_id: "client-2",
    tenant_id: "tenant-2",
    client_id_environment_managed: false,
    tenant_id_environment_managed: false,
  };
  const initial = {
    ...applyConfigurationSnapshot(createConfigurationState(), snapshot),
    identity_setup: { preview: { preview_id: "stale-preview" } },
    error: "old error",
  };

  const saved = applyGlobalEntraConfiguration(initial, savedGlobalEntra);

  assert.deepEqual(saved.snapshot.workspace, workspace);
  assert.deepEqual(saved.snapshot.global_entra_configuration, savedGlobalEntra);
  assert.equal(saved.identity_setup, null);
  assert.equal(saved.notice, "Application Entra configuration saved.");
  assert.equal(saved.error, "");
});

test("workspace route shows the persistent route navigation and supported local settings", () => {
  const state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  const markup = configurationMarkup(state, assistancePolicy);

  assert.match(markup, /data-configuration-route="workspace"[^>]*aria-current="page"/);
  assert.match(markup, /data-configuration-route="document-defaults"/);
  assert.doesNotMatch(markup, /data-configuration-route="workflow"[^>]*disabled/);
  assert.doesNotMatch(markup, /data-configuration-route="notifications"[^>]*disabled/);
  assert.match(markup, /\/DMS\/Edit/);
  assert.match(markup, /\/DMS\/Publish/);
  assert.match(markup, /data-configuration-form="review-interval"/);
  assert.match(markup, /Claude Desktop assistance policy/);
});

test("workflow route configures folder roles and opens identity source in place", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = setConfigurationRoute(state, "workflow");
  let markup = configurationMarkup(state, assistancePolicy);

  assert.match(markup, /People source/);
  assert.match(markup, /DMS workflow/);
  assert.match(markup, /refreshed 2026-08-11T12:00:00Z/);
  assert.match(markup, /data-configuration-form="workflow-policy"/);
  assert.match(markup, /Lukas Roth/);
  assert.match(markup, /Anna Berg/);
  assert.match(markup, /data-configuration-secondary="identity-source"/);

  state = openConfigurationSecondary(state, "identity-source");
  markup = configurationMarkup(state, assistancePolicy);
  assert.match(markup, /Back to Workflow/);
  assert.match(markup, /Eligible people — read only/);
  assert.match(markup, /identity-source-start/);
  assert.match(markup, /delegated Microsoft Graph access/);
  assert.match(markup, /Application Entra configuration/);
  assert.match(markup, /name="tenantId"[^>]*readonly/);
  assert.match(markup, /Library Entra group/);
  assert.equal(closeConfigurationSecondary(state).secondary, null);
});

test("identity-source challenge opens the host browser without WebView navigation", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = openConfigurationSecondary(state, "identity-source");
  state = {
    ...state,
    identity_setup: {
      challenge: {
        challenge_id: "challenge-1",
        message: "Use the supplied code to sign in.",
        user_code: "ABCD-EFGH",
        verification_uri: "https://microsoft.com/devicelogin",
      },
    },
  };

  const markup = configurationMarkup(state, assistancePolicy);
  assert.match(markup, /data-open-external="https:\/\/microsoft\.com\/devicelogin"/);
  assert.match(markup, /Open sign-in page/);
  assert.doesNotMatch(markup, /target="_blank"/);
});

test("failed identity-source challenge offers a same-surface restart with the last group", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = openConfigurationSecondary(state, "identity-source");
  state = {
    ...state,
    error: "Microsoft Entra sign-in challenge is no longer available; start again",
    identity_setup: {
      challenge: {
        challenge_id: "challenge-1",
        message: "Use the supplied code to sign in.",
        user_code: "ABCD-EFGH",
        verification_uri: "https://microsoft.com/devicelogin",
      },
      last_group_id: "00000000-0000-0000-0000-000000000000",
    },
  };

  const failedMarkup = configurationMarkup(state, assistancePolicy);
  assert.match(failedMarkup, /Previous sign-in failed/);
  assert.match(failedMarkup, /data-configuration-form="identity-source-start"/);
  assert.match(failedMarkup, /name="groupId" value="00000000-0000-0000-0000-000000000000"/);
  assert.match(failedMarkup, /Sign in again/);
  assert.doesNotMatch(failedMarkup, /I have signed in — preview group/);
  assert.doesNotMatch(failedMarkup, /data-configuration-form="identity-source-complete"/);
  assert.doesNotMatch(failedMarkup, /ABCD-EFGH/);

  const activeMarkup = configurationMarkup({ ...state, error: "" }, assistancePolicy);
  assert.doesNotMatch(activeMarkup, /Sign in again/);
  assert.doesNotMatch(activeMarkup, /Previous sign-in failed/);
});

test("first identity-source preview requires initial edit-root roles", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), {
    ...snapshot,
    identity_source: null,
    eligible_people: [],
    workflow_policies: [],
  });
  state = openConfigurationSecondary(state, "identity-source");
  state = {
    ...state,
    identity_setup: {
      preview: {
        preview_id: "preview-1",
        tenant_display: "Example tenant",
        group_label: "DMS workflow",
        eligible_people: snapshot.eligible_people,
      },
    },
  };

  const markup = configurationMarkup(state, assistancePolicy);
  assert.match(markup, /Initial edit-root workflow roles/);
  assert.match(markup, /name="initialEditorId" required/);
  assert.match(markup, /name="initialApproverId" required/);
  assert.match(markup, /Lukas Roth/);
  assert.match(markup, /Anna Berg/);
});

test("replacement identity-source preview leaves existing roles unresolved", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = openConfigurationSecondary(state, "identity-source");
  state = {
    ...state,
    identity_setup: {
      preview: {
        preview_id: "preview-2",
        tenant_display: "Example tenant",
        group_label: "Replacement workflow",
        eligible_people: snapshot.eligible_people,
      },
    },
  };

  const markup = configurationMarkup(state, assistancePolicy);
  assert.match(markup, /leaves existing role references unresolved/);
  assert.doesNotMatch(markup, /Initial edit-root workflow roles/);
  assert.doesNotMatch(markup, /name="initialEditorId"/);
  assert.doesNotMatch(markup, /name="initialApproverId"/);
});

test("notifications route submits a write-only SMTP app password without rendering it", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = setConfigurationRoute(state, "notifications");
  const markup = configurationMarkup(state, assistancePolicy);

  assert.match(markup, /data-configuration-form="notifications"/);
  assert.match(markup, /smtp\.example\.test/);
  assert.match(markup, /OS credential store/);
  assert.match(markup, /name="smtpAppPassword"/);
  assert.match(markup, /type="password"/);
  assert.match(markup, /Credential configured/);
});

test("confidentiality catalogue is a secondary surface that returns to document defaults", () => {
  let state = applyConfigurationSnapshot(createConfigurationState(), snapshot);
  state = setConfigurationRoute(state, "document-defaults");
  state = openConfigurationSecondary(state, "confidentiality-types");
  const markup = configurationMarkup(state, assistancePolicy);

  assert.match(markup, /Back to Document defaults/);
  assert.match(markup, /data-configuration-form="confidentiality-type"/);
  assert.match(markup, /Create confidentiality type/);
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
  assert.deepEqual(
    configurationMutationRequest(
      "workflow-policy",
      new Map([["editor", "editor-1"], ["approver", "__inherit"]]),
      "Policies/HR",
    ),
    {
      command: "set_workflow_policy",
      arguments: { folder: "Policies/HR", editor: "editor-1", approver: "__inherit" },
    },
  );
  assert.deepEqual(
    configurationMutationRequest(
      "identity-source-start",
      new Map([["groupId", " group "]]),
      ".",
    ),
    {
      command: "begin_identity_source_sign_in",
      arguments: { groupId: "group" },
    },
  );
  assert.deepEqual(
    configurationMutationRequest(
      "global-entra",
      new Map([["clientId", " client "], ["tenantId", " tenant "]]),
      ".",
    ),
    {
      command: "configure_global_entra",
      arguments: { clientId: "client", tenantId: "tenant" },
    },
  );
  assert.deepEqual(
    configurationMutationRequest(
      "identity-source-apply",
      new Map([
        ["previewId", " preview "],
        ["initialEditorId", " editor-1 "],
        ["initialApproverId", " approver-1 "],
        ["confirmed", "on"],
      ]),
      ".",
    ),
    {
      command: "apply_identity_source",
      arguments: {
        previewId: "preview",
        initialEditorId: "editor-1",
        initialApproverId: "approver-1",
        confirmed: true,
      },
    },
  );
  assert.deepEqual(
    configurationMutationRequest(
      "notifications",
      new Map([
        ["transport", "smtp"],
        ["relayHost", " smtp.example.test "],
        ["relayPort", "587"],
        ["sender", " dms@example.test "],
        ["smtpAppPassword", " one-way-secret "],
      ]),
      ".",
    ),
    {
      command: "configure_notifications",
      arguments: {
        transport: "smtp",
        relayHost: "smtp.example.test",
        relayPort: 587,
        sender: "dms@example.test",
        smtpAppPassword: "one-way-secret",
      },
    },
  );
  assert.deepEqual(
    configurationMutationRequest(
      "confidentiality-type",
      new Map([["id", " restricted "], ["label", " Restricted "], ["enabled", "on"]]),
      ".",
    ),
    {
      command: "configure_confidentiality_type",
      arguments: {
        id: "restricted",
        label: "Restricted",
        enabled: true,
        workspaceDefault: false,
      },
    },
  );
});
