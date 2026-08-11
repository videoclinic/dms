import { assistancePolicyMarkup } from "./assistance.mjs";

const ROUTES = [
  ["workspace", "Workspace", "Roots and local metadata"],
  ["document-defaults", "Document defaults", "Classification and catalogues"],
  ["workflow", "Workflow", "People and role routing"],
  ["notifications", "Notifications", "Review and release email"],
];
const AVAILABLE_ROUTES = new Set(["workspace", "document-defaults"]);

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
    selected_folder: ".",
    snapshot: null,
    notice: "",
    error: "",
  };
}

export function applyConfigurationSnapshot(state, snapshot, notice = "") {
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

export function setConfigurationRoute(state, route) {
  if (!ROUTES.some(([id]) => id === route)) {
    throw new Error(`Unknown configuration route: ${route}`);
  }
  return { ...state, route, notice: "", error: "" };
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
  return `<section class="card configuration-card configuration-catalogue"><div class="configuration-card-heading"><div><h3>Document types</h3><p>Add, rename, or disable workspace document types.</p></div><button class="button secondary" type="button" disabled title="Not available in this build">Manage confidentiality types…</button></div>${rows}<form class="configuration-type-row create" data-configuration-form="document-type"><label><span class="visually-hidden">New document type ID</span><input name="id" required pattern="[a-z0-9]+(?:-[a-z0-9]+)*" placeholder="type-id"></label><label><span class="visually-hidden">New document type label</span><input name="label" required placeholder="Display label"></label><label class="configuration-enabled"><input type="checkbox" name="enabled" checked> Enabled</label><button class="button" type="submit">Create document type</button></form></section>`;
}

function documentDefaultsMarkup(state) {
  const snapshot = state.snapshot;
  const root = snapshot.confidentiality_policies.find((policy) => policy.folder === ".");
  const rootType = snapshot.confidentiality_types.find((type) => type.id === root?.type_id);
  const enabledCount = snapshot.confidentiality_types.filter((type) => type.enabled).length;
  return `<section class="configuration-summary"><div><strong>Workspace default</strong><span>${escapeHtml(rootType?.label ?? "Not configured")}</span></div><span class="badge">${enabledCount} enabled confidentiality ${enabledCount === 1 ? "type" : "types"}</span></section><div class="configuration-defaults-grid">${folderTreeMarkup(state)}${selectedPolicyMarkup(state)}</div>${documentTypesMarkup(snapshot)}`;
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
  const body = state.route === "workspace"
    ? workspaceMarkup(state.snapshot, assistancePolicy)
    : state.route === "document-defaults"
      ? documentDefaultsMarkup(state)
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
  throw new Error(`Unknown configuration mutation: ${kind}`);
}
