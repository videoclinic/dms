import {
  applyLibrarySnapshot,
  createLibraryState,
  entryDocumentId,
  historyTarget,
  libraryMarkup,
  membershipKind,
  normalizeLibraryPath,
  selectedEntries,
  toggleLibrarySelection,
} from "./library.mjs";

const DESTINATIONS = [
  ["Library", "▦"],
  ["Releases", "□"],
  ["Audit & Reports", "≣"],
  ["Maintenance", "◇"],
  ["Configuration", "⚙"],
];

export function defaultPreferences() {
  return { sidebar_expanded: true, saved_views: [] };
}

export function createInitialState(preferences = defaultPreferences()) {
  return {
    preferences: {
      sidebar_expanded: preferences.sidebar_expanded !== false,
      saved_views: Array.isArray(preferences.saved_views) ? preferences.saved_views : [],
    },
    activities: [],
    current_key: null,
    workspace: null,
    library: createLibraryState(),
    sidebar_overlay: false,
    flyout: null,
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
  return { ...state, activities, current_key: key, sidebar_overlay: false, flyout: null };
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
  if (!activity || activity.destination !== "Library") return activity;
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

function setupMarkup(error) {
  return `<section class="card"><span class="badge">Phase 1 desktop shell</span><h2>Open a DMS workspace</h2><p>Choose an edit root that already contains <code>.dms/workspace.json</code>. Workspace creation remains explicit in the headless CLI until the configuration phase lands.</p><form id="open-workspace-form" class="form-row"><div class="field"><label for="edit-root">Edit root</label><input id="edit-root" name="editRoot" required autocomplete="off" placeholder="C:\\DMS\\Edit or /Users/name/DMS/Edit"></div><button class="button" type="submit">Open workspace</button></form><p class="status" role="alert">${escapeHtml(error)}</p></section>`;
}

function activityMarkup(state, activity) {
  if (!activity) {
    return setupMarkup(state.error);
  }
  const workspace = state.workspace;
  if (activity.destination === "Library") {
    return libraryMarkup(workspace, activity, state.library, state.error);
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
  mainContent.classList.toggle("library-active", activity?.destination === "Library");
  mainContent.innerHTML = state.workspace
    ? activityMarkup(state, activity)
    : setupMarkup(state.error);

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
    appState = { ...appState, current_key: null, sidebar_overlay: false, flyout: null };
    render(appState);
    return;
  }
  const folder = destination === "Library" ? "." : null;
  const label = folder ? "Library · /" : destination;
  appState = openActivity(appState, {
    workspace_id: appState.workspace.workspace_id,
    destination,
    task: destination,
    label,
    document_id: null,
    route_state: folder ? { folder } : {},
  });
  render(appState);
  if (destination === "Library") {
    void loadLibraryFolder(folder, "replace");
  }
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

async function refreshWorkspaceAndLibrary() {
  const workspace = await invokeCommand("open_workspace", { editRoot: appState.workspace.edit_root });
  appState = { ...appState, workspace };
  await loadLibraryFolder(appState.library.folder.relative_path, "replace");
}

async function handleLibraryClick(event) {
  if (currentActivity(appState)?.destination !== "Library") return false;
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

async function handleClick(event) {
  const destination = event.target.closest("[data-destination]")?.dataset.destination;
  if (destination) {
    openDestination(destination);
    return;
  }

  if (await handleLibraryClick(event)) return;

  const activityRemove = event.target.closest("[data-activity-remove]")?.dataset.activityRemove;
  if (activityRemove) {
    appState = closeActivity(appState, activityRemove);
    render(appState);
    return;
  }
  const activityKeyValue = event.target.closest("[data-activity-key]")?.dataset.activityKey;
  if (activityKeyValue) {
    appState = { ...appState, current_key: activityKeyValue, flyout: null };
    render(appState);
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
      if (view.destination === "Library") {
        appState = {
          ...appState,
          library: { ...appState.library, sort: view.route_state?.sort ?? "name" },
        };
        void loadLibraryFolder(
          view.route_state?.folder ?? ".",
          "replace",
          view.document_id ?? null,
        );
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
  if (event.target.id !== "open-workspace-form") return;
  event.preventDefault();
  const editRoot = new FormData(event.target).get("editRoot");
  try {
    const workspace = await invokeCommand("open_workspace", { editRoot });
    appState = { ...appState, workspace, error: "" };
    openDestination("Library");
  } catch (error) {
    appState = { ...appState, error: String(error) };
    render(appState);
  }
}

function handleChange(event) {
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
