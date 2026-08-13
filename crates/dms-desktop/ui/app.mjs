import {
  applyDocumentSelection,
  applyLibrarySnapshot,
  confidentialityUpdateRequest,
  createLibraryState,
  documentControlUpdateRequest,
  entryDocumentId,
  historyTarget,
  lifecycleActionRequest,
  libraryMarkup,
  libraryOpenRequest,
  membershipKind,
  normalizeLibraryPath,
  selectedEntries,
  toggleLibrarySelection,
  toggleTreeFolder,
} from "./library.mjs";
import {
  applyDocumentNotes,
  createNoteDocumentState,
  documentNotesMarkup,
  noteDocumentState,
  updateNoteDocumentState,
} from "./notes.mjs";
import {
  applyReleaseSnapshot,
  createReleaseState,
  periodicReviewMarkup,
  periodicReviewRequest,
  releaseMaintenanceMarkup,
  releaseWithdrawalRequest,
  workspaceMaintenanceMarkup,
  workspaceRestoreRequest,
} from "./maintenance.mjs";
import {
  assistanceDocumentState,
  assistanceMarkup,
  createAssistanceState,
  updateAssistanceState,
} from "./assistance.mjs";
import {
  applyConfigurationSnapshot,
  closeConfigurationSecondary,
  configurationMarkup,
  configurationMutationRequest,
  createConfigurationState,
  openConfigurationSecondary,
  selectConfigurationFolder,
  setConfigurationRoute,
} from "./configuration.mjs";
import {
  applyAuditReportSnapshot,
  auditReportRequest,
  auditReportsMarkup,
  createAuditReportState,
} from "./reports.mjs";

const DESTINATIONS = [
  ["Library", "▦"],
  ["Releases", "□"],
  ["Audit & Reports", "≣"],
  ["Maintenance", "◇"],
  ["Configuration", "⚙"],
];
const RECENT_LIBRARIES_LIMIT = 10;

export function defaultPreferences() {
  return { sidebar_expanded: true, saved_views: [], recent_libraries: [] };
}

function normalizedRecentLibraries(paths) {
  return [...new Set((Array.isArray(paths) ? paths : [])
    .map((path) => String(path).trim())
    .filter(Boolean))].slice(0, RECENT_LIBRARIES_LIMIT);
}

export function rememberRecentLibrary(preferences, editRoot) {
  const root = String(editRoot ?? "").trim();
  const existing = normalizedRecentLibraries(preferences.recent_libraries);
  return {
    ...preferences,
    recent_libraries: root
      ? [root, ...existing.filter((candidate) => candidate !== root)].slice(0, RECENT_LIBRARIES_LIMIT)
      : existing,
  };
}

export function removeRecentLibrary(preferences, editRoot) {
  const root = String(editRoot ?? "").trim();
  return {
    ...preferences,
    recent_libraries: normalizedRecentLibraries(preferences.recent_libraries)
      .filter((candidate) => candidate !== root),
  };
}

export function createInitialState(preferences = defaultPreferences()) {
  return {
    preferences: {
      sidebar_expanded: preferences.sidebar_expanded !== false,
      saved_views: Array.isArray(preferences.saved_views) ? preferences.saved_views : [],
      recent_libraries: normalizedRecentLibraries(preferences.recent_libraries),
    },
    activities: [],
    current_key: null,
    workspace: null,
    library: createLibraryState(),
    note_documents: {},
    assistance_documents: {},
    assistance_policy: { value: null, error: "" },
    configuration: createConfigurationState(),
    releases: createReleaseState(),
    periodic_reviews: { markers: [], loading: false, error: "", notice: "" },
    audit_reports: createAuditReportState(),
    maintenance: {
      backup_outcome: null,
      restore_outcome: null,
      lock_status: null,
      notice: "",
      error: "",
    },
    sidebar_overlay: false,
    flyout: null,
    setup_edit_root: "",
    error: "",
  };
}

function normalizedFolder(folder) {
  const normalized = String(folder ?? ".").replaceAll("\\", "/").replace(/^\/+|\/+$/g, "");
  return normalized || ".";
}

export function activityKey(activity) {
  if (activity.task === "Library") {
    return `${activity.workspace_id}:Library`;
  }
  if (activity.document_id) {
    return `${activity.workspace_id}:${activity.task}:document:${activity.document_id}`;
  }
  if (activity.route_state?.folder) {
    return `${activity.workspace_id}:${activity.task}:folder:${normalizedFolder(activity.route_state.folder)}`;
  }
  return `${activity.workspace_id}:${activity.task}`;
}

export function savedViewId(activity) {
  if (activity.task !== "Library") return activityKey(activity);
  const folder = normalizedFolder(activity.route_state?.folder);
  const sort = activity.route_state?.sort ?? "name";
  const document = activity.document_id ?? "none";
  return `${activity.workspace_id}:saved:Library:${folder}:${sort}:${document}`;
}

export function openActivity(state, activity) {
  const key = activityKey(activity);
  const next = { ...activity, key };
  const existing = state.activities.findIndex((candidate) => candidate.key === key);
  const activities = [...state.activities];
  if (existing === -1) {
    activities.push(next);
  } else {
    activities[existing] = next;
  }
  return { ...state, activities, current_key: key, flyout: null };
}

export function permalinkActivity(resolution) {
  const suffix = resolution.document_number ? ` · ${resolution.document_number}` : "";
  const shared = {
    workspace_id: resolution.workspace.workspace_id,
    destination: "Library",
    document_id: resolution.document_id,
  };
  if (resolution.target === "notes") {
    return {
      ...shared,
      task: "Notes",
      label: `Notes · ${resolution.title}${suffix}`,
      route_state: {},
    };
  }
  if (resolution.target === "review") {
    return {
      ...shared,
      task: "Review",
      label: `Review · ${resolution.title}${suffix}`,
      route_state: { review: resolution.review_id },
    };
  }
  return {
    ...shared,
    task: "Library",
    label: resolution.folder === "." ? "Library · /" : `Library · ${resolution.folder}`,
    route_state: { folder: resolution.folder },
  };
}

export function applyPermalinkDocumentSelection(library, detail) {
  const entry = library.folder.entries.find(
    (candidate) => entryDocumentId(candidate) === detail.document_id,
  );
  return {
    ...library,
    selection: entry ? [normalizeLibraryPath(entry.relative_path)] : [],
    detail,
    detail_error: "",
  };
}

export function closeActivity(state, key) {
  const activities = state.activities.filter((activity) => activity.key !== key);
  const current_key = state.current_key === key
    ? activities.at(-1)?.key ?? null
    : state.current_key;
  return { ...state, activities, current_key };
}

export function toggleSavedView(preferences, activity) {
  const id = savedViewId(activity);
  const savedViews = preferences.saved_views ?? [];
  const existing = savedViews.some((view) => view.id === id);
  return {
    ...preferences,
    saved_views: existing
      ? savedViews.filter((view) => view.id !== id)
      : [...savedViews, {
          id,
          workspace_id: activity.workspace_id,
          destination: activity.destination,
          task: activity.task,
          label: activity.label,
          document_id: activity.document_id ?? null,
          route_state: { ...(activity.route_state ?? {}) },
        }],
  };
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function currentActivity(state) {
  return state.activities.find((activity) => activity.key === state.current_key) ?? null;
}

function bookmarkActivity(state) {
  const activity = currentActivity(state);
  if (!activity || activity.task !== "Library") return activity;
  const selected = selectedEntries(state.library);
  const documentId = selected.length === 1 && membershipKind(selected[0]) === "in_library"
    ? entryDocumentId(selected[0])
    : null;
  return {
    ...activity,
    document_id: documentId,
    route_state: {
      ...activity.route_state,
      folder: normalizeLibraryPath(state.library.folder.relative_path),
      sort: state.library.sort,
    },
  };
}

function activityFromSavedView(view) {
  return {
    workspace_id: view.workspace_id,
    destination: view.destination,
    task: view.task,
    label: view.label,
    document_id: view.document_id ?? null,
    route_state: { ...(view.route_state ?? {}) },
  };
}

function invokeCommand(command, arguments_) {
  const invoke = globalThis.__TAURI__?.core?.invoke;
  if (!invoke) {
    return Promise.reject(new Error("This action is available only in the desktop app."));
  }
  return invoke(command, arguments_);
}

function persistPreferences(state) {
  return invokeCommand("save_preferences", { preferences: state.preferences }).catch((error) => {
    console.error("Could not save preferences", error);
  });
}

function groupMarkup(title, items, kind) {
  const rows = items.length === 0
    ? '<p class="empty-group">None</p>'
    : items.map((item) => {
        const current = kind === "activity" && item.key === appState.current_key ? " current" : "";
        const action = kind === "saved" ? "−" : "×";
        const actionLabel = kind === "saved" ? "Remove saved view" : "Close pane";
        return `<button class="list-button${current}" type="button" data-${kind}-key="${escapeHtml(kind === "saved" ? item.id : item.key)}" title="${escapeHtml(item.label)}" aria-label="${escapeHtml(item.label)}"><span aria-hidden="true">${kind === "saved" ? "★" : "▹"}</span><span class="item-label">${escapeHtml(item.label)}</span><span class="item-action" data-${kind}-remove="${escapeHtml(kind === "saved" ? item.id : item.key)}" aria-label="${actionLabel}" title="${actionLabel}">${action}</span></button>`;
      }).join("");
  return `<h2 class="group-heading">${title}</h2><div class="group-list">${rows}</div>`;
}

function renderNavigation(state) {
  const navigation = document.querySelector("#primary-navigation");
  const destinations = state.workspace ? DESTINATIONS : [["Set up workspace", "+"]];
  const selected = currentActivity(state)?.destination ?? (state.workspace ? "Library" : "Set up workspace");
  navigation.innerHTML = destinations.map(([label, icon]) => {
    const current = selected === label ? " current" : "";
    return `<button class="nav-button${current}" type="button" data-destination="${escapeHtml(label)}" aria-label="${escapeHtml(label)}" title="${escapeHtml(label)}"><span class="nav-icon" aria-hidden="true">${icon}</span><span class="item-label">${escapeHtml(label)}</span></button>`;
  }).join("");
}

function renderGroups(state) {
  const saved = groupMarkup("Saved views", state.preferences.saved_views, "saved");
  const activities = groupMarkup("Open panes", state.activities, "activity");
  document.querySelector("#expanded-groups").innerHTML = saved + activities;
  document.querySelector("#collapsed-groups").innerHTML = '<button class="rail-button" type="button" data-flyout="saved" aria-label="Saved views" title="Saved views">★</button><button class="rail-button" type="button" data-flyout="activity" aria-label="Open panes" title="Open panes">▹</button>';

  const flyout = document.querySelector("#sidebar-flyout");
  flyout.hidden = !state.flyout;
  flyout.innerHTML = state.flyout === "saved" ? saved : activities;
  document.querySelectorAll(`[data-flyout="${state.flyout ?? ""}"]`).forEach((button) => button.classList.add("active"));
}

function setupValue(values, name) {
  return typeof values.get === "function" ? values.get(name) : values[name];
}

export function workspaceSetupRequest(formId, values) {
  const editRoot = String(setupValue(values, "editRoot") ?? "").trim();
  if (formId === "open-workspace-form") {
    return {
      command: "open_workspace",
      arguments: { editRoot },
      lockOptions: {
        takeOverStale: setupValue(values, "takeOverStale") === "on"
          || setupValue(values, "takeOverStale") === true,
        overrideExisting: setupValue(values, "overrideExisting") === "on"
          || setupValue(values, "overrideExisting") === true,
      },
    };
  }
  if (formId === "initialize-workspace-form") {
    return {
      command: "initialize_workspace",
      arguments: {
        editRoot,
        publishRoot: String(setupValue(values, "publishRoot") ?? "").trim(),
        confirmed: setupValue(values, "confirmed") === "on" || setupValue(values, "confirmed") === true,
      },
    };
  }
  throw new Error(`Unknown workspace setup form: ${formId}`);
}

function directoryFieldMarkup(id, name, label, placeholder, value = "") {
  const initialValue = value ? ` value="${escapeHtml(value)}"` : "";
  return `<div class="field"><label for="${id}">${label}</label><div class="directory-field"><input id="${id}" name="${name}" required autocomplete="off" placeholder="${placeholder}"${initialValue}><button class="button secondary" type="button" data-directory-target="${id}">Browse…</button></div></div>`;
}

function recentLibraryLabel(editRoot) {
  const path = String(editRoot).replace(/[\\/]+$/, "");
  return path.split(/[\\/]/).at(-1) || editRoot;
}

function recentLibrariesMarkup(recentLibraries) {
  if (recentLibraries.length === 0) {
    return '<p class="empty-recent-libraries">No recently opened libraries.</p>';
  }
  return recentLibraries.map((editRoot) => {
    const path = escapeHtml(editRoot);
    const label = escapeHtml(recentLibraryLabel(editRoot));
    return `<div class="recent-library-row"><button class="recent-library-open" type="button" data-recent-library-open="${path}" aria-label="Open recent library ${path}" title="${path}"><span aria-hidden="true">▦</span><span><strong>${label}</strong><small>${path}</small></span></button><button class="icon-button" type="button" data-recent-library-remove="${path}" aria-label="Remove ${path} from recent libraries" title="Remove from recent libraries">×</button></div>`;
  }).join("");
}

export function setupMarkup(error, recentLibraries = [], openEditRoot = "") {
  const recent = normalizedRecentLibraries(recentLibraries);
  const status = error ? `<p class="status" role="alert">${escapeHtml(error)}</p>` : "";
  return `<section class="setup-workspace"><header><span class="badge">Local workspace</span><h2>Set up DMS Desktop</h2><p>Open existing metadata or initialize explicit edit and publish roots. No documents are moved or copied during setup.</p></header><section class="recent-libraries card" aria-labelledby="recent-libraries-heading"><h3 id="recent-libraries-heading">Recent libraries</h3><div class="recent-libraries-list">${recentLibrariesMarkup(recent)}</div>${status}</section><div class="setup-grid"><section class="card"><h3>Open an existing workspace</h3><p>Choose an edit root that already contains <code>.dms/workspace.json</code>. A current advisory lock blocks opening.</p><form id="open-workspace-form" class="setup-form">${directoryFieldMarkup("open-edit-root", "editRoot", "Edit root", "C:\\DMS\\Edit or /Users/name/DMS/Edit", openEditRoot)}<label class="confirm-field"><input type="checkbox" name="takeOverStale"> Take over the lock only if it is stale.</label><label class="confirm-field lock-override"><input type="checkbox" name="overrideExisting"><span><strong>Override any existing lock.</strong> Another DMS instance may still be writing workspace metadata.</span></label><button class="button" type="submit">Open workspace</button></form></section><section class="card"><h3>Initialize a workspace</h3><p>The desktop creates <code>.dms</code> under the edit root and creates the publish root if it does not exist.</p><form id="initialize-workspace-form" class="setup-form">${directoryFieldMarkup("initialize-edit-root", "editRoot", "Edit root", "C:\\DMS\\Edit or /Users/name/DMS/Edit")}${directoryFieldMarkup("publish-root", "publishRoot", "Publish root", "C:\\DMS\\Publish or /Users/name/DMS/Publish")}<label class="confirm-field"><input type="checkbox" name="confirmed" required> Initialize these roots and create workspace metadata.</label><button class="button" type="submit">Initialize workspace</button></form></section></div></section>`;
}

function activityMarkup(state, activity) {
  if (!activity) {
    return setupMarkup(state.error, state.preferences.recent_libraries, state.setup_edit_root);
  }
  const workspace = state.workspace;
  if (activity.task === "Notes") {
    return documentNotesMarkup(activity, noteDocumentState(state.note_documents, activity.document_id));
  }
  if (activity.task === "Review") {
    return `<section class="card"><span class="badge">Review target</span><h2>${escapeHtml(activity.label)}</h2><p>This permalink selected the stable review request for the document.</p><dl class="details-grid"><dt>Document ID</dt><dd>${escapeHtml(activity.document_id)}</dd><dt>Review ID</dt><dd>${escapeHtml(activity.route_state.review)}</dd></dl></section>`;
  }
  if (activity.task === "Claude assistance") {
    return assistanceMarkup(
      activity,
      assistanceDocumentState(state.assistance_documents, activity.document_id),
    );
  }
  if (activity.destination === "Library") {
    return libraryMarkup(workspace, activity, state.library, state.error);
  }
  if (activity.destination === "Releases") {
    return releaseMaintenanceMarkup(state.releases);
  }
  if (activity.destination === "Audit & Reports") {
    return `${auditReportsMarkup(state.audit_reports)}${periodicReviewMarkup(state.periodic_reviews)}`;
  }
  if (activity.destination === "Maintenance") {
    return workspaceMaintenanceMarkup(state.maintenance);
  }
  if (activity.destination === "Configuration") {
    const route = activity.route_state?.route ?? "workspace";
    return configurationMarkup(
      route === state.configuration.route
        ? state.configuration
        : setConfigurationRoute(state.configuration, route),
      state.assistance_policy,
    );
  }
  return `<section class="card"><span class="badge">${escapeHtml(activity.destination)}</span><h2>${escapeHtml(activity.label)}</h2><p>The desktop shell is connected to the shared Rust core. Domain workflows beyond the phase-1 shell remain unavailable until their CHG phases are implemented.</p><dl class="details-grid"><dt>Workspace ID</dt><dd>${escapeHtml(workspace.workspace_id)}</dd><dt>Edit root</dt><dd>${escapeHtml(workspace.edit_root)}</dd><dt>Publish root</dt><dd>${escapeHtml(workspace.publish_root)}</dd><dt>Controlled documents</dt><dd>${escapeHtml(workspace.document_count)}</dd></dl></section>`;
}

function render(state) {
  const root = document.querySelector("#app");
  root.classList.toggle("sidebar-collapsed", !state.preferences.sidebar_expanded && !state.sidebar_overlay);
  renderNavigation(state);
  renderGroups(state);

  const activity = currentActivity(state);
  document.querySelector("#activity-heading").textContent = activity?.label ?? "Set up workspace";
  const mainContent = document.querySelector("#main-content");
  mainContent.classList.toggle("library-active", activity?.task === "Library");
  mainContent.innerHTML = state.workspace
    ? activityMarkup(state, activity)
    : setupMarkup(state.error, state.preferences.recent_libraries, state.setup_edit_root);

  const bookmark = document.querySelector("#bookmark-view");
  const bookmarkTarget = bookmarkActivity(state);
  const bookmarked = bookmarkTarget
    && state.preferences.saved_views.some((view) => view.id === savedViewId(bookmarkTarget));
  bookmark.hidden = !activity;
  bookmark.textContent = bookmarked ? "★ Bookmarked" : "☆ Bookmark this view";
  bookmark.classList.toggle("bookmarked", Boolean(bookmarked));
  bookmark.setAttribute("aria-pressed", String(Boolean(bookmarked)));

  const foot = document.querySelector("#workspace-foot");
  foot.innerHTML = state.workspace
    ? `<strong>${escapeHtml(state.workspace.workspace_id)}</strong><br>edit: ${escapeHtml(state.workspace.edit_root)}<br>publish: ${escapeHtml(state.workspace.publish_root)}`
    : "No workspace open";
}

function openDestination(destination) {
  if (destination === "Set up workspace") {
    appState = { ...appState, current_key: null, flyout: null };
    render(appState);
    return;
  }
  const folder = destination === "Library" ? "." : null;
  const configurationRoute = destination === "Configuration" ? "workspace" : null;
  const label = folder ? "Library · /" : destination;
  appState = openActivity(appState, {
    workspace_id: appState.workspace.workspace_id,
    destination,
    task: destination,
    label,
    document_id: null,
    route_state: folder ? { folder } : configurationRoute ? { route: configurationRoute } : {},
  });
  if (configurationRoute) {
    appState = {
      ...appState,
      configuration: setConfigurationRoute(appState.configuration, configurationRoute),
    };
  }
  render(appState);
  if (destination === "Library") {
    void loadLibraryFolder(folder, "replace");
  } else if (destination === "Releases") {
    void loadReleases();
  } else if (destination === "Audit & Reports") {
    void loadPeriodicReviews();
    void loadAuditReports();
  } else if (destination === "Maintenance") {
    void loadWorkspaceLockStatus();
  } else if (destination === "Configuration") {
    void loadWorkspaceConfiguration();
    void loadClaudeAssistancePolicy();
  }
}

export async function switchWorkspaceSession(
  currentWorkspace,
  currentLockStatus,
  workspace,
  lockOptions,
  invoke,
) {
  if (currentWorkspace?.edit_root === workspace.edit_root) return null;
  const lockStatus = await invoke("acquire_workspace_lock", {
    editRoot: workspace.edit_root,
    takeOverStale: lockOptions.takeOverStale ?? false,
    overrideExisting: lockOptions.overrideExisting ?? false,
  });
  if (currentWorkspace) {
    const currentOwner = currentLockStatus?.lock;
    if (!currentOwner) {
      await invoke("release_workspace_lock", {
        editRoot: workspace.edit_root,
        owner: lockStatus.lock,
        confirmed: true,
      });
      throw new Error("Active workspace lock owner is unavailable.");
    }
    try {
      await invoke("release_workspace_lock", {
        editRoot: currentWorkspace.edit_root,
        owner: currentOwner,
        confirmed: true,
      });
    } catch (error) {
      await invoke("release_workspace_lock", {
        editRoot: workspace.edit_root,
        owner: lockStatus.lock,
        confirmed: true,
      });
      throw error;
    }
  }
  return lockStatus;
}

async function activateWorkspace(workspace, lockOptions = {}, openLibrary = true) {
  const transitioned = await switchWorkspaceSession(
    appState.workspace,
    appState.maintenance.lock_status,
    workspace,
    lockOptions,
    invokeCommand,
  );
  const lockStatus = transitioned ?? appState.maintenance.lock_status;
  const preferences = rememberRecentLibrary(appState.preferences, workspace.edit_root);
  const sidebarOverlay = appState.sidebar_overlay;
  const initial = createInitialState(preferences);
  appState = {
    ...initial,
    workspace,
    maintenance: { ...initial.maintenance, lock_status: lockStatus },
    sidebar_overlay: sidebarOverlay,
  };
  await persistPreferences(appState);
  if (openLibrary) openDestination("Library");
}

async function openPermalink(uri) {
  const resolution = await invokeCommand("resolve_registered_permalink", { uri });
  if (appState.workspace?.workspace_id !== resolution.workspace.workspace_id) {
    await activateWorkspace(resolution.workspace, {}, false);
  }
  const activity = permalinkActivity(resolution);
  appState = openActivity(appState, activity);
  render(appState);
  if (resolution.target === "notes") {
    await loadDocumentNotes(resolution.document_id);
  } else if (resolution.target === "document") {
    await loadLibraryFolder(resolution.folder, "replace");
    try {
      const detail = await invokeCommand("load_document_selection", {
        editRoot: appState.workspace.edit_root,
        documentId: resolution.document_id,
      });
      appState = {
        ...appState,
        library: applyPermalinkDocumentSelection(appState.library, detail),
        error: "",
      };
    } catch (error) {
      appState = { ...appState, error: String(error) };
    }
    render(appState);
  }
}

async function loadWorkspaceLockStatus(notice = "") {
  try {
    const lockStatus = await invokeCommand("workspace_lock_status", {
      editRoot: appState.workspace.edit_root,
    });
    appState = {
      ...appState,
      maintenance: { ...appState.maintenance, lock_status: lockStatus, notice, error: "" },
    };
  } catch (error) {
    appState = {
      ...appState,
      maintenance: { ...appState.maintenance, notice: "", error: String(error) },
    };
  }
  render(appState);
}

async function applyWorkspaceLockMutation(command, arguments_, notice) {
  try {
    await invokeCommand(command, {
      editRoot: appState.workspace.edit_root,
      ...arguments_,
    });
    await loadWorkspaceLockStatus(notice);
  } catch (error) {
    appState = {
      ...appState,
      maintenance: { ...appState.maintenance, notice: "", error: String(error) },
    };
    render(appState);
  }
}

async function loadClaudeAssistancePolicy() {
  try {
    const value = await invokeCommand("load_claude_assistance_policy", {
      editRoot: appState.workspace.edit_root,
    });
    appState = { ...appState, assistance_policy: { value, error: "" } };
  } catch (error) {
    appState = {
      ...appState,
      assistance_policy: { ...appState.assistance_policy, error: String(error) },
    };
  }
  render(appState);
}

async function loadWorkspaceConfiguration(notice = "") {
  try {
    const snapshot = await invokeCommand("load_workspace_configuration", {
      editRoot: appState.workspace.edit_root,
    });
    appState = {
      ...appState,
      workspace: snapshot.workspace,
      configuration: applyConfigurationSnapshot(appState.configuration, snapshot, notice),
    };
  } catch (error) {
    appState = {
      ...appState,
      configuration: { ...appState.configuration, notice: "", error: String(error) },
    };
  }
  render(appState);
}

function updateLibraryActivity(folder) {
  const activity = currentActivity(appState);
  if (!activity || activity.destination !== "Library") return;
  const normalized = normalizeLibraryPath(folder);
  appState = openActivity(appState, {
    ...activity,
    label: normalized === "." ? "Library · /" : `Library · ${normalized}`,
    document_id: null,
    route_state: { ...activity.route_state, folder: normalized, sort: appState.library.sort },
  });
}

async function loadLibraryFolder(folder, historyMode = "push", documentId = null) {
  const target = normalizeLibraryPath(folder);
  appState = { ...appState, library: { ...appState.library, loading: true }, error: "" };
  render(appState);
  try {
    const snapshot = await invokeCommand("load_library", {
      editRoot: appState.workspace.edit_root,
      folder: target,
    });
    appState = {
      ...appState,
      library: applyLibrarySnapshot(appState.library, snapshot, target, historyMode),
      error: "",
    };
    if (documentId) {
      const entry = appState.library.folder.entries.find(
        (candidate) => entryDocumentId(candidate) === documentId,
      );
      if (entry) {
        appState = {
          ...appState,
          library: {
            ...appState.library,
            selection: [normalizeLibraryPath(entry.relative_path)],
          },
        };
      }
    }
    updateLibraryActivity(target);
  } catch (error) {
    appState = {
      ...appState,
      library: { ...appState.library, loading: false },
      error: String(error),
    };
  }
  render(appState);
  if (documentId) void loadSelectedDocument();
}

async function loadSelectedDocument() {
  const selected = selectedEntries(appState.library);
  if (selected.length !== 1 || membershipKind(selected[0]) !== "in_library") return;
  const documentId = entryDocumentId(selected[0]);
  try {
    const detail = await invokeCommand("load_document_selection", {
      editRoot: appState.workspace.edit_root,
      documentId,
    });
    if (selectedEntries(appState.library).some((entry) => entryDocumentId(entry) === documentId)) {
      appState = { ...appState, library: { ...appState.library, detail }, error: "" };
      render(appState);
    }
  } catch (error) {
    appState = { ...appState, error: String(error) };
    render(appState);
  }
}

async function loadReleases(command = "load_releases", arguments_ = {}) {
  appState = { ...appState, releases: { ...appState.releases, loading: true, error: "" } };
  render(appState);
  try {
    const snapshot = await invokeCommand(command, {
      editRoot: appState.workspace.edit_root,
      ...arguments_,
    });
    appState = { ...appState, releases: applyReleaseSnapshot(appState.releases, snapshot) };
  } catch (error) {
    appState = {
      ...appState,
      releases: { ...appState.releases, loading: false, error: String(error) },
    };
  }
  render(appState);
}

async function loadPeriodicReviews(notice = "") {
  appState = {
    ...appState,
    periodic_reviews: { ...appState.periodic_reviews, loading: true, error: "" },
  };
  render(appState);
  try {
    const markers = await invokeCommand("load_periodic_reviews", {
      editRoot: appState.workspace.edit_root,
    });
    appState = {
      ...appState,
      periodic_reviews: { markers, loading: false, error: "", notice },
    };
  } catch (error) {
    appState = {
      ...appState,
      periodic_reviews: { ...appState.periodic_reviews, loading: false, error: String(error) },
    };
  }
  render(appState);
}

async function loadAuditReports(notice = "") {
  appState = {
    ...appState,
    audit_reports: { ...appState.audit_reports, loading: true, error: "" },
  };
  render(appState);
  try {
    const snapshot = await invokeCommand("load_audit_reports", {
      editRoot: appState.workspace.edit_root,
    });
    appState = {
      ...appState,
      audit_reports: applyAuditReportSnapshot(appState.audit_reports, snapshot, notice),
    };
  } catch (error) {
    appState = {
      ...appState,
      audit_reports: { ...appState.audit_reports, loading: false, error: String(error) },
    };
  }
  render(appState);
}

async function loadDocumentNotes(documentId) {
  appState = {
    ...appState,
    note_documents: updateNoteDocumentState(appState.note_documents, documentId, {
      loading: true,
      error: "",
    }),
  };
  render(appState);
  try {
    const detail = await invokeCommand("load_document_notes", {
      editRoot: appState.workspace.edit_root,
      documentId,
    });
    appState = {
      ...appState,
      note_documents: applyDocumentNotes(appState.note_documents, detail),
    };
  } catch (error) {
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, documentId, {
        loading: false,
        error: String(error),
      }),
    };
  }
  render(appState);
}

function openDocumentNotes() {
  const detail = appState.library.detail;
  if (!detail) return;
  const suffix = detail.control.document_number ? ` · ${detail.control.document_number}` : "";
  appState = openActivity(appState, {
    workspace_id: appState.workspace.workspace_id,
    destination: "Library",
    task: "Notes",
    label: `Notes · ${detail.control.title}${suffix}`,
    document_id: detail.document_id,
    route_state: {},
  });
  if (!appState.note_documents[detail.document_id]) {
    appState = {
      ...appState,
      note_documents: {
        ...appState.note_documents,
        [detail.document_id]: createNoteDocumentState(),
      },
    };
  }
  render(appState);
  void loadDocumentNotes(detail.document_id);
}

async function loadClaudeAssistanceAvailability(documentId) {
  appState = {
    ...appState,
    assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
      loading: true,
      error: "",
    }),
  };
  render(appState);
  try {
    const availability = await invokeCommand("claude_assistance_availability", {
      editRoot: appState.workspace.edit_root,
      documentId,
    });
    appState = {
      ...appState,
      assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
        availability,
        loading: false,
      }),
    };
  } catch (error) {
    appState = {
      ...appState,
      assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
        loading: false,
        error: String(error),
      }),
    };
  }
  render(appState);
}

function openDocumentAssistance() {
  const detail = appState.library.detail;
  if (!detail) return;
  const suffix = detail.control.document_number ? ` · ${detail.control.document_number}` : "";
  appState = openActivity(appState, {
    workspace_id: appState.workspace.workspace_id,
    destination: "Library",
    task: "Claude assistance",
    label: `Evaluate changes · ${detail.control.title}${suffix}`,
    document_id: detail.document_id,
    route_state: {},
  });
  if (!appState.assistance_documents[detail.document_id]) {
    appState = {
      ...appState,
      assistance_documents: {
        ...appState.assistance_documents,
        [detail.document_id]: createAssistanceState(),
      },
    };
  }
  render(appState);
  void loadClaudeAssistanceAvailability(detail.document_id);
}

async function handleAssistanceClick(event) {
  const activity = currentActivity(appState);
  if (activity?.task !== "Claude assistance") return false;
  const documentId = activity.document_id;
  if (event.target.closest("[data-assistance-preview]")) {
    try {
      const state = assistanceDocumentState(appState.assistance_documents, documentId);
      const selectedExcerptLines = state.preview
        ? [...document.querySelectorAll("[data-assistance-excerpt]:checked")]
          .map((input) => Number(input.value))
          .filter(Number.isSafeInteger)
        : null;
      const preview = await invokeCommand("preview_claude_assistance", {
        editRoot: appState.workspace.edit_root,
        documentId,
        selectedExcerptLines,
      });
      appState = {
        ...appState,
        assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
          preview,
          error: "",
          launched: false,
        }),
      };
    } catch (error) {
      appState = {
        ...appState,
        assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
          error: String(error),
        }),
      };
    }
    render(appState);
    return true;
  }
  if (event.target.closest("[data-assistance-handoff]")) {
    const consent = document.querySelector("[data-assistance-consent]")?.checked;
    const state = assistanceDocumentState(appState.assistance_documents, documentId);
    try {
      if (!consent) throw new Error("Review the exact payload and confirm external processing first.");
      const payload = state.preview?.payload;
      if (!payload) throw new Error("Select excerpts and preview a payload within the configured limit first.");
      if (!navigator.clipboard) throw new Error("Clipboard access is unavailable.");
      await navigator.clipboard.writeText(payload.prompt);
      await invokeCommand("launch_claude_assistance", {
        editRoot: appState.workspace.edit_root,
        documentId,
        payloadDigest: payload.payload_digest,
        selectedExcerptLines: state.preview.selected_excerpt_lines,
        confirmed: true,
      });
      appState = {
        ...appState,
        assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
          launched: true,
          error: "",
        }),
      };
    } catch (error) {
      appState = {
        ...appState,
        assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
          error: String(error),
        }),
      };
    }
    render(appState);
    return true;
  }
  if (event.target.closest("[data-assistance-accept]")) {
    const response = document.querySelector("[data-assistance-response]")?.value ?? "";
    appState = {
      ...appState,
      assistance_documents: updateAssistanceState(appState.assistance_documents, documentId, {
        response,
        accepted_changelog: response,
      }),
    };
    render(appState);
    return true;
  }
  return false;
}

async function refreshWorkspaceAndLibrary() {
  const workspace = await invokeCommand("open_workspace", { editRoot: appState.workspace.edit_root });
  appState = { ...appState, workspace };
  await loadLibraryFolder(appState.library.folder.relative_path, "replace");
}

async function handleLibraryClick(event) {
  if (currentActivity(appState)?.destination !== "Library") return false;
  const treeToggle = event.target.closest("[data-library-tree-toggle]")?.dataset.libraryTreeToggle;
  if (treeToggle) {
    appState = { ...appState, library: toggleTreeFolder(appState.library, treeToggle) };
    render(appState);
    return true;
  }
  const folder = event.target.closest("[data-library-folder]")?.dataset.libraryFolder;
  if (folder) {
    void loadLibraryFolder(folder);
    return true;
  }
  const history = event.target.closest("[data-library-history]")?.dataset.libraryHistory;
  if (history) {
    const target = historyTarget(appState.library, history);
    if (target) void loadLibraryFolder(target, history);
    return true;
  }
  if (event.target.closest("[data-library-up]")) {
    const parent = appState.library.folder.parent;
    if (parent) void loadLibraryFolder(parent);
    return true;
  }
  if (event.target.closest("[data-library-refresh]")) {
    void loadLibraryFolder(appState.library.folder.relative_path, "replace");
    return true;
  }
  if (event.target.closest("[data-library-clear-search]")) {
    appState = {
      ...appState,
      library: { ...appState.library, results: null, query: "", page: 0, selection: [], detail: null },
    };
    render(appState);
    return true;
  }
  if (event.target.closest("[data-library-clear-selection]")) {
    appState = { ...appState, library: { ...appState.library, selection: [], detail: null } };
    render(appState);
    return true;
  }
  const pageDirection = event.target.closest("[data-library-page]")?.dataset.libraryPage;
  if (pageDirection) {
    const delta = pageDirection === "next" ? 1 : -1;
    appState = {
      ...appState,
      library: { ...appState.library, page: Math.max(0, appState.library.page + delta) },
    };
    render(appState);
    return true;
  }
  if (event.target.closest("[data-library-open-selected]")) {
    const selected = selectedEntries(appState.library)[0];
    if (selected?.kind === "folder") void loadLibraryFolder(selected.relative_path);
    return true;
  }
  if (event.target.closest("[data-library-add]")) {
    const paths = selectedEntries(appState.library).map((entry) => normalizeLibraryPath(entry.relative_path));
    try {
      await invokeCommand("add_library_documents", {
        editRoot: appState.workspace.edit_root,
        paths,
      });
      await refreshWorkspaceAndLibrary();
    } catch (error) {
      appState = { ...appState, error: String(error) };
      render(appState);
    }
    return true;
  }
  if (event.target.closest("[data-library-unregister]")) {
    const documentIds = selectedEntries(appState.library).map(entryDocumentId).filter(Boolean);
    try {
      await invokeCommand("unregister_library_documents", {
        editRoot: appState.workspace.edit_root,
        documentIds,
      });
      await refreshWorkspaceAndLibrary();
    } catch (error) {
      appState = { ...appState, error: String(error) };
      render(appState);
    }
    return true;
  }
  const openTarget = event.target.closest("[data-library-open-source]")
    ? "source"
    : event.target.closest("[data-library-open-release]")
      ? "release"
      : null;
  if (openTarget) {
    try {
      const request = libraryOpenRequest(appState.library.detail, openTarget);
      await invokeCommand(request.command, {
        editRoot: appState.workspace.edit_root,
        ...request.arguments,
      });
      appState = { ...appState, error: "" };
    } catch (error) {
      appState = { ...appState, error: String(error) };
    }
    render(appState);
    return true;
  }
  if (event.target.closest("[data-library-open-evidence]")) {
    appState = {
      ...appState,
      library: { ...appState.library, evidence_open: true },
    };
    render(appState);
    return true;
  }
  if (event.target.closest("[data-library-approver-sign-in]")) {
    try {
      const challenge = await invokeCommand("begin_approver_sign_in", {
        editRoot: appState.workspace.edit_root,
      });
      appState = {
        ...appState,
        library: { ...appState.library, approver_sign_in: { challenge }, detail_error: "" },
        error: "",
      };
    } catch (error) {
      appState = {
        ...appState,
        library: { ...appState.library, detail_error: String(error) },
      };
    }
    render(appState);
    return true;
  }
  const approverSignInCompletion = event.target.closest("[data-library-approver-sign-in-complete]")
    ?.dataset.libraryApproverSignInComplete;
  if (approverSignInCompletion) {
    try {
      const actor = await invokeCommand("complete_approver_sign_in", {
        challengeId: approverSignInCompletion,
      });
      appState = {
        ...appState,
        library: { ...appState.library, approver_sign_in: { actor } },
        error: "",
      };
    } catch (error) {
      appState = {
        ...appState,
        library: { ...appState.library, detail_error: String(error) },
      };
    }
    render(appState);
    return true;
  }
  const lifecycleAction = event.target.closest("[data-library-lifecycle-action]")
    ?.dataset.libraryLifecycleAction;
  if (lifecycleAction) {
    try {
      const request = lifecycleActionRequest(lifecycleAction, null, appState.library.detail);
      const detail = await invokeCommand(request.command, {
        editRoot: appState.workspace.edit_root,
        ...request.arguments,
      });
      appState = {
        ...appState,
        library: applyDocumentSelection(appState.library, detail, true),
        error: "",
      };
    } catch (error) {
      appState = {
        ...appState,
        library: { ...appState.library, detail_error: String(error) },
      };
    }
    render(appState);
    return true;
  }
  if (event.target.closest("[data-library-open-notes]")) {
    openDocumentNotes();
    return true;
  }
  if (event.target.closest("[data-library-open-assistance]")) {
    openDocumentAssistance();
    return true;
  }
  if (event.target.closest("[data-library-copy-permalink]")) {
    try {
      if (!navigator.clipboard) throw new Error("Clipboard access is unavailable.");
      await navigator.clipboard.writeText(appState.library.detail.permalink);
      appState = { ...appState, error: "" };
    } catch (error) {
      appState = { ...appState, error: String(error) };
    }
    render(appState);
    return true;
  }
  const row = event.target.closest("[data-library-entry]");
  if (row) {
    appState = {
      ...appState,
      library: toggleLibrarySelection(
        appState.library,
        row.dataset.libraryEntry,
        event.ctrlKey || event.metaKey,
      ),
    };
    render(appState);
    void loadSelectedDocument();
    return true;
  }
  return false;
}

async function applyNoteMutation(command, arguments_) {
  const documentId = arguments_.documentId;
  try {
    const detail = await invokeCommand(command, {
      editRoot: appState.workspace.edit_root,
      ...arguments_,
    });
    appState = {
      ...appState,
      note_documents: applyDocumentNotes(appState.note_documents, detail),
    };
  } catch (error) {
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, documentId, {
        error: String(error),
      }),
    };
  }
  render(appState);
}

async function handleNotesClick(event) {
  const activity = currentActivity(appState);
  if (activity?.task !== "Notes") return false;
  const documentId = activity.document_id;
  const edit = event.target.closest("[data-note-edit]")?.dataset.noteEdit;
  if (edit) {
    const note = noteDocumentState(appState.note_documents, documentId)
      .detail?.notes.find((candidate) => candidate.id === edit);
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, documentId, {
        editing_id: edit,
        editing_body: note?.body ?? "",
        delete_id: null,
      }),
    };
    render(appState);
    return true;
  }
  if (event.target.closest("[data-note-edit-cancel]")) {
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, documentId, {
        editing_id: null,
        editing_body: null,
      }),
    };
    render(appState);
    return true;
  }
  const remove = event.target.closest("[data-note-delete-request]")?.dataset.noteDeleteRequest;
  if (remove) {
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, documentId, {
        editing_id: null,
        delete_id: remove,
      }),
    };
    render(appState);
    return true;
  }
  if (event.target.closest("[data-note-delete-cancel]")) {
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, documentId, { delete_id: null }),
    };
    render(appState);
    return true;
  }
  const confirm = event.target.closest("[data-note-delete-confirm]")?.dataset.noteDeleteConfirm;
  if (confirm) {
    await applyNoteMutation("remove_document_note", { documentId, noteId: confirm });
    return true;
  }
  return false;
}

async function handleClick(event) {
  const externalUrl = event.target.closest("[data-open-external]")?.dataset.openExternal;
  if (externalUrl) {
    try {
      await invokeCommand("open_external_url", { url: externalUrl });
      appState = { ...appState, error: "" };
    } catch (error) {
      if (currentActivity(appState)?.destination === "Library") {
        appState = {
          ...appState,
          library: { ...appState.library, detail_error: String(error) },
        };
      } else {
        appState = {
          ...appState,
          configuration: { ...appState.configuration, notice: "", error: String(error) },
        };
      }
    }
    render(appState);
    return;
  }

  const recentRemove = event.target.closest("[data-recent-library-remove]")?.dataset.recentLibraryRemove;
  if (recentRemove) {
    appState = {
      ...appState,
      preferences: removeRecentLibrary(appState.preferences, recentRemove),
    };
    await persistPreferences(appState);
    render(appState);
    return;
  }

  const recentOpen = event.target.closest("[data-recent-library-open]")?.dataset.recentLibraryOpen;
  if (recentOpen) {
    appState = { ...appState, setup_edit_root: recentOpen, error: "" };
    render(appState);
    try {
      const workspace = await invokeCommand("open_workspace", { editRoot: recentOpen });
      await activateWorkspace(workspace);
    } catch (error) {
      appState = { ...appState, error: String(error) };
      render(appState);
    }
    return;
  }

  const directoryTarget = event.target.closest("[data-directory-target]")?.dataset.directoryTarget;
  if (directoryTarget) {
    try {
      const selected = await invokeCommand("select_directory", {});
      if (selected) document.getElementById(directoryTarget).value = selected;
    } catch (error) {
      appState = { ...appState, error: String(error) };
      render(appState);
    }
    return;
  }

  const destination = event.target.closest("[data-destination]")?.dataset.destination;
  if (destination) {
    openDestination(destination);
    return;
  }

  const configurationRouteButton = event.target.closest("[data-configuration-route]");
  const configurationRoute = configurationRouteButton?.dataset.configurationRoute;
  if (configurationRoute) {
    try {
      const activity = currentActivity(appState);
      const configuration = setConfigurationRoute(appState.configuration, configurationRoute);
      appState = openActivity({ ...appState, configuration }, {
        ...activity,
        label: `Configuration · ${configurationRouteButton.querySelector("strong").textContent}`,
        route_state: { ...activity.route_state, route: configurationRoute },
      });
    } catch (error) {
      appState = {
        ...appState,
        configuration: { ...appState.configuration, notice: "", error: String(error) },
      };
    }
    render(appState);
    return;
  }

  const configurationSecondary = event.target.closest("[data-configuration-secondary]")?.dataset.configurationSecondary;
  if (configurationSecondary) {
    try {
      appState = {
        ...appState,
        configuration: openConfigurationSecondary(appState.configuration, configurationSecondary),
      };
    } catch (error) {
      appState = {
        ...appState,
        configuration: { ...appState.configuration, notice: "", error: String(error) },
      };
    }
    render(appState);
    return;
  }

  if (event.target.closest("[data-configuration-secondary-close]")) {
    appState = {
      ...appState,
      configuration: closeConfigurationSecondary(appState.configuration),
    };
    render(appState);
    return;
  }

  const configurationFolder = event.target.closest("[data-configuration-folder]")?.dataset.configurationFolder;
  if (configurationFolder) {
    try {
      appState = {
        ...appState,
        configuration: selectConfigurationFolder(appState.configuration, configurationFolder),
      };
    } catch (error) {
      appState = {
        ...appState,
        configuration: { ...appState.configuration, notice: "", error: String(error) },
      };
    }
    render(appState);
    return;
  }

  if (await handleNotesClick(event)) return;
  if (await handleAssistanceClick(event)) return;
  if (await handleLibraryClick(event)) return;

  const verifyReport = event.target.closest("[data-report-verify]")?.dataset.reportVerify;
  if (verifyReport) {
    appState = {
      ...appState,
      audit_reports: { ...appState.audit_reports, loading: true, error: "" },
    };
    render(appState);
    try {
      const snapshot = await invokeCommand("verify_audit_report", {
        editRoot: appState.workspace.edit_root,
        eventId: verifyReport,
      });
      appState = {
        ...appState,
        audit_reports: applyAuditReportSnapshot(appState.audit_reports, snapshot, "Report verified."),
      };
    } catch (error) {
      appState = {
        ...appState,
        audit_reports: { ...appState.audit_reports, loading: false, error: String(error) },
      };
    }
    render(appState);
    return;
  }
  const openReportFolder = event.target.closest("[data-report-open-folder]")?.dataset.reportOpenFolder;
  if (openReportFolder) {
    try {
      await invokeCommand("open_audit_report_folder", {
        editRoot: appState.workspace.edit_root,
        eventId: openReportFolder,
      });
      appState = { ...appState, audit_reports: { ...appState.audit_reports, error: "" } };
    } catch (error) {
      appState = {
        ...appState,
        audit_reports: { ...appState.audit_reports, error: String(error) },
      };
    }
    render(appState);
    return;
  }
  const reportPage = event.target.closest("[data-report-page]")?.dataset.reportPage;
  if (reportPage) {
    const delta = reportPage === "next" ? 1 : -1;
    appState = {
      ...appState,
      audit_reports: { ...appState.audit_reports, page: Math.max(0, appState.audit_reports.page + delta) },
    };
    render(appState);
    return;
  }

  if (event.target.closest("[data-release-verify-all]")) {
    void loadReleases("verify_all_releases");
    return;
  }
  const openRelease = event.target.closest("[data-release-open]");
  if (openRelease) {
    try {
      await invokeCommand("open_release_pdf", {
        editRoot: appState.workspace.edit_root,
        documentId: openRelease.dataset.documentId,
        releaseId: openRelease.dataset.releaseOpen,
      });
      appState = { ...appState, releases: { ...appState.releases, error: "" } };
    } catch (error) {
      appState = {
        ...appState,
        releases: { ...appState.releases, error: String(error) },
      };
    }
    render(appState);
    return;
  }
  const verifyRelease = event.target.closest("[data-release-verify]");
  if (verifyRelease) {
    void loadReleases("verify_release", {
      documentId: verifyRelease.dataset.documentId,
      releaseId: verifyRelease.dataset.releaseVerify,
    });
    return;
  }
  const startReview = event.target.closest("[data-periodic-review-start]")?.dataset.periodicReviewStart;
  if (startReview) {
    try {
      await invokeCommand("start_periodic_review", {
        editRoot: appState.workspace.edit_root,
        documentId: startReview,
      });
      await loadPeriodicReviews();
    } catch (error) {
      appState = {
        ...appState,
        periodic_reviews: { ...appState.periodic_reviews, error: String(error) },
      };
      render(appState);
    }
    return;
  }
  const releasePage = event.target.closest("[data-release-page]")?.dataset.releasePage;
  if (releasePage) {
    const delta = releasePage === "next" ? 1 : -1;
    appState = {
      ...appState,
      releases: { ...appState.releases, page: Math.max(0, appState.releases.page + delta) },
    };
    render(appState);
    return;
  }

  const activityRemove = event.target.closest("[data-activity-remove]")?.dataset.activityRemove;
  if (activityRemove) {
    appState = closeActivity(appState, activityRemove);
    render(appState);
    return;
  }
  const activityKeyValue = event.target.closest("[data-activity-key]")?.dataset.activityKey;
  if (activityKeyValue) {
    appState = { ...appState, current_key: activityKeyValue, flyout: null };
    const selectedActivity = currentActivity(appState);
    if (selectedActivity?.destination === "Configuration") {
      appState = {
        ...appState,
        configuration: setConfigurationRoute(
          appState.configuration,
          selectedActivity.route_state?.route ?? "workspace",
        ),
      };
    }
    render(appState);
    const activity = currentActivity(appState);
    if (activity?.task === "Notes" && !noteDocumentState(appState.note_documents, activity.document_id).detail) {
      void loadDocumentNotes(activity.document_id);
    } else if (activity?.task === "Claude assistance" && !assistanceDocumentState(appState.assistance_documents, activity.document_id).availability) {
      void loadClaudeAssistanceAvailability(activity.document_id);
    } else if (activity?.destination === "Releases" && appState.releases.rows.length === 0) {
      void loadReleases();
    } else if (activity?.destination === "Audit & Reports") {
      if (appState.audit_reports.rows.length === 0) void loadAuditReports();
      if (appState.periodic_reviews.markers.length === 0) void loadPeriodicReviews();
    } else if (activity?.destination === "Maintenance" && !appState.maintenance.lock_status) {
      void loadWorkspaceLockStatus();
    } else if (activity?.destination === "Configuration") {
      if (!appState.configuration.snapshot) void loadWorkspaceConfiguration();
      if (!appState.assistance_policy.value) void loadClaudeAssistancePolicy();
    }
    return;
  }

  const savedRemove = event.target.closest("[data-saved-remove]")?.dataset.savedRemove;
  if (savedRemove) {
    appState = {
      ...appState,
      preferences: {
        ...appState.preferences,
        saved_views: appState.preferences.saved_views.filter((view) => view.id !== savedRemove),
      },
    };
    persistPreferences(appState);
    render(appState);
    return;
  }
  const savedKey = event.target.closest("[data-saved-key]")?.dataset.savedKey;
  if (savedKey) {
    const view = appState.preferences.saved_views.find((candidate) => candidate.id === savedKey);
    if (view?.workspace_id !== appState.workspace?.workspace_id) {
      appState = { ...appState, error: "That saved view's workspace is unavailable. You can remove the saved view." };
    } else {
      appState = openActivity(appState, activityFromSavedView(view));
      if (view.task === "Notes" && view.document_id) {
        void loadDocumentNotes(view.document_id);
      } else if (view.destination === "Library") {
        appState = {
          ...appState,
          library: { ...appState.library, sort: view.route_state?.sort ?? "name" },
        };
        void loadLibraryFolder(
          view.route_state?.folder ?? ".",
          "replace",
          view.document_id ?? null,
        );
      } else if (view.destination === "Releases") {
        void loadReleases();
      } else if (view.destination === "Audit & Reports") {
        void loadPeriodicReviews();
        void loadAuditReports();
      } else if (view.destination === "Configuration") {
        appState = {
          ...appState,
          configuration: setConfigurationRoute(
            appState.configuration,
            view.route_state?.route ?? "workspace",
          ),
        };
        void loadWorkspaceConfiguration();
        void loadClaudeAssistancePolicy();
      }
    }
    render(appState);
    return;
  }

  const flyout = event.target.closest("[data-flyout]")?.dataset.flyout;
  if (flyout) {
    appState = { ...appState, flyout: appState.flyout === flyout ? null : flyout };
    render(appState);
  }
}

async function handleSubmit(event) {
  const configurationMutation = event.target.dataset.configurationForm;
  if (configurationMutation) {
    event.preventDefault();
    try {
      const request = configurationMutationRequest(
        configurationMutation,
        new FormData(event.target),
        appState.configuration.selected_folder,
      );
      const result = await invokeCommand(request.command, {
        editRoot: appState.workspace.edit_root,
        ...request.arguments,
      });
      if (configurationMutation === "identity-source-start") {
        appState = {
          ...appState,
          configuration: {
            ...appState.configuration,
            identity_setup: { challenge: result, last_group_id: request.arguments.groupId },
            notice: "",
            error: "",
          },
        };
        render(appState);
        return;
      }
      if (configurationMutation === "identity-source-complete") {
        appState = {
          ...appState,
          configuration: { ...appState.configuration, identity_setup: { preview: result }, notice: "", error: "" },
        };
        render(appState);
        return;
      }
      if (configurationMutation === "global-entra") {
        appState = {
          ...appState,
          configuration: {
            ...appState.configuration,
            snapshot: {
              ...appState.configuration.snapshot,
              global_entra_configuration: result,
            },
            notice: "Application Entra configuration saved.",
            error: "",
          },
        };
        render(appState);
        return;
      }
      const notices = {
        "review-interval": "Default review interval saved.",
        "document-type": "Document type saved.",
        "confidentiality-type": "Confidentiality type saved.",
        "confidentiality-policy": "Folder confidentiality policy saved.",
        "remove-confidentiality-policy": "Folder confidentiality policy removed.",
        "workflow-policy": "Workflow roles saved.",
        "remove-workflow-policy": "Folder workflow exception removed.",
        "global-entra": "Application Entra configuration saved.",
        notifications: "Notification transport saved.",
      };
      const notice = notices[configurationMutation] ?? "Configuration saved.";
      appState = {
        ...appState,
        workspace: result.workspace,
        configuration: {
          ...applyConfigurationSnapshot(appState.configuration, result, notice),
          identity_setup: null,
        },
      };
    } catch (error) {
      appState = {
        ...appState,
        configuration: { ...appState.configuration, notice: "", error: String(error) },
      };
    }
    render(appState);
    return;
  }
  if (["library-document-control-form", "library-confidentiality-form"].includes(event.target.id)) {
    event.preventDefault();
    try {
      const values = new FormData(event.target);
      const request = event.target.id === "library-document-control-form"
        ? documentControlUpdateRequest(values, appState.library.detail)
        : confidentialityUpdateRequest(values, appState.library.detail);
      const detail = await invokeCommand(request.command, {
        editRoot: appState.workspace.edit_root,
        ...request.arguments,
      });
      appState = {
        ...appState,
        library: applyDocumentSelection(appState.library, detail),
        error: "",
      };
    } catch (error) {
      appState = {
        ...appState,
        library: { ...appState.library, detail_error: String(error) },
      };
    }
    render(appState);
    return;
  }
  const lifecycleAction = event.target.dataset.libraryLifecycleForm;
  if (lifecycleAction) {
    event.preventDefault();
    const values = new FormData(event.target);
    const draft = {
      reason: String(values.get("reason") ?? ""),
      confirmed: values.get("confirmed") === "yes",
    };
    try {
      const request = lifecycleActionRequest(lifecycleAction, values, appState.library.detail);
      const detail = await invokeCommand(request.command, {
        editRoot: appState.workspace.edit_root,
        ...request.arguments,
      });
      appState = {
        ...appState,
        library: applyDocumentSelection(appState.library, detail, true),
        error: "",
      };
    } catch (error) {
      appState = {
        ...appState,
        library: {
          ...appState.library,
          detail_error: String(error),
          lifecycle_drafts: {
            ...appState.library.lifecycle_drafts,
            [lifecycleAction]: draft,
          },
        },
      };
    }
    render(appState);
    return;
  }
  if (event.target.matches("[data-release-withdraw-form]")) {
    event.preventDefault();
    const request = releaseWithdrawalRequest(new FormData(event.target));
    await loadReleases(request.command, request.arguments);
    return;
  }
  if (event.target.id === "audit-report-generate-form") {
    event.preventDefault();
    const request = auditReportRequest(new FormData(event.target));
    appState = {
      ...appState,
      audit_reports: { ...appState.audit_reports, loading: true, error: "", notice: "" },
    };
    render(appState);
    try {
      const snapshot = await invokeCommand(request.command, {
        editRoot: appState.workspace.edit_root,
        ...request.arguments,
      });
      const format = request.arguments.request.format.toUpperCase();
      appState = {
        ...appState,
        audit_reports: applyAuditReportSnapshot(
          appState.audit_reports,
          snapshot,
          `${format} audit report generated.`,
        ),
      };
    } catch (error) {
      appState = {
        ...appState,
        audit_reports: { ...appState.audit_reports, loading: false, error: String(error) },
      };
    }
    render(appState);
    return;
  }
  if (event.target.id === "audit-report-filter-form") {
    event.preventDefault();
    const query = String(new FormData(event.target).get("query") ?? "");
    appState = {
      ...appState,
      audit_reports: { ...appState.audit_reports, query, page: 0 },
    };
    render(appState);
    return;
  }
  if (event.target.matches("[data-periodic-review-form]")) {
    event.preventDefault();
    const action = event.submitter?.value;
    try {
      const request = periodicReviewRequest(action, new FormData(event.target));
      const outcome = await invokeCommand(request.command, {
        editRoot: appState.workspace.edit_root,
        ...request.arguments,
      });
      const notice = action === "result"
        ? "Periodic-review result recorded."
        : action === "cancel"
          ? "Periodic review cancelled; the release schedule was not changed."
          : outcome.status === "failed"
            ? ""
            : `Reminder ${outcome.status}.`;
      await loadPeriodicReviews(notice);
      if (action === "reminder" && outcome.status === "failed") {
        appState = {
          ...appState,
          periodic_reviews: {
            ...appState.periodic_reviews,
            error: `Reminder failed: ${outcome.detail}`,
          },
        };
        render(appState);
      }
    } catch (error) {
      appState = {
        ...appState,
        periodic_reviews: { ...appState.periodic_reviews, error: String(error) },
      };
      render(appState);
    }
    return;
  }
  if (event.target.id === "claude-policy-form") {
    event.preventDefault();
    const form = new FormData(event.target);
    const allowedConfidentialityTypeIds = String(form.get("allowedIds") ?? "")
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
    try {
      await invokeCommand("configure_claude_assistance", {
        editRoot: appState.workspace.edit_root,
        enabled: form.get("enabled") === "on",
        allowedConfidentialityTypeIds,
        maxPayloadChars: Number(form.get("maxPayloadChars")),
      });
      await loadClaudeAssistancePolicy();
    } catch (error) {
      appState = {
        ...appState,
        assistance_policy: { ...appState.assistance_policy, error: String(error) },
      };
      render(appState);
    }
    return;
  }
  if (event.target.id === "release-filter-form") {
    event.preventDefault();
    const query = String(new FormData(event.target).get("query") ?? "");
    appState = { ...appState, releases: { ...appState.releases, query, page: 0 } };
    render(appState);
    return;
  }
  if (event.target.id === "workspace-backup-form") {
    event.preventDefault();
    const archivePath = String(new FormData(event.target).get("archivePath") ?? "").trim();
    appState = {
      ...appState,
      maintenance: { ...appState.maintenance, notice: "", error: "" },
    };
    render(appState);
    try {
      const outcome = await invokeCommand("backup_workspace", {
        editRoot: appState.workspace.edit_root,
        archivePath,
      });
      appState = {
        ...appState,
        maintenance: {
          ...appState.maintenance,
          backup_outcome: outcome,
          notice: "Backup created.",
          error: "",
        },
      };
    } catch (error) {
      appState = {
        ...appState,
        maintenance: { ...appState.maintenance, error: String(error) },
      };
    }
    render(appState);
    return;
  }
  if (event.target.id === "workspace-lock-status-form") {
    event.preventDefault();
    await loadWorkspaceLockStatus();
    return;
  }

  if (event.target.id === "workspace-lock-config-form") {
    event.preventDefault();
    const form = new FormData(event.target);
    await applyWorkspaceLockMutation(
      "configure_workspace_lock_staleness",
      {
        hours: Number(form.get("hours")),
        confirmed: form.get("confirmed") === "on",
      },
      "Workspace lock-staleness threshold saved.",
    );
    return;
  }
  if (event.target.id === "workspace-restore-form") {
    event.preventDefault();
    appState = {
      ...appState,
      maintenance: { ...appState.maintenance, notice: "", error: "" },
    };
    render(appState);
    try {
      const outcome = await invokeCommand(
        "restore_workspace_backup",
        workspaceRestoreRequest(new FormData(event.target)),
      );
      appState = {
        ...appState,
        maintenance: {
          ...appState.maintenance,
          restore_outcome: outcome,
          notice: "Backup verified and restored.",
          error: "",
        },
      };
    } catch (error) {
      appState = {
        ...appState,
        maintenance: { ...appState.maintenance, error: String(error) },
      };
    }
    render(appState);
    return;
  }
  if (event.target.id === "document-note-compose-form") {
    event.preventDefault();
    const activity = currentActivity(appState);
    const form = new FormData(event.target);
    const body = String(form.get("body") ?? "");
    const author = String(form.get("author") ?? "").trim();
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, activity.document_id, {
        compose_body: body,
        compose_author: author,
      }),
    };
    await applyNoteMutation("add_document_note", {
      documentId: activity.document_id,
      body,
      author: author || null,
    });
    return;
  }
  if (event.target.id === "document-note-edit-form") {
    event.preventDefault();
    const activity = currentActivity(appState);
    const form = new FormData(event.target);
    const body = String(form.get("body") ?? "");
    appState = {
      ...appState,
      note_documents: updateNoteDocumentState(appState.note_documents, activity.document_id, {
        editing_body: body,
      }),
    };
    await applyNoteMutation("edit_document_note", {
      documentId: activity.document_id,
      noteId: event.target.dataset.noteId,
      body,
    });
    return;
  }
  if (event.target.id === "library-reassociate-form") {
    event.preventDefault();
    const path = String(new FormData(event.target).get("path") ?? "").trim();
    try {
      await invokeCommand("reassociate_library_document", {
        editRoot: appState.workspace.edit_root,
        documentId: appState.library.detail.document_id,
        path,
      });
      await refreshWorkspaceAndLibrary();
    } catch (error) {
      appState = { ...appState, error: String(error) };
      render(appState);
    }
    return;
  }
  if (event.target.id === "library-search-form") {
    event.preventDefault();
    const form = new FormData(event.target);
    const query = String(form.get("query") ?? "").trim();
    const entireLibrary = form.get("entireLibrary") === "on";
    if (!query) {
      appState = {
        ...appState,
        library: { ...appState.library, query: "", results: null, page: 0, selection: [], detail: null },
      };
      render(appState);
      return;
    }
    try {
      const results = await invokeCommand("search_library", {
        editRoot: appState.workspace.edit_root,
        folder: entireLibrary ? "." : appState.library.folder.relative_path,
        query,
      });
      appState = {
        ...appState,
        library: {
          ...appState.library,
          query,
          entire_library: entireLibrary,
          results,
          page: 0,
          selection: [],
          detail: null,
        },
        error: "",
      };
    } catch (error) {
      appState = { ...appState, error: String(error) };
    }
    render(appState);
    return;
  }
  if (!["open-workspace-form", "initialize-workspace-form"].includes(event.target.id)) return;
  event.preventDefault();
  try {
    const request = workspaceSetupRequest(event.target.id, new FormData(event.target));
    const workspace = await invokeCommand(request.command, request.arguments);
    await activateWorkspace(workspace, request.lockOptions ?? {});
  } catch (error) {
    appState = { ...appState, error: String(error) };
    render(appState);
  }
}

function handleChange(event) {
  const activity = currentActivity(appState);
  if (activity?.task === "Claude assistance" && event.target.matches("[data-assistance-response]")) {
    appState = {
      ...appState,
      assistance_documents: updateAssistanceState(appState.assistance_documents, activity.document_id, {
        response: event.target.value,
      }),
    };
    return;
  }
  if (activity?.task === "Claude assistance" && event.target.matches("[data-assistance-changelog]")) {
    appState = {
      ...appState,
      assistance_documents: updateAssistanceState(appState.assistance_documents, activity.document_id, {
        accepted_changelog: event.target.value,
      }),
    };
    return;
  }
  if (event.target.matches("[data-release-page-size]")) {
    appState = {
      ...appState,
      releases: { ...appState.releases, page_size: Number(event.target.value), page: 0 },
    };
    render(appState);
    return;
  }
  if (event.target.matches("[data-report-page-size]")) {
    appState = {
      ...appState,
      audit_reports: {
        ...appState.audit_reports,
        page_size: Number(event.target.value),
        page: 0,
      },
    };
    render(appState);
    return;
  }
  if (event.target.matches("[data-library-page-size]")) {
    appState = {
      ...appState,
      library: { ...appState.library, page_size: Number(event.target.value), page: 0 },
    };
    render(appState);
    return;
  }
  if (!event.target.matches("[data-library-sort]")) return;
  appState = { ...appState, library: { ...appState.library, sort: event.target.value, page: 0 } };
  updateLibraryActivity(appState.library.folder.relative_path);
  render(appState);
}

function handleDoubleClick(event) {
  const row = event.target.closest("[data-library-entry][data-library-kind='folder']");
  if (row) void loadLibraryFolder(row.dataset.libraryEntry);
}

function handleKeyDown(event) {
  if (event.key !== "Enter") return;
  const row = event.target.closest("[data-library-entry][data-library-kind='folder']");
  if (row) {
    event.preventDefault();
    void loadLibraryFolder(row.dataset.libraryEntry);
  }
}

let appState = createInitialState();

export async function closeWorkspaceSession(state, invoke, destroy) {
  if (!state.workspace) return false;
  const owner = state.maintenance.lock_status?.lock;
  if (!owner) {
    await destroy();
    return false;
  }
  try {
    await invoke("release_workspace_lock", {
      editRoot: state.workspace.edit_root,
      owner,
      confirmed: true,
    });
  } finally {
    await destroy();
  }
  return true;
}

async function registerWindowCloseHandler() {
  const appWindow = globalThis.__TAURI__?.window?.getCurrentWindow?.();
  if (!appWindow?.onCloseRequested) return;
  await appWindow.onCloseRequested(async (event) => {
    if (!appState.workspace) return;
    event.preventDefault();
    try {
      await closeWorkspaceSession(appState, invokeCommand, () => appWindow.destroy());
    } catch (error) {
      console.warn("Could not remove workspace advisory lock during close", error);
    }
  });
}

const handledPermalinks = new Set();
let permalinkQueue = Promise.resolve();

function queuePermalinks(urls) {
  for (const value of urls ?? []) {
    const uri = String(value);
    if (handledPermalinks.has(uri)) continue;
    handledPermalinks.add(uri);
    permalinkQueue = permalinkQueue.then(async () => {
      try {
        await openPermalink(uri);
      } catch (error) {
        handledPermalinks.delete(uri);
        appState = { ...appState, error: String(error) };
        render(appState);
      }
    });
  }
}

async function registerDeepLinkHandler() {
  const deepLink = globalThis.__TAURI__?.deepLink;
  if (!deepLink?.onOpenUrl || !deepLink?.getCurrent) return;
  await deepLink.onOpenUrl(queuePermalinks);
  queuePermalinks(await deepLink.getCurrent());
}

async function start() {
  try {
    const preferences = await invokeCommand("load_preferences", {});
    appState = createInitialState(preferences);
  } catch (error) {
    console.warn("Using default preferences", error);
  }

  document.addEventListener("click", handleClick);
  document.addEventListener("submit", handleSubmit);
  document.addEventListener("change", handleChange);
  document.addEventListener("dblclick", handleDoubleClick);
  document.addEventListener("keydown", handleKeyDown);
  await registerWindowCloseHandler();
  await registerDeepLinkHandler();
  document.querySelector("#collapse-sidebar").addEventListener("click", () => {
    appState = {
      ...appState,
      sidebar_overlay: false,
      preferences: { ...appState.preferences, sidebar_expanded: false },
    };
    persistPreferences(appState);
    render(appState);
  });
  document.querySelector("#open-sidebar").addEventListener("click", () => {
    appState = { ...appState, sidebar_overlay: !appState.sidebar_overlay, flyout: null };
    render(appState);
  });
  document.querySelector("#bookmark-view").addEventListener("click", () => {
    const activity = bookmarkActivity(appState);
    if (!activity) return;
    appState = {
      ...appState,
      preferences: toggleSavedView(appState.preferences, activity),
    };
    persistPreferences(appState);
    render(appState);
  });
  render(appState);
}

if (typeof document !== "undefined") {
  start();
}
