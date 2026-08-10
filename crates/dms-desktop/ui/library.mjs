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
  return { ...library, selection, detail: null };
}

export function selectedEntries(library) {
  const entries = library.results ?? library.folder?.entries ?? [];
  return library.selection
    .map((path) => entries.find((entry) => normalizeLibraryPath(entry.relative_path) === path))
    .filter(Boolean);
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
  return `<div class="selection-header"><span class="badge">In library</span><button class="text-button" type="button" data-library-clear-selection>Clear</button></div><h3>${escapeHtml(detail.control.title)}</h3>${detail.control.document_number ? `<p class="document-number">${escapeHtml(detail.control.document_number)}</p>` : ""}<div class="source-identity"><strong>Source file</strong><span>${escapeHtml(detail.source_name)}</span><small>${escapeHtml(detail.relative_path)}</small></div><details open><summary>Document control data</summary><dl class="selection-details"><dt>Lifecycle</dt><dd>${escapeHtml(detail.lifecycle)}</dd><dt>Document type</dt><dd>${escapeHtml(detail.control.document_type ?? "Not set")}</dd><dt>Owner</dt><dd>${escapeHtml(detail.control.owner ?? "Not set")}</dd><dt>Confidentiality</dt><dd>${escapeHtml(confidentiality?.label ?? "Not configured")}${confidentiality ? ` · ${escapeHtml(confidentiality.document_override ? "override" : `from ${confidentiality.source_folder}`)}` : ""}</dd><dt>Editor</dt><dd>${escapeHtml(role(roles?.editor))}</dd><dt>Approver</dt><dd>${escapeHtml(role(roles?.approver))}</dd></dl></details><details open><summary>Actions</summary><div class="selection-actions"><button class="button" type="button" data-library-open-notes>Open notes</button><button class="button secondary" type="button" data-library-open-assistance>Evaluate changes with Claude</button><button class="button secondary" type="button" data-library-copy-permalink>Copy permalink</button><button class="button danger" type="button" data-library-unregister>Unregister</button></div><form id="library-reassociate-form" class="reassociate-form"><label>Reassociate source<input name="path" required value="${escapeHtml(detail.relative_path)}" aria-label="New edit-root-relative source path"></label><button class="button secondary" type="submit">Reassociate</button></form></details><details><summary>Revision cycle</summary><p>No revision cycle is open.</p></details><details><summary>Releases</summary><p>No released version exists.</p></details>`;
}

export function libraryMarkup(workspace, activity, library, error = "") {
  const folder = normalizeLibraryPath(library.folder?.relative_path ?? activity?.route_state?.folder);
  const breadcrumbs = breadcrumbSegments(folder, workspace.edit_root.split(/[\\/]/).filter(Boolean).at(-1) ?? "Library")
    .map((segment) => `<button type="button" data-library-folder="${escapeHtml(segment.path)}">${escapeHtml(segment.label)}</button>`)
    .join('<span aria-hidden="true">›</span>');
  const searchScope = library.entire_library ? "Entire library" : "Current folder";
  return `<section class="library-workspace"><div class="library-toolbar"><button class="icon-button" type="button" data-library-history="back" ${library.back.length ? "" : "disabled"} aria-label="Back" title="Back">←</button><button class="icon-button" type="button" data-library-history="forward" ${library.forward.length ? "" : "disabled"} aria-label="Forward" title="Forward">→</button><button class="icon-button" type="button" data-library-up ${folder === "." ? "disabled" : ""} aria-label="Up" title="Up">↑</button><button class="icon-button" type="button" data-library-refresh aria-label="Refresh" title="Refresh">↻</button><nav class="breadcrumbs" aria-label="Current folder">${breadcrumbs}</nav><form id="library-search-form" class="library-search"><input name="query" value="${escapeHtml(library.query)}" aria-label="Search library" placeholder="Search files, paths, titles, numbers"><label><input type="checkbox" name="entireLibrary" ${library.entire_library ? "checked" : ""}> Entire library</label><button class="button secondary" type="submit">Search</button>${library.results ? '<button class="text-button" type="button" data-library-clear-search>Clear</button>' : ""}</form><label class="sort-control">Sort <select data-library-sort><option value="name" ${library.sort === "name" ? "selected" : ""}>Name</option><option value="title" ${library.sort === "title" ? "selected" : ""}>Title</option><option value="number" ${library.sort === "number" ? "selected" : ""}>Document number</option><option value="lifecycle" ${library.sort === "lifecycle" ? "selected" : ""}>Lifecycle</option></select></label></div>${error ? `<p class="library-error" role="alert">${escapeHtml(error)}</p>` : ""}<div class="library-grid"><aside class="folder-tree" aria-label="Library folders"><h2>Folders</h2>${treeMarkup(library.tree, folder, library.expanded_folders)}</aside><section class="folder-contents"><header><div><span class="eyebrow">${library.results ? `Search · ${escapeHtml(searchScope)}` : "Current folder"}</span><h2>${escapeHtml(folder === "." ? "Library" : folder.split("/").at(-1))}</h2></div><span>${(library.results ?? library.folder.entries ?? []).length} entries</span></header><div class="table-scroll"><table><thead><tr><th>Name</th><th>Title</th><th>Membership</th><th>Lifecycle</th><th>Relative path</th></tr></thead><tbody>${rowsMarkup(library)}</tbody></table></div></section><aside class="selection-pane" aria-label="Selection details">${selectionMarkup(library)}</aside></div></section>`;
}
