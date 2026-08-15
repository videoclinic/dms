import { assistancePolicyMarkup } from "./assistance.mjs";

const ROUTES = [
  ["workspace", "Workspace", "Roots and local metadata"],
  ["document-defaults", "Document defaults", "Classification and catalogues"],
  ["workflow", "Workflow", "People and role routing"],
  ["notifications", "Notifications", "Review and release email"],
];
const AVAILABLE_ROUTES = new Set(ROUTES.map(([id]) => id));
const SECONDARY_SURFACES = new Set(["identity-source", "confidentiality-types"]);

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function createConfigurationState() {
  return {
    route: "workspace",
    secondary: null,
    selected_folder: ".",
    snapshot: null,
    identity_setup: null,
    notice: "",
    error: "",
  };
}

export function applyConfigurationSnapshot(state, snapshot, notice = "") {
  if (!snapshot?.workspace) {
    return { ...state, notice: "", error: "" };
  }
  const folders = snapshot?.policy_folders?.map((folder) => folder.relative_path) ?? [];
  const selected = folders.includes(state.selected_folder) ? state.selected_folder : ".";
  return {
    ...state,
    snapshot,
    selected_folder: selected,
    notice,
    error: "",
  };
}

export function applyGlobalEntraConfiguration(state, configuration) {
  return {
    ...state,
    snapshot: {
      ...state.snapshot,
      global_entra_configuration: configuration,
    },
    identity_setup: null,
    notice: "Application Entra configuration saved.",
    error: "",
  };
}

export function setConfigurationRoute(state, route) {
  if (!ROUTES.some(([id]) => id === route)) {
    throw new Error(`Unknown configuration route: ${route}`);
  }
  return { ...state, route, secondary: null, notice: "", error: "" };
}

export function openConfigurationSecondary(state, secondary) {
  if (!SECONDARY_SURFACES.has(secondary)) {
    throw new Error(`Unknown configuration surface: ${secondary}`);
  }
  return { ...state, secondary, notice: "", error: "" };
}

export function closeConfigurationSecondary(state) {
  return { ...state, secondary: null, notice: "", error: "" };
}

export function selectConfigurationFolder(state, folder) {
  const normalized = String(folder ?? "").trim() || ".";
  const exists = state.snapshot?.policy_folders?.some(
    (candidate) => candidate.relative_path === normalized,
  );
  if (!exists) throw new Error(`Unknown configuration folder: ${normalized}`);
  return { ...state, selected_folder: normalized, notice: "", error: "" };
}

function routeNavigation(route) {
  return `<section class="configuration-nav" aria-label="Configuration navigation"><div><h2>Configuration</h2><p>Choose the task that matches the setting you need.</p></div><nav class="configuration-routes">${ROUTES.map(([id, label, description]) => {
    const current = id === route;
    const available = AVAILABLE_ROUTES.has(id);
    return `<button class="configuration-route${current ? " current" : ""}" type="button" data-configuration-route="${id}" ${current ? 'aria-current="page"' : ""} ${available ? "" : 'disabled title="Not available in this build"'}><strong>${label}</strong><span>${description}</span></button>`;
  }).join("")}</nav></section>`;
}

function workspaceMarkup(snapshot, assistancePolicy) {
  const workspace = snapshot.workspace;
  return `<div class="configuration-grid"><section class="card configuration-card"><span class="badge">Workspace</span><h2>Local workspace</h2><p>These roots and the stable workspace identity come from <code>.dms</code>.</p><dl class="details-grid"><dt>Workspace ID</dt><dd>${escapeHtml(workspace.workspace_id)}</dd><dt>Edit root</dt><dd>${escapeHtml(workspace.edit_root)}</dd><dt>Publish root</dt><dd>${escapeHtml(workspace.publish_root)}</dd><dt>Controlled documents</dt><dd>${escapeHtml(workspace.document_count)}</dd></dl><form class="configuration-form" data-configuration-form="review-interval"><label>Default review interval (months)<input name="months" type="number" min="1" required value="${escapeHtml(snapshot.default_review_interval_months)}"></label><button class="button" type="submit">Save review interval</button></form></section>${assistancePolicyMarkup(assistancePolicy)}</div>`;
}

function policyFolderLabel(folder) {
  return folder === "." ? "Edit root" : folder.split("/").at(-1);
}

function folderTreeMarkup(state) {
  return `<section class="card configuration-card"><h3>Choose default or exception</h3><div class="configuration-folder-tree">${state.snapshot.policy_folders.map(({ relative_path: folder }) => {
    const depth = folder === "." ? 0 : folder.split("/").length;
    const current = folder === state.selected_folder;
    return `<button type="button" data-configuration-folder="${escapeHtml(folder)}" aria-current="${current}" class="configuration-folder${current ? " current" : ""}" style="--folder-depth:${depth}"><span aria-hidden="true">▦</span><span><strong>${escapeHtml(policyFolderLabel(folder))}</strong><small>${escapeHtml(folder)}</small></span></button>`;
  }).join("")}</div></section>`;
}

function selectedPolicyMarkup(state) {
  const snapshot = state.snapshot;
  const selected = state.selected_folder;
  const direct = snapshot.confidentiality_policies.find((policy) => policy.folder === selected);
  const enabledTypes = snapshot.confidentiality_types.filter((type) => type.enabled);
  const inherited = [...snapshot.confidentiality_policies]
    .filter((policy) => selected === policy.folder || selected.startsWith(`${policy.folder === "." ? "" : `${policy.folder}/`}`))
    .sort((left, right) => right.folder.length - left.folder.length)[0];
  const effectiveType = snapshot.confidentiality_types.find(
    (type) => type.id === (direct?.type_id ?? inherited?.type_id),
  );
  const options = enabledTypes.map((type) => `<option value="${escapeHtml(type.id)}" ${type.id === (direct?.type_id ?? effectiveType?.id) ? "selected" : ""}>${escapeHtml(type.label)} (${escapeHtml(type.id)})</option>`).join("");
  const canSave = options.length > 0;
  return `<section class="card configuration-card"><span class="badge">Selected folder</span><h3>${escapeHtml(selected === "." ? "Edit root" : selected)}</h3><p>${direct ? `Direct policy: ${escapeHtml(effectiveType?.label ?? direct.type_id)}.` : `Inherited policy: ${escapeHtml(effectiveType?.label ?? "not configured")}.`}</p><form class="configuration-form" data-configuration-form="confidentiality-policy"><label>Confidentiality type<select name="typeId" required ${canSave ? "" : "disabled"}>${options || '<option value="">No enabled types</option>'}</select></label><button class="button" type="submit" ${canSave ? "" : "disabled"}>Save folder policy</button></form>${selected !== "." && direct ? '<form data-configuration-form="remove-confidentiality-policy"><button class="button secondary" type="submit">Remove folder policy</button></form>' : ""}</section>`;
}

function documentTypesMarkup(snapshot) {
  const rows = snapshot.document_types.length === 0
    ? '<p class="subtle">No document types configured.</p>'
    : snapshot.document_types.map((type) => `<form class="configuration-type-row" data-configuration-form="document-type"><input type="hidden" name="id" value="${escapeHtml(type.id)}"><label><span class="visually-hidden">Label for ${escapeHtml(type.id)}</span><input name="label" required value="${escapeHtml(type.label)}"></label><code>${escapeHtml(type.id)}</code><label class="configuration-enabled"><input type="checkbox" name="enabled" ${type.enabled ? "checked" : ""}> Enabled</label><button class="button secondary" type="submit">Save</button></form>`).join("");
  return `<section class="card configuration-card configuration-catalogue"><div class="configuration-card-heading"><div><h3>Document types</h3><p>Add, rename, or disable workspace document types.</p></div><button class="button secondary" type="button" data-configuration-secondary="confidentiality-types">Manage confidentiality types…</button></div>${rows}<form class="configuration-type-row create" data-configuration-form="document-type"><label><span class="visually-hidden">New document type ID</span><input name="id" required pattern="[a-z0-9]+(?:-[a-z0-9]+)*" placeholder="type-id"></label><label><span class="visually-hidden">New document type label</span><input name="label" required placeholder="Display label"></label><label class="configuration-enabled"><input type="checkbox" name="enabled" checked> Enabled</label><button class="button" type="submit">Create document type</button></form></section>`;
}

function documentDefaultsMarkup(state) {
  const snapshot = state.snapshot;
  const root = snapshot.confidentiality_policies.find((policy) => policy.folder === ".");
  const rootType = snapshot.confidentiality_types.find((type) => type.id === root?.type_id);
  const enabledCount = snapshot.confidentiality_types.filter((type) => type.enabled).length;
  return `<section class="configuration-summary"><div><strong>Workspace default</strong><span>${escapeHtml(rootType?.label ?? "Not configured")}</span></div><span class="badge">${enabledCount} enabled confidentiality ${enabledCount === 1 ? "type" : "types"}</span></section><div class="configuration-defaults-grid">${folderTreeMarkup(state)}${selectedPolicyMarkup(state)}</div>${documentTypesMarkup(snapshot)}`;
}

function confidentialityTypesMarkup(state) {
  const snapshot = state.snapshot;
  const rootTypeId = snapshot.confidentiality_policies.find((policy) => policy.folder === ".")?.type_id;
  const rows = snapshot.confidentiality_types.length === 0
    ? '<p class="subtle">No confidentiality types configured.</p>'
    : snapshot.confidentiality_types.map((type) => `<form class="configuration-type-row confidentiality" data-configuration-form="confidentiality-type"><input type="hidden" name="id" value="${escapeHtml(type.id)}"><label><span class="visually-hidden">Label for ${escapeHtml(type.id)}</span><input name="label" required value="${escapeHtml(type.label)}"></label><code>${escapeHtml(type.id)}</code><label class="configuration-enabled"><input type="checkbox" name="enabled" ${type.enabled ? "checked" : ""}> Enabled</label><label class="configuration-enabled"><input type="checkbox" name="workspaceDefault" ${type.id === rootTypeId ? "checked" : ""}> Workspace default</label><button class="button secondary" type="submit">Save</button></form>`).join("");
  const firstType = snapshot.confidentiality_types.length === 0;
  const firstDefault = firstType
    ? '<input type="hidden" name="workspaceDefault" value="on"><label class="configuration-enabled"><input type="checkbox" checked disabled> Workspace default</label>'
    : '<label class="configuration-enabled"><input type="checkbox" name="workspaceDefault"> Workspace default</label>';
  return `<section class="configuration-secondary"><button class="button secondary" type="button" data-configuration-secondary-close>← Back to Document defaults</button><section class="card configuration-card configuration-catalogue"><span class="badge">Secondary configuration</span><h2>Confidentiality types</h2><p>IDs are stable metadata keys. Labels can change; types in use cannot be disabled.</p>${rows}<form class="configuration-type-row confidentiality create" data-configuration-form="confidentiality-type"><label><span class="visually-hidden">New confidentiality type ID</span><input name="id" required pattern="[a-z0-9]+(?:-[a-z0-9]+)*" placeholder="type-id"></label><label><span class="visually-hidden">New confidentiality type label</span><input name="label" required placeholder="Display label"></label><label class="configuration-enabled"><input type="checkbox" name="enabled" checked> Enabled</label>${firstDefault}<button class="button" type="submit">Create confidentiality type</button></form></section></section>`;
}

function roleSelectMarkup(snapshot, policy, roleName, rootFolder) {
  const role = policy?.[roleName];
  const source = snapshot.identity_source;
  const person = snapshot.eligible_people.find((candidate) => candidate.object_id === role?.object_id);
  const resolved = Boolean(person && source?.binding_id === role?.binding_id);
  const options = [];
  if (!rootFolder) {
    options.push(`<option value="__inherit" ${role ? "" : "selected"}>Inherit from parent</option>`);
  } else if (!role) {
    options.push('<option value="" selected disabled>Choose a person</option>');
  }
  if (role && !resolved) {
    options.push(`<option value="__unchanged" selected>Unresolved assignment — keep unchanged</option>`);
  }
  options.push(...snapshot.eligible_people.map((candidate) => `<option value="${escapeHtml(candidate.object_id)}" ${resolved && candidate.object_id === role.object_id ? "selected" : ""}>${escapeHtml(candidate.display_name)} — ${escapeHtml(candidate.email)}</option>`));
  return `<label>${roleName === "editor" ? "Editor" : "Approver"}<select name="${roleName}" required ${source && snapshot.eligible_people.length > 0 ? "" : "disabled"}>${options.join("")}</select></label>`;
}

function identitySourcePreviewMarkup(preview, source) {
  const initialSetup = !source;
  const initialRoleSelect = (roleName) => {
    const fieldName = roleName === "editor" ? "initialEditorId" : "initialApproverId";
    const label = roleName === "editor" ? "Editor" : "Approver";
    const options = [
      '<option value="" selected disabled>Choose a person</option>',
      ...preview.eligible_people.map((person) => `<option value="${escapeHtml(person.object_id)}">${escapeHtml(person.display_name)} — ${escapeHtml(person.email)}</option>`),
    ];
    return `<label>${label}<select name="${fieldName}" required ${preview.eligible_people.length > 0 ? "" : "disabled"}>${options.join("")}</select></label>`;
  };
  const roleMarkup = initialSetup
    ? `<fieldset><legend>Initial edit-root workflow roles</legend><p>Choose the required workspace defaults from this group. They are saved atomically with the identity source.</p>${initialRoleSelect("editor")}${initialRoleSelect("approver")}</fieldset>`
    : "";
  const consequence = initialSetup
    ? "Applying this binding saves the people source and required edit-root roles together."
    : "Applying this binding replaces the current people source, invalidates stale workflow candidates, and leaves existing role references unresolved.";
  const disabled = initialSetup && preview.eligible_people.length === 0 ? "disabled" : "";
  return `<section class="card configuration-card"><h3>Preview identity source</h3><dl class="details-grid"><dt>Tenant</dt><dd>${escapeHtml(preview.tenant_display)}</dd><dt>Group</dt><dd>${escapeHtml(preview.group_label)}</dd><dt>Eligible people</dt><dd>${escapeHtml(preview.eligible_people.length)}</dd></dl><p>${consequence}</p><form class="configuration-form" data-configuration-form="identity-source-apply"><input type="hidden" name="previewId" value="${escapeHtml(preview.preview_id)}">${roleMarkup}<label class="configuration-enabled"><input type="checkbox" name="confirmed" required> I confirm this group is the workspace’s people source.</label><button class="button" type="submit" ${disabled}>Apply identity source</button></form></section>`;
}

function workflowMarkup(state) {
  const snapshot = state.snapshot;
  const source = snapshot.identity_source;
  const selected = state.selected_folder;
  const direct = snapshot.workflow_policies.find((policy) => policy.folder === selected);
  const rootFolder = selected === ".";
  const sourceSummary = source
    ? `<div><strong>${escapeHtml(source.group_label)}</strong><span>${snapshot.eligible_people.length} eligible ${snapshot.eligible_people.length === 1 ? "person" : "people"} · refreshed ${escapeHtml(source.last_refreshed_at ?? "Not yet refreshed")}</span></div>`
    : '<div><strong>Not configured</strong><span>Connect one Microsoft Entra group before assigning roles.</span></div>';
  const editor = roleSelectMarkup(snapshot, direct, "editor", rootFolder);
  const approver = roleSelectMarkup(snapshot, direct, "approver", rootFolder);
  const canSave = Boolean(source && snapshot.eligible_people.length > 0);
  return `<section class="configuration-summary"><div><strong>People source</strong><span>One direct-user Microsoft Entra group</span></div>${sourceSummary}<button class="button secondary" type="button" data-configuration-secondary="identity-source">Manage identity source…</button></section><div class="configuration-defaults-grid">${folderTreeMarkup(state)}<section class="card configuration-card"><span class="badge">Selected folder</span><h3>${escapeHtml(rootFolder ? "Edit root" : selected)}</h3><p>${direct ? "Direct workflow role assignment." : rootFolder ? "Root roles are required after an identity source is connected." : "Editor and approver inherit independently from the nearest parent assignment."}</p><form class="configuration-form" data-configuration-form="workflow-policy">${editor}${approver}<button class="button" type="submit" ${canSave ? "" : "disabled"}>Save workflow roles</button></form>${!rootFolder && direct ? '<form data-configuration-form="remove-workflow-policy"><button class="button secondary" type="submit">Remove folder exception</button></form>' : ""}</section></div>`;
}

function identitySourceMarkup(state) {
  const source = state.snapshot.identity_source;
  const people = state.snapshot.eligible_people;
  const setup = state.identity_setup;
  const global = state.snapshot.global_entra_configuration;
  const groupPageUrl = source
    ? `https://myaccount.microsoft.com/groups/${encodeURIComponent(source.group_id)}`
    : "";
  const details = source
    ? `<dl class="details-grid"><dt>Public client ID</dt><dd><code>${escapeHtml(global?.client_id ?? "Not configured")}</code></dd><dt>Tenant ID</dt><dd><code>${escapeHtml(global?.tenant_id ?? "Not configured")}</code></dd><dt>Group</dt><dd>${escapeHtml(source.group_label)}</dd><dt>Group ID</dt><dd><button class="button secondary" type="button" data-open-external="${escapeHtml(groupPageUrl)}" aria-label="Open Microsoft 365 group page for Group ID ${escapeHtml(source.group_id)}"><code>${escapeHtml(source.group_id)}</code></button></dd><dt>Last refresh</dt><dd>${escapeHtml(source.last_refreshed_at ?? "Not yet refreshed")}</dd></dl>`
    : '<p class="status">No Microsoft Entra identity source is configured.</p>';
  const rows = people.length === 0
    ? '<p class="subtle">No eligible people are cached.</p>'
    : `<div class="configuration-people">${people.map((person) => `<div><strong>${escapeHtml(person.display_name)}</strong><span>${escapeHtml(person.email)}</span><code>${escapeHtml(person.object_id)}</code></div>`).join("")}</div>`;
  const failedChallenge = Boolean(setup?.challenge && state.error && setup.last_group_id);
  const setupMarkup = setup?.challenge
    ? failedChallenge
      ? `<section class="card configuration-card"><h3>Previous sign-in failed</h3><p>Generate a new Microsoft Entra device code before continuing.</p><form class="configuration-form" data-configuration-form="identity-source-start"><input type="hidden" name="groupId" value="${escapeHtml(setup.last_group_id)}"><button class="button secondary" type="submit">Sign in again</button></form></section>`
      : `<section class="card configuration-card"><h3>Complete Microsoft Entra sign-in</h3><p>${escapeHtml(setup.challenge.message)}</p><dl class="details-grid"><dt>Code</dt><dd><code>${escapeHtml(setup.challenge.user_code)}</code></dd><dt>Sign-in page</dt><dd><button class="button secondary" type="button" data-open-external="${escapeHtml(setup.challenge.verification_uri)}">Open sign-in page</button></dd></dl><form class="configuration-form" data-configuration-form="identity-source-complete"><input type="hidden" name="challengeId" value="${escapeHtml(setup.challenge.challenge_id)}"><button class="button" type="submit">I have signed in — preview group</button></form></section>`
    : setup?.preview
      ? identitySourcePreviewMarkup(setup.preview, source)
      : `<section class="card configuration-card"><h3>${source ? "Replace identity source" : "Set up identity source"}</h3><p>Enter one direct-user group. Sign-in uses delegated Microsoft Graph access.</p><form class="configuration-form" data-configuration-form="identity-source-start"><label>Library Entra group ID<input name="groupId" required placeholder="00000000-0000-0000-0000-000000000000"></label><button class="button" type="submit">Sign in and preview group</button></form></section>`;
  const globalMarkup = `<section class="card configuration-card"><h3>Application Entra configuration</h3><p>Shared by local libraries for this OS user; not stored in <code>.dms</code>.</p><form class="configuration-form" data-configuration-form="global-entra"><label>Public client ID<input name="clientId" value="${escapeHtml(global?.client_id ?? "")}" ${global?.client_id_environment_managed ? "readonly" : ""}></label><label>Tenant ID<input name="tenantId" value="${escapeHtml(global?.tenant_id ?? "")}" ${global?.tenant_id_environment_managed ? "readonly" : ""}></label><button class="button" type="submit">Save application configuration</button></form></section>`;
  return `<section class="configuration-secondary"><button class="button secondary" type="button" data-configuration-secondary-close>← Back to Workflow</button><div class="configuration-grid">${globalMarkup}<section class="card configuration-card"><span class="badge">Secondary configuration</span><h2>Microsoft Entra identity source</h2>${details}</section>${setupMarkup}<section class="card configuration-card"><h3>Eligible people — read only</h3><p>Only direct, enabled user members returned by Microsoft Graph can be assigned.</p>${rows}<form data-configuration-form="identity-source-refresh"><button class="button secondary" type="submit" ${source ? "" : "disabled"}>Refresh people</button></form></section></div></section>`;
}

function notificationsMarkup(snapshot) {
  const settings = snapshot.notification_settings;
  const transport = settings?.transport ?? "mailto";
  const smtp = settings?.smtp;
  return `<section class="card configuration-card configuration-notifications"><span class="badge">Workspace notification transport</span><h2>Review and release email</h2><p>Choose one transport for workflow notices. Delivery credentials stay in the OS credential store and are never written to <code>.dms</code>.</p><form class="configuration-form configuration-notification-form" data-configuration-form="notifications"><label>Transport<select name="transport" required><option value="smtp" ${transport === "smtp" ? "selected" : ""}>SMTP relay</option><option value="mailto" ${transport === "mailto" ? "selected" : ""}>Host mail app (mailto)</option></select></label><div class="configuration-grid"><label>SMTP relay host<input name="relayHost" value="${escapeHtml(smtp?.relay_host ?? "")}" placeholder="smtp.example.com"></label><label>SMTP relay port<input name="relayPort" type="number" min="1" max="65535" value="${escapeHtml(smtp?.relay_port ?? 587)}"></label><label>Sender address<input name="sender" type="email" value="${escapeHtml(smtp?.sender ?? "")}" placeholder="dms@example.com"></label><label>Microsoft 365 app password<input name="smtpAppPassword" type="password" autocomplete="new-password"></label></div><p class="subtle">${snapshot.smtp_credential_configured ? "Credential configured. Leave the password blank to retain it." : "No credential configured."}</p><button class="button" type="submit">Save notification transport</button></form></section>`;
}

function unavailableRouteMarkup(route) {
  const label = ROUTES.find(([id]) => id === route)?.[1] ?? "Configuration";
  return `<section class="card configuration-card"><span class="badge">Unavailable</span><h2>${escapeHtml(label)}</h2><p>This configuration route is not available in this build.</p></section>`;
}

export function configurationMarkup(state, assistancePolicy) {
  const navigation = routeNavigation(state.route);
  if (!state.snapshot) {
    return `${navigation}<section class="card configuration-card"><p>${escapeHtml(state.error || "Loading workspace configuration…")}</p></section>`;
  }
  const status = state.error
    ? `<p class="status" role="alert">${escapeHtml(state.error)}</p>`
    : state.notice
      ? `<p class="status success">${escapeHtml(state.notice)}</p>`
      : "";
  const body = state.secondary === "identity-source"
    ? identitySourceMarkup(state)
    : state.secondary === "confidentiality-types"
      ? confidentialityTypesMarkup(state)
      : state.route === "workspace"
        ? workspaceMarkup(state.snapshot, assistancePolicy)
        : state.route === "document-defaults"
          ? documentDefaultsMarkup(state)
          : state.route === "workflow"
            ? workflowMarkup(state)
            : state.route === "notifications"
              ? notificationsMarkup(state.snapshot)
              : unavailableRouteMarkup(state.route);
  return `<section class="configuration-workspace">${navigation}${status}${body}</section>`;
}

function formValue(values, name) {
  return String(values.get(name) ?? "").trim();
}

export function configurationMutationRequest(kind, values, selectedFolder) {
  if (kind === "review-interval") {
    return {
      command: "configure_default_review_interval",
      arguments: { months: Number(formValue(values, "months")) },
    };
  }
  if (kind === "document-type") {
    return {
      command: "configure_document_type",
      arguments: {
        id: formValue(values, "id"),
        label: formValue(values, "label"),
        enabled: values.has("enabled"),
      },
    };
  }
  if (kind === "confidentiality-policy") {
    return {
      command: "set_confidentiality_policy",
      arguments: { folder: selectedFolder, typeId: formValue(values, "typeId") },
    };
  }
  if (kind === "remove-confidentiality-policy") {
    return {
      command: "remove_confidentiality_policy",
      arguments: { folder: selectedFolder },
    };
  }
  if (kind === "workflow-policy") {
    return {
      command: "set_workflow_policy",
      arguments: {
        folder: selectedFolder,
        editor: formValue(values, "editor"),
        approver: formValue(values, "approver"),
      },
    };
  }
  if (kind === "remove-workflow-policy") {
    return {
      command: "set_workflow_policy",
      arguments: { folder: selectedFolder, editor: "__inherit", approver: "__inherit" },
    };
  }
  if (kind === "notifications") {
    return {
      command: "configure_notifications",
      arguments: {
        transport: formValue(values, "transport"),
        relayHost: formValue(values, "relayHost"),
        relayPort: Number(formValue(values, "relayPort")),
        sender: formValue(values, "sender"),
        smtpAppPassword: formValue(values, "smtpAppPassword"),
      },
    };
  }
  if (kind === "identity-source-start") {
    return {
      command: "begin_identity_source_sign_in",
      arguments: { groupId: formValue(values, "groupId") },
    };
  }
  if (kind === "global-entra") {
    return {
      command: "configure_global_entra",
      arguments: { clientId: formValue(values, "clientId"), tenantId: formValue(values, "tenantId") },
    };
  }
  if (kind === "identity-source-complete") {
    return {
      command: "complete_identity_source_sign_in",
      arguments: { challengeId: formValue(values, "challengeId") },
    };
  }
  if (kind === "identity-source-apply") {
    return {
      command: "apply_identity_source",
      arguments: {
        previewId: formValue(values, "previewId"),
        initialEditorId: formValue(values, "initialEditorId") || null,
        initialApproverId: formValue(values, "initialApproverId") || null,
        confirmed: values.has("confirmed"),
      },
    };
  }
  if (kind === "identity-source-refresh") {
    return { command: "refresh_identity_source", arguments: {} };
  }
  if (kind === "confidentiality-type") {
    return {
      command: "configure_confidentiality_type",
      arguments: {
        id: formValue(values, "id"),
        label: formValue(values, "label"),
        enabled: values.has("enabled"),
        workspaceDefault: values.has("workspaceDefault"),
      },
    };
  }
  throw new Error(`Unknown configuration mutation: ${kind}`);
}
