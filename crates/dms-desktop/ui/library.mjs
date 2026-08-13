export function normalizeLibraryPath(path) {
  const normalized = String(path ?? ".").replaceAll("\\", "/").replace(/^\/+|\/+$/g, "");
  return normalized || ".";
}

export function createLibraryState() {
  return {
    tree: [],
    folder: { relative_path: ".", parent: null, entries: [] },
    selection: [],
    detail: null,
    detail_error: "",
    evidence_open: false,
    lifecycle_drafts: {},
    results: null,
    query: "",
    entire_library: false,
    sort: "name",
    page_size: 25,
    page: 0,
    back: [],
    forward: [],
    expanded_folders: ["."],
    loading: false,
  };
}

function folderExpansionPath(path) {
  const normalized = normalizeLibraryPath(path);
  const expanded = ["."];
  if (normalized === ".") return expanded;
  let current = "";
  for (const component of normalized.split("/")) {
    current = current ? `${current}/${component}` : component;
    expanded.push(current);
  }
  return expanded;
}

export function toggleTreeFolder(library, relativePath) {
  const path = normalizeLibraryPath(relativePath);
  const expanded = new Set(library.expanded_folders ?? ["."]);
  if (expanded.has(path)) {
    expanded.delete(path);
  } else {
    expanded.add(path);
  }
  return { ...library, expanded_folders: [...expanded] };
}

export function buildFolderTree(folders) {
  const nodes = new Map((folders ?? []).map((folder) => {
    const path = normalizeLibraryPath(folder.relative_path);
    return [path, { name: folder.name, path, children: [] }];
  }));
  const roots = [];
  for (const node of nodes.values()) {
    if (node.path === ".") {
      roots.push(node);
      continue;
    }
    const components = node.path.split("/");
    const parentPath = components.length === 1 ? "." : components.slice(0, -1).join("/");
    const parent = nodes.get(parentPath);
    if (parent) {
      parent.children.push(node);
    } else {
      roots.push(node);
    }
  }
  const sort = (nodes_) => {
    nodes_.sort((left, right) => left.name.localeCompare(right.name, undefined, { sensitivity: "base", numeric: true }));
    nodes_.forEach((node) => sort(node.children));
  };
  sort(roots);
  return roots;
}

export function applyLibrarySnapshot(library, snapshot, target, historyMode = "push") {
  const current = normalizeLibraryPath(library.folder?.relative_path);
  const next = normalizeLibraryPath(target);
  const expanded = new Set(library.expanded_folders ?? ["."]);
  folderExpansionPath(next).forEach((path) => expanded.add(path));
  let back = [...library.back];
  let forward = [...library.forward];
  if (historyMode === "push" && current !== next) {
    back.push(current);
    forward = [];
  } else if (historyMode === "back") {
    back.pop();
    forward.push(current);
  } else if (historyMode === "forward") {
    forward.pop();
    back.push(current);
  }
  return {
    ...library,
    tree: snapshot.tree ?? [],
    folder: snapshot.folder,
    selection: [],
    detail: null,
    detail_error: "",
    evidence_open: false,
    lifecycle_drafts: {},
    results: null,
    query: "",
    page: 0,
    back,
    forward,
    expanded_folders: [...expanded],
    loading: false,
  };
}

export function historyTarget(library, direction) {
  const history = direction === "back" ? library.back : library.forward;
  return history.at(-1) ?? null;
}

export function membershipKind(entry) {
  if (typeof entry?.membership === "string") return entry.membership;
  if (entry?.membership?.in_library) return "in_library";
  return null;
}

export function entryDocumentId(entry) {
  return entry?.membership?.in_library?.document_id ?? entry?.document?.id ?? null;
}

export function toggleLibrarySelection(library, relativePath, additive = false) {
  const path = normalizeLibraryPath(relativePath);
  const selected = library.selection.includes(path);
  const selection = additive
    ? (selected ? library.selection.filter((candidate) => candidate !== path) : [...library.selection, path])
    : (selected && library.selection.length === 1 ? [] : [path]);
  return {
    ...library,
    selection,
    detail: null,
    detail_error: "",
    evidence_open: false,
    lifecycle_drafts: {},
  };
}

export function selectedEntries(library) {
  const entries = library.results ?? library.folder?.entries ?? [];
  return library.selection
    .map((path) => entries.find((entry) => normalizeLibraryPath(entry.relative_path) === path))
    .filter(Boolean);
}

export function libraryOpenRequest(detail, target) {
  const command = {
    source: "open_document_source",
    release: "open_current_release_pdf",
  }[target];
  if (!command || !detail?.document_id) throw new Error("A document file target is required.");
  return { command, arguments: { documentId: detail.document_id } };
}

function validIsoDate(value) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const parsed = new Date(`${value}T00:00:00Z`);
  return !Number.isNaN(parsed.valueOf()) && parsed.toISOString().slice(0, 10) === value;
}

export function documentControlUpdateRequest(values, detail) {
  if (!detail?.document_id) throw new Error("A selected document is required.");
  const title = String(values.get("title") ?? "").trim();
  const effectiveDate = String(values.get("effectiveDate") ?? "").trim();
  if (!title) throw new Error("Document title cannot be empty.");
  if (effectiveDate && !validIsoDate(effectiveDate)) {
    throw new Error("Effective date must use YYYY-MM-DD.");
  }
  return {
    command: "update_document_control",
    arguments: {
      documentId: detail.document_id,
      title,
      documentNumber: String(values.get("documentNumber") ?? "").trim(),
      documentType: String(values.get("documentType") ?? "").trim(),
      owner: String(values.get("owner") ?? "").trim(),
      effectiveDate,
    },
  };
}

export function confidentialityUpdateRequest(values, detail) {
  if (!detail?.document_id) throw new Error("A selected document is required.");
  return {
    command: "set_document_confidentiality",
    arguments: {
      documentId: detail.document_id,
      confidentialityTypeId: String(values.get("confidentialityTypeId") ?? "").trim(),
    },
  };
}

export function lifecycleActionRequest(action, values, detail) {
  if (!detail?.document_id) throw new Error("A selected document is required.");
  if (action === "begin_revision") {
    return {
      command: "begin_document_revision",
      arguments: { documentId: detail.document_id },
    };
  }
  if (action === "submit_candidate") {
    const targetMode = String(values?.get("targetMode") ?? "").trim();
    const requesterObjectId = String(values?.get("requesterObjectId") ?? "").trim();
    const changelog = String(values?.get("changelog") ?? "").trim();
    const manualMajor = String(values?.get("manualMajor") ?? "").trim();
    const manualMinor = String(values?.get("manualMinor") ?? "").trim();
    if (!["next_minor", "next_major", "manual"].includes(targetMode)) {
      throw new Error("Choose a target version.");
    }
    if (!requesterObjectId) throw new Error("Choose the requesting editor.");
    if (!changelog) throw new Error("A release changelog is required.");
    if (targetMode === "manual" && (!/^\d+$/.test(manualMajor) || !/^\d+$/.test(manualMinor))) {
      throw new Error("Manual target version needs whole major and minor numbers.");
    }
    return {
      command: "submit_document_candidate",
      arguments: {
        input: {
          documentId: detail.document_id,
          targetMode,
          manualMajor: targetMode === "manual" ? Number(manualMajor) : null,
          manualMinor: targetMode === "manual" ? Number(manualMinor) : null,
          changelog,
          requesterObjectId,
          reviewOverrideReason: String(values?.get("reviewOverrideReason") ?? "").trim(),
          mailtoConfirmed: false,
        },
      },
    };
  }
  if (action === "retry_review_notification") {
    if (values?.get("mailtoConfirmed") !== "yes") {
      throw new Error("Confirm that the host mail message was sent.");
    }
    return {
      command: "retry_review_notification",
      arguments: { documentId: detail.document_id, mailtoConfirmed: true },
    };
  }
  if (action === "decide_review") {
    const decision = String(values?.get("decision") ?? "").trim();
    if (!["approved", "rejected", "changes_requested"].includes(decision)) {
      throw new Error("Choose an approval decision.");
    }
    return {
      command: "decide_document_review",
      arguments: {
        documentId: detail.document_id,
        decision,
        comment: String(values?.get("comment") ?? "").trim(),
        mailtoConfirmed: false,
      },
    };
  }
  if (action === "release_candidate") {
    return {
      command: "release_document_candidate",
      arguments: {
        documentId: detail.document_id,
        releaseOverrideReason: String(values?.get("releaseOverrideReason") ?? "").trim(),
        mailtoConfirmed: false,
      },
    };
  }
  if (action === "retry_decision_notification") {
    if (values?.get("mailtoConfirmed") !== "yes") {
      throw new Error("Confirm that the host mail message was sent.");
    }
    const candidateId = detail.retryable_decision_candidate?.id;
    if (!candidateId) throw new Error("No decision notification is awaiting confirmation.");
    return {
      command: "retry_decision_notification",
      arguments: { documentId: detail.document_id, candidateId, mailtoConfirmed: true },
    };
  }
  if (action === "retry_minor_publication_notification") {
    if (values?.get("mailtoConfirmed") !== "yes") {
      throw new Error("Confirm that the host mail message was sent.");
    }
    const releaseId = detail.retryable_minor_publication?.release_id;
    if (!releaseId) throw new Error("No minor-publication notification is awaiting confirmation.");
    return {
      command: "retry_minor_publication_notification",
      arguments: { documentId: detail.document_id, releaseId, mailtoConfirmed: true },
    };
  }
  const reason = String(values?.get("reason") ?? "").trim();
  const confirmed = values?.get("confirmed") === "yes";
  if (!reason) throw new Error("A reason is required.");
  if (!confirmed) throw new Error("Explicit confirmation is required.");
  const command = {
    cancel_review: "cancel_document_review",
    mark_obsolete: "mark_document_obsolete",
  }[action];
  if (!command) throw new Error(`Unsupported lifecycle action: ${action}`);
  return {
    command,
    arguments: { documentId: detail.document_id, reason, confirmed },
  };
}

export function applyDocumentSelection(library, detail, openEvidence = false) {
  const updateEntry = (entry) => entryDocumentId(entry) === detail.document_id
    ? { ...entry, document: { ...entry.document, lifecycle: detail.lifecycle, control: detail.control } }
    : entry;
  return {
    ...library,
    detail,
    detail_error: "",
    evidence_open: openEvidence,
    lifecycle_drafts: {},
    folder: { ...library.folder, entries: (library.folder?.entries ?? []).map(updateEntry) },
    results: library.results?.map(updateEntry) ?? null,
  };
}

export function breadcrumbSegments(path, rootLabel = "Library") {
  const normalized = normalizeLibraryPath(path);
  const segments = [{ label: rootLabel, path: "." }];
  if (normalized === ".") return segments;
  let current = "";
  for (const component of normalized.split("/")) {
    current = current ? `${current}/${component}` : component;
    segments.push({ label: component, path: current });
  }
  return segments;
}

export function sortLibraryEntries(entries, sort) {
  const value = (entry) => {
    if (sort === "title") return entry.document?.control?.title ?? "";
    if (sort === "number") return entry.document?.control?.document_number ?? "";
    if (sort === "lifecycle") return entry.document?.lifecycle ?? "";
    return entry.name ?? "";
  };
  return [...entries].sort((left, right) => {
    if (left.kind !== right.kind) return left.kind === "folder" ? -1 : 1;
    return value(left).localeCompare(value(right), undefined, { sensitivity: "base", numeric: true });
  });
}

export function paginateLibraryEntries(entries, pageSize, requestedPage) {
  const size = [10, 25, 50, 100].includes(Number(pageSize)) ? Number(pageSize) : 25;
  const pageCount = Math.max(1, Math.ceil(entries.length / size));
  const page = Math.min(Math.max(0, Number(requestedPage) || 0), pageCount - 1);
  return {
    entries: entries.slice(page * size, (page + 1) * size),
    page,
    page_count: pageCount,
    total: entries.length,
  };
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function membershipLabel(entry) {
  const membership = membershipKind(entry);
  if (membership === "in_library") return "In library";
  if (membership === "not_in_library") return "Not in library";
  if (membership === "unsupported") return "Unsupported draft";
  return "Folder";
}

function treeMarkup(tree, currentPath, expandedFolders) {
  const expanded = new Set(expandedFolders ?? ["."]);
  const nodeMarkup = (node, level) => {
    const hasChildren = node.children.length > 0;
    const isExpanded = expanded.has(node.path);
    const current = node.path === currentPath;
    const branchState = hasChildren ? ` aria-expanded="${isExpanded}"` : "";
    const toggle = hasChildren
      ? `<button class="tree-toggle" type="button" data-library-tree-toggle="${escapeHtml(node.path)}" aria-expanded="${isExpanded}" aria-label="${isExpanded ? "Collapse" : "Expand"} ${escapeHtml(node.name)}"><span aria-hidden="true">${isExpanded ? "▾" : "▸"}</span></button>`
      : '<span class="tree-toggle-spacer" aria-hidden="true"></span>';
    const children = hasChildren
      ? `<ul class="tree-group" role="group"${isExpanded ? "" : " hidden"}>${node.children.map((child) => nodeMarkup(child, level + 1)).join("")}</ul>`
      : "";
    return `<li class="tree-item${current ? " current" : ""}" role="treeitem" aria-level="${level}"${current ? ' aria-current="page"' : ""}${branchState}><div class="tree-row">${toggle}<button class="tree-label" type="button" data-library-folder="${escapeHtml(node.path)}"><span aria-hidden="true">▰</span><span>${escapeHtml(node.name)}</span></button></div>${children}</li>`;
  };
  return `<ul class="tree-root" role="tree">${buildFolderTree(tree).map((node) => nodeMarkup(node, 1)).join("")}</ul>`;
}

function rowsMarkup(library) {
  const allEntries = sortLibraryEntries(library.results ?? library.folder.entries ?? [], library.sort);
  const page = paginateLibraryEntries(allEntries, library.page_size, library.page);
  const rows = page.entries.length === 0
    ? '<tr><td colspan="5" class="empty-table">This folder has no visible entries.</td></tr>'
    : page.entries.map((entry) => {
        const path = normalizeLibraryPath(entry.relative_path);
        const selected = library.selection.includes(path) ? " selected" : "";
        const icon = entry.kind === "folder" ? "▸" : "□";
        return `<tr class="library-row${selected}" tabindex="0" data-library-entry="${escapeHtml(path)}" data-library-kind="${escapeHtml(entry.kind)}"><td><span class="entry-name"><span aria-hidden="true">${icon}</span>${escapeHtml(entry.name)}</span></td><td>${escapeHtml(entry.document?.control?.title ?? "—")}</td><td>${escapeHtml(membershipLabel(entry))}</td><td>${escapeHtml(entry.document?.lifecycle ?? "—")}</td><td>${escapeHtml(path)}</td></tr>`;
      }).join("");
  const paging = page.total > library.page_size
    ? `<button class="text-button" type="button" data-library-page="previous" ${page.page === 0 ? "disabled" : ""}>Previous</button><span>Page ${page.page + 1} of ${page.page_count}</span><button class="text-button" type="button" data-library-page="next" ${page.page + 1 === page.page_count ? "disabled" : ""}>Next</button>`
    : `<span>${page.total} entries</span>`;
  return `${rows}<tr class="pagination-row"><td colspan="5"><div><label>Rows per page <select data-library-page-size><option value="10" ${library.page_size === 10 ? "selected" : ""}>10</option><option value="25" ${library.page_size === 25 ? "selected" : ""}>25</option><option value="50" ${library.page_size === 50 ? "selected" : ""}>50</option><option value="100" ${library.page_size === 100 ? "selected" : ""}>100</option></select></label><span>${paging}</span></div></td></tr>`;
}

function workflowEventMarkup(event) {
  const body = event.body ?? {};
  const label = String(body.event_type ?? "workflow_event").replaceAll("_", " ");
  const comment = body.operator_comment ?? body.decision_comment ?? body.changelog;
  return `<article class="workflow-event"><header><strong>${escapeHtml(label)}</strong><time>${escapeHtml(new Date(body.timestamp).toLocaleString())}</time></header>${comment ? `<p>${escapeHtml(comment)}</p>` : ""}<dl><dt>Event ID</dt><dd>${escapeHtml(body.event_id)}</dd><dt>Hash</dt><dd><code>${escapeHtml(event.event_hash)}</code></dd><dt>Predecessor</dt><dd><code>${escapeHtml(body.predecessor_hash ?? "Chain start")}</code></dd></dl></article>`;
}

function lifecyclePanelMarkup(library, detail) {
  const actions = detail.lifecycle_actions ?? {};
  const availability = (name, fallback) => actions[name] ?? { available: false, reason: fallback };
  const begin = availability("begin_revision", "Lifecycle state is unavailable.");
  const cancel = availability("cancel_review", "Lifecycle state is unavailable.");
  const obsolete = availability("mark_obsolete", "Lifecycle state is unavailable.");
  const form = (action, title, available, reason) => {
    const draft = library.lifecycle_drafts?.[action] ?? {};
    const disabled = available ? "" : "disabled";
    return `<form class="lifecycle-action" data-library-lifecycle-form="${action}"><strong>${title}</strong>${reason ? `<small>${escapeHtml(reason)}</small>` : ""}<label>Reason<textarea name="reason" required ${disabled}>${escapeHtml(draft.reason ?? "")}</textarea></label><label class="confirmation"><input type="checkbox" name="confirmed" value="yes" ${draft.confirmed ? "checked" : ""} ${disabled}> I confirm this lifecycle change.</label><button class="button ${action === "mark_obsolete" ? "danger" : "secondary"}" type="submit" ${disabled}>${title}</button></form>`;
  };
  const events = (detail.workflow_events ?? []).map(workflowEventMarkup).join("")
    || '<p class="source-path">No canonical workflow evidence has been recorded.</p>';
  const verification = typeof detail.workflow_verification === "string"
    ? detail.workflow_verification.replaceAll("_", " ")
    : detail.workflow_verification?.tampered_at
      ? `tampered at ${detail.workflow_verification.tampered_at}`
      : "invalid";
  return `<section class="lifecycle-panel" aria-labelledby="lifecycle-actions-heading"><h4 id="lifecycle-actions-heading">Lifecycle actions</h4><div class="lifecycle-actions"><div class="lifecycle-action"><strong>Begin revision</strong>${begin.reason ? `<small>${escapeHtml(begin.reason)}</small>` : ""}<button class="button secondary" type="button" data-library-lifecycle-action="begin_revision" ${begin.available ? "" : "disabled"}>Begin revision</button></div>${form("cancel_review", "Cancel review", cancel.available, cancel.reason)}${form("mark_obsolete", "Mark obsolete", obsolete.available, obsolete.reason)}${externalLifecycleMarkup(library, detail)}</div><button class="button secondary" type="button" data-library-open-evidence>View workflow evidence</button><details class="workflow-evidence" ${library.evidence_open ? "open" : ""}><summary>Canonical workflow evidence · ${escapeHtml(verification)}</summary>${events}</details></section>`;
}

function externalLifecycleMarkup(library, detail) {
  const candidate = detail.active_candidate;
  const mailConfirmation = (label) => `<label class="confirmation"><input type="checkbox" name="mailtoConfirmed" value="yes"> I confirm the host mail message for ${escapeHtml(label)} was sent.</label>`;
  const requesterOptions = (detail.eligible_people ?? [])
    .map((person) => `<option value="${escapeHtml(person.object_id)}">${escapeHtml(person.display_name)} · ${escapeHtml(person.email)}</option>`)
    .join("");
  const failedSignIn = Boolean(library.approver_sign_in?.challenge && library.detail_error);
  const signIn = library.approver_sign_in?.challenge
    ? failedSignIn
      ? '<p class="source-path"><strong>Previous sign-in failed.</strong> Generate a new Microsoft Entra device code before continuing.</p><button class="button secondary" type="button" data-library-approver-sign-in>Sign in again</button>'
      : `<p class="source-path">Complete Microsoft sign-in with code ${escapeHtml(library.approver_sign_in.challenge.user_code)}.</p><button class="button secondary" type="button" data-open-external="${escapeHtml(library.approver_sign_in.challenge.verification_uri)}">Open sign-in page</button><button class="button secondary" type="button" data-library-approver-sign-in-complete="${escapeHtml(library.approver_sign_in.challenge.challenge_id)}">Complete approver sign-in</button>`
    : library.approver_sign_in?.actor
      ? `<p class="source-path">Approver sign-in ready for ${escapeHtml(library.approver_sign_in.actor.display_name)}.</p>`
      : '<button class="button secondary" type="button" data-library-approver-sign-in>Sign in as approver</button>';
  const submit = detail.lifecycle === "draft" && !candidate
    ? `<form class="lifecycle-action" data-library-lifecycle-form="submit_candidate"><strong>Submit release candidate</strong><label>Target version<select name="targetMode"><option value="next_major">Next major (approval required)</option><option value="next_minor">Next minor</option><option value="manual">Manual target</option></select></label><label>Manual major<input name="manualMajor" inputmode="numeric"></label><label>Manual minor<input name="manualMinor" inputmode="numeric"></label><label>Requesting editor<select name="requesterObjectId" required><option value="">Choose person</option>${requesterOptions}</select></label><label>Changelog<textarea name="changelog" required></textarea></label><label>Review content-check override reason (only when needed)<textarea name="reviewOverrideReason"></textarea></label><button class="button" type="submit">Submit candidate</button></form>`
    : "";
  const reviewRetry = candidate?.status === "review_delivery_failed"
    ? `<form class="lifecycle-action" data-library-lifecycle-form="retry_review_notification"><strong>Confirm review request delivery</strong><small>The host mail handler opened without advancing the review.</small>${mailConfirmation("the review request")}<button class="button" type="submit">Confirm review message sent</button></form>`
    : "";
  const decision = candidate?.status === "in_review"
    ? `<form class="lifecycle-action" data-library-lifecycle-form="decide_review"><strong>Record review decision</strong>${signIn}<label>Decision<select name="decision" required><option value="">Choose decision</option><option value="approved">Approve</option><option value="rejected">Reject</option><option value="changes_requested">Request changes</option></select></label><label>Comment<textarea name="comment"></textarea></label><button class="button" type="submit">Record decision</button></form>`
    : "";
  const release = candidate && ((candidate.approval_required && candidate.status === "approved") || (!candidate.approval_required && candidate.status === "draft"))
    ? `<form class="lifecycle-action" data-library-lifecycle-form="release_candidate"><strong>Export and release ${escapeHtml(`V${candidate.version.major}.${candidate.version.minor}`)}</strong><small>Uses installed Office for .docx or the native print WebView for Markdown.</small><label>Release content-check override reason (only when needed)<textarea name="releaseOverrideReason"></textarea></label><button class="button" type="submit">Export PDF and release</button></form>`
    : "";
  const decisionRetry = detail.retryable_decision_candidate
    ? `<form class="lifecycle-action" data-library-lifecycle-form="retry_decision_notification"><strong>Confirm decision notification delivery</strong>${mailConfirmation("the decision outcome")}<button class="button secondary" type="submit">Confirm decision message sent</button></form>`
    : "";
  const minorRetry = detail.retryable_minor_publication
    ? `<form class="lifecycle-action" data-library-lifecycle-form="retry_minor_publication_notification"><strong>Confirm minor-publication delivery</strong>${mailConfirmation("the minor publication")}<button class="button secondary" type="submit">Confirm publication message sent</button></form>`
    : "";
  return `${submit}${reviewRetry}${decision}${release}${decisionRetry}${minorRetry}`;
}

function selectionMarkup(library) {
  const selected = selectedEntries(library);
  if (selected.length === 0) {
    return '<div class="selection-empty"><h3>Selection</h3><p>Select a folder or file to see its identity and available actions.</p></div>';
  }
  if (selected.length > 1) {
    const allAddable = selected.every((entry) => membershipKind(entry) === "not_in_library");
    const allRegistered = selected.every((entry) => membershipKind(entry) === "in_library");
    const identities = selected.slice(0, 5).map((entry) => `<li>${escapeHtml(entry.name)}</li>`).join("");
    return `<div class="selection-header"><span class="badge">${selected.length} selected</span><button class="text-button" type="button" data-library-clear-selection>Clear</button></div><ul class="identity-list">${identities}</ul><div class="selection-actions">${allAddable ? `<button class="button" type="button" data-library-add>Add ${selected.length} documents to library</button>` : ""}${allRegistered ? `<button class="button danger" type="button" data-library-unregister>Unregister ${selected.length} documents</button>` : ""}${!allAddable && !allRegistered ? "<p>Mixed selections have no common action.</p>" : ""}</div>`;
  }
  const entry = selected[0];
  if (entry.kind === "folder") {
    return `<h3>${escapeHtml(entry.name)}</h3><p class="source-path">${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><button class="button" type="button" data-library-open-selected>Open folder</button>`;
  }
  const membership = membershipKind(entry);
  if (membership === "not_in_library") {
    return `<span class="badge">Not in library</span><h3>${escapeHtml(entry.name)}</h3><p class="source-path">${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><button class="button" type="button" data-library-add>Add to library</button>`;
  }
  if (membership === "unsupported") {
    return `<span class="badge muted">Unsupported draft</span><h3>${escapeHtml(entry.name)}</h3><p class="source-path">${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><p>This file remains visible but cannot be registered.</p>`;
  }
  const detail = library.detail;
  if (!detail || detail.document_id !== entryDocumentId(entry)) {
    return `<h3>${escapeHtml(entry.document?.control?.title ?? entry.name)}</h3><p class="source-path">${escapeHtml(entry.name)}<br>${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><p>Loading document control data…</p>`;
  }
  const confidentiality = detail.effective_confidentiality;
  const roles = detail.effective_workflow_roles;
  const role = (value) => value?.display_name ?? (value?.object_id ? `Unresolved · ${value.object_id}` : "Not configured");
  const sourceAvailable = detail.source_exists && detail.source_state === "registered";
  const release = detail.current_release;
  const currentReleaseIdentity = release
    ? `<div class="source-identity"><strong>Current released PDF · V${escapeHtml(release.version)}</strong><span>${escapeHtml(release.relative_pdf_path)}</span><small>${release.pdf_exists ? "Available" : "Missing PDF"}</small></div>`
    : '<div class="source-identity"><strong>Current released PDF</strong><span>No active release</span></div>';
  const documentTypeOptions = (detail.document_types ?? [])
    .filter((type) => type.enabled || type.id === detail.control.document_type)
    .map((type) => `<option value="${escapeHtml(type.id)}" ${type.id === detail.control.document_type ? "selected" : ""}>${escapeHtml(type.label)}</option>`)
    .join("");
  const confidentialityOptions = (detail.confidentiality_types ?? [])
    .filter((type) => type.enabled || type.id === detail.confidentiality_override)
    .map((type) => `<option value="${escapeHtml(type.id)}" ${type.id === detail.confidentiality_override ? "selected" : ""}>${escapeHtml(type.label)}</option>`)
    .join("");
  const editor = `<section class="document-control-editor" aria-labelledby="document-control-editor-heading"><h4 id="document-control-editor-heading">Edit document control data</h4><p class="source-path">Applies to ${escapeHtml(detail.source_name)} · ${escapeHtml(detail.relative_path)}</p>${library.detail_error ? `<p class="library-detail-error" role="alert">${escapeHtml(library.detail_error)}</p>` : ""}<form id="library-document-control-form"><div class="document-control-fields"><label>Title<input name="title" required value="${escapeHtml(detail.control.title)}"></label><label>Document number<input name="documentNumber" value="${escapeHtml(detail.control.document_number ?? "")}"></label><label>Document type<select name="documentType"><option value="">Not set</option>${documentTypeOptions}</select></label><label>Owner<input name="owner" value="${escapeHtml(detail.control.owner ?? "")}"></label><label>Effective date<input name="effectiveDate" type="date" value="${escapeHtml(detail.control.effective_date ?? "")}"></label></div><button class="button" type="submit">Save document control</button></form><form id="library-confidentiality-form" class="confidentiality-editor"><label>Confidentiality override<select name="confidentialityTypeId"><option value="">Use inherited folder policy</option>${confidentialityOptions}</select></label><button class="button secondary" type="submit">Apply confidentiality</button></form></section>`;
  const currentRelease = `${currentReleaseIdentity}${editor}${lifecyclePanelMarkup(library, detail)}`;
  return `<div class="selection-header"><span class="badge">In library</span><button class="text-button" type="button" data-library-clear-selection>Clear</button></div><h3>${escapeHtml(detail.control.title)}</h3>${detail.control.document_number ? `<p class="document-number">${escapeHtml(detail.control.document_number)}</p>` : ""}<div class="source-identity"><strong>Source file</strong><span>${escapeHtml(detail.source_name)}</span><small>${escapeHtml(detail.relative_path)}</small></div>${currentRelease}<details open><summary>Document control data</summary><dl class="selection-details"><dt>Lifecycle</dt><dd>${escapeHtml(detail.lifecycle)}</dd><dt>Document type</dt><dd>${escapeHtml(detail.control.document_type ?? "Not set")}</dd><dt>Owner</dt><dd>${escapeHtml(detail.control.owner ?? "Not set")}</dd><dt>Confidentiality</dt><dd>${escapeHtml(confidentiality?.label ?? "Not configured")}${confidentiality ? ` · ${escapeHtml(confidentiality.document_override ? "override" : `from ${confidentiality.source_folder}`)}` : ""}</dd><dt>Editor</dt><dd>${escapeHtml(role(roles?.editor))}</dd><dt>Approver</dt><dd>${escapeHtml(role(roles?.approver))}</dd></dl></details><details open><summary>Actions</summary><div class="selection-actions"><button class="button" type="button" data-library-open-source ${sourceAvailable ? "" : "disabled"}>Open source draft</button><button class="button" type="button" data-library-open-release ${release?.pdf_exists ? "" : "disabled"}>Open current released PDF</button><button class="button" type="button" data-library-open-notes>Open notes</button><button class="button secondary" type="button" data-library-open-assistance>Evaluate changes with Claude</button><button class="button secondary" type="button" data-library-copy-permalink>Copy permalink</button><button class="button danger" type="button" data-library-unregister>Unregister</button></div><form id="library-reassociate-form" class="reassociate-form"><label>Reassociate source<input name="path" required value="${escapeHtml(detail.relative_path)}" aria-label="New edit-root-relative source path"></label><button class="button secondary" type="submit">Reassociate</button></form></details><details><summary>Revision cycle</summary><p>No revision cycle is open.</p></details><details><summary>Releases</summary><p>Release evidence remains available from the Releases destination.</p></details>`;
}

export function libraryMarkup(workspace, activity, library, error = "") {
  const folder = normalizeLibraryPath(library.folder?.relative_path ?? activity?.route_state?.folder);
  const breadcrumbs = breadcrumbSegments(folder, workspace.edit_root.split(/[\\/]/).filter(Boolean).at(-1) ?? "Library")
    .map((segment) => `<button type="button" data-library-folder="${escapeHtml(segment.path)}">${escapeHtml(segment.label)}</button>`)
    .join('<span aria-hidden="true">›</span>');
  const searchScope = library.entire_library ? "Entire library" : "Current folder";
  return `<section class="library-workspace"><div class="library-toolbar"><button class="icon-button" type="button" data-library-history="back" ${library.back.length ? "" : "disabled"} aria-label="Back" title="Back">←</button><button class="icon-button" type="button" data-library-history="forward" ${library.forward.length ? "" : "disabled"} aria-label="Forward" title="Forward">→</button><button class="icon-button" type="button" data-library-up ${folder === "." ? "disabled" : ""} aria-label="Up" title="Up">↑</button><button class="icon-button" type="button" data-library-refresh aria-label="Refresh" title="Refresh">↻</button><nav class="breadcrumbs" aria-label="Current folder">${breadcrumbs}</nav><form id="library-search-form" class="library-search"><input name="query" value="${escapeHtml(library.query)}" aria-label="Search library" placeholder="Search files, paths, titles, numbers"><label><input type="checkbox" name="entireLibrary" ${library.entire_library ? "checked" : ""}> Entire library</label><button class="button secondary" type="submit">Search</button>${library.results ? '<button class="text-button" type="button" data-library-clear-search>Clear</button>' : ""}</form><label class="sort-control">Sort <select data-library-sort><option value="name" ${library.sort === "name" ? "selected" : ""}>Name</option><option value="title" ${library.sort === "title" ? "selected" : ""}>Title</option><option value="number" ${library.sort === "number" ? "selected" : ""}>Document number</option><option value="lifecycle" ${library.sort === "lifecycle" ? "selected" : ""}>Lifecycle</option></select></label></div>${error ? `<p class="library-error" role="alert">${escapeHtml(error)}</p>` : ""}<div class="library-grid"><aside class="folder-tree" aria-label="Library folders"><h2>Folders</h2>${treeMarkup(library.tree, folder, library.expanded_folders)}</aside><section class="folder-contents"><header><div><span class="eyebrow">${library.results ? `Search · ${escapeHtml(searchScope)}` : "Current folder"}</span><h2>${escapeHtml(folder === "." ? "Library" : folder.split("/").at(-1))}</h2></div><span>${(library.results ?? library.folder.entries ?? []).length} entries</span></header><div class="table-scroll"><table><thead><tr><th>Name</th><th>Title</th><th>Membership</th><th>Lifecycle</th><th>Relative path</th></tr></thead><tbody>${rowsMarkup(library)}</tbody></table></div></section><aside class="selection-pane" aria-label="Selection details">${selectionMarkup(library)}</aside></div></section>`;
}
