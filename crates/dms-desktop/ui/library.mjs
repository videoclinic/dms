export function normalizeLibraryPath(path) {
  const normalized = String(path ?? ".").replaceAll("\\", "/").replace(/^\/+|\/+$/g, "");
  return normalized || ".";
}

export function clampLibraryDetailWidth(width, maximum = 640) {
  const numeric = Number(width);
  const boundedMaximum = Math.max(280, Math.min(640, Number(maximum) || 640));
  return Math.min(boundedMaximum, Math.max(280, Number.isFinite(numeric) ? numeric : 420));
}

export function resizeLibraryDetailWidth(startWidth, startX, currentX, maximum = 640) {
  return clampLibraryDetailWidth(Number(startWidth) - (Number(currentX) - Number(startX)), maximum);
}

export function clampLibraryTreeWidth(width, maximum = 420) {
  const numeric = Number(width);
  const boundedMaximum = Math.max(170, Math.min(420, Number(maximum) || 420));
  return Math.min(boundedMaximum, Math.max(170, Number.isFinite(numeric) ? numeric : 230));
}

export function resizeLibraryTreeWidth(startWidth, startX, currentX, maximum = 420) {
  return clampLibraryTreeWidth(Number(startWidth) + (Number(currentX) - Number(startX)), maximum);
}

export const LIBRARY_PANE_SIDES = ["tree", "detail"];

function isLibraryPaneSide(side) {
  return LIBRARY_PANE_SIDES.includes(String(side));
}

export function isLibraryPaneFolded(library, side) {
  if (!isLibraryPaneSide(side)) return false;
  return Boolean(library?.[`${side}_folded`]);
}

export function setLibraryPaneFolded(library, side, isFolded) {
  if (!isLibraryPaneSide(side)) return library;
  return { ...library, [`${side}_folded`]: Boolean(isFolded) };
}

export function toggleLibraryPaneFold(library, side) {
  return setLibraryPaneFolded(library, side, !isLibraryPaneFolded(library, side));
}

export const DEFAULT_SELECTION_OPEN = {
  control: true,
  schedule: true,
  revision: true,
  releases: true,
  actions: true,
};

export function selectionSectionOpen(library, key) {
  const open = library?.selection_open ?? DEFAULT_SELECTION_OPEN;
  if (Object.prototype.hasOwnProperty.call(open, key)) return Boolean(open[key]);
  return Boolean(DEFAULT_SELECTION_OPEN[key] ?? true);
}

export function setSelectionSectionOpen(library, key, isOpen) {
  return {
    ...library,
    selection_open: {
      ...DEFAULT_SELECTION_OPEN,
      ...(library.selection_open ?? {}),
      [key]: Boolean(isOpen),
    },
  };
}

export function createLibraryState() {
  return {
    tree: [],
    folder: { relative_path: ".", parent: null, entries: [] },
    selection: [],
    detail: null,
    detail_error: "",
    reassociate_path: "",
    evidence_open: false,
    selection_open: { ...DEFAULT_SELECTION_OPEN },
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
    show_draft_documents: true,
    show_available_to_add: true,
    show_unsupported_files: true,
    show_moved_documents: true,
    detail_width: 420,
    tree_width: 230,
    tree_folded: false,
    detail_folded: false,
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
    return [path, { name: folder.name, path, counters: folder.counters ?? {}, children: [] }];
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
    reassociate_path: "",
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
  if (entry?.membership?.lost_source) return "lost_source";
  return null;
}

export function entryDocumentId(entry) {
  return entry?.membership?.in_library?.document_id
    ?? entry?.membership?.lost_source?.document_id
    ?? entry?.document?.id
    ?? null;
}

export function filterLibraryEntries(entries, library) {
  return (entries ?? []).filter((entry) => {
    if (entry.kind === "folder") return true;
    const membership = membershipKind(entry);
    if (membership === "lost_source") return library.show_moved_documents !== false;
    if (membership === "in_library" && entry.document?.lifecycle === "draft") {
      return library.show_draft_documents;
    }
    if (membership === "not_in_library") return library.show_available_to_add;
    if (membership === "unsupported") return library.show_unsupported_files;
    return true;
  });
}

export function toggleLibraryVisibility(library, key) {
  if (![
    "show_draft_documents",
    "show_available_to_add",
    "show_unsupported_files",
    "show_moved_documents",
  ].includes(key)) {
    return library;
  }
  const next = { ...library, [key]: !library[key], page: 0 };
  const entries = filterLibraryEntries(next.results ?? next.folder?.entries ?? [], next);
  const visiblePaths = new Set(entries.map((entry) => normalizeLibraryPath(entry.relative_path)));
  const selection = next.selection.filter((path) => visiblePaths.has(path));
  return {
    ...next,
    selection,
    detail: selection.length === next.selection.length ? next.detail : null,
    detail_error: selection.length === next.selection.length ? next.detail_error : "",
  };
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
    reassociate_path: "",
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

export function applyReassociateBrowseSelection(library, selected) {
  if (selected == null || selected === "") return library;
  return {
    ...library,
    reassociate_path: String(selected),
  };
}

export function chooseReassociateSourceRequest(editRoot, storedPath) {
  return {
    command: "choose_reassociate_source",
    arguments: { editRoot, storedPath },
  };
}

function validIsoDate(value) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const parsed = new Date(`${value}T00:00:00Z`);
  return !Number.isNaN(parsed.valueOf()) && parsed.toISOString().slice(0, 10) === value;
}

function identityKind(identity) {
  if (!identity) return "unassigned";
  if (typeof identity === "string") return identity === "placeholder" ? "placeholder" : "legacy";
  if (identity.kind) return identity.kind;
  if (identity.entra || identity.object_id) return "entra";
  if (identity.placeholder) return "placeholder";
  if (identity.legacy) return "legacy";
  return "unresolved";
}

function identityObjectId(identity) {
  return identity?.object_id ?? identity?.entra?.object_id ?? identity?.person?.object_id ?? null;
}

function identityLabel(identity, placeholder = "Not configured") {
  const kind = identityKind(identity);
  if (kind === "placeholder") {
    return identity?.label ?? identity?.placeholder?.label ?? placeholder;
  }
  if (kind === "legacy") {
    return identity?.label ?? identity?.legacy?.label ?? String(identity ?? placeholder);
  }
  const person = identity?.person ?? identity?.entra ?? identity;
  if (person?.display_name) return person.display_name;
  if (person?.object_id) return `Unresolved · ${person.object_id}`;
  return placeholder;
}

export function documentControlUpdateRequest(values, detail) {
  if (!detail?.document_id) throw new Error("A selected document is required.");
  const title = String(values.get("title") ?? "").trim();
  const ownerObjectId = String(values.get("ownerObjectId") ?? "").trim();
  if (!title) throw new Error("Document title cannot be empty.");
  if (!ownerObjectId) throw new Error("Choose an eligible Microsoft Entra owner.");
  return {
    command: "update_document_control",
    arguments: {
      documentId: detail.document_id,
      title,
      documentNumber: String(values.get("documentNumber") ?? "").trim(),
      documentType: String(values.get("documentType") ?? "").trim(),
      ownerObjectId,
    },
  };
}

export function documentReviewScheduleRequest(values, detail) {
  if (!detail?.document_id) throw new Error("A selected document is required.");
  const mode = String(values.get("scheduleMode") ?? "").trim();
  const interval = String(values.get("reviewIntervalMonths") ?? "").trim();
  const exemptionReason = String(values.get("reviewExemptionReason") ?? "").trim();
  if (!["inherit", "override", "exempt"].includes(mode)) {
    throw new Error("Choose a document review schedule.");
  }
  if (mode === "override" && (!/^\d+$/.test(interval) || Number(interval) < 1 || Number(interval) > 120)) {
    throw new Error("Review interval must be a whole number from 1 to 120 months.");
  }
  if (mode === "exempt" && !exemptionReason) {
    throw new Error("A review exemption reason is required.");
  }
  return {
    command: "update_document_review_schedule",
    arguments: {
      documentId: detail.document_id,
      reviewIntervalMonths: mode === "override" ? Number(interval) : null,
      reviewExemptionReason: mode === "exempt" ? exemptionReason : null,
    },
  };
}

/** Current document schedule as the form's baseline (mode + mode-specific fields). */
export function reviewScheduleBaseline(detail) {
  const schedule = detail?.review_schedule ?? {};
  if (schedule.exemption_reason) {
    return {
      mode: "exempt",
      intervalMonths: "",
      exemptionReason: String(schedule.exemption_reason),
    };
  }
  if (schedule.interval_months != null && schedule.interval_months !== "") {
    return {
      mode: "override",
      intervalMonths: String(schedule.interval_months),
      exemptionReason: "",
    };
  }
  return { mode: "inherit", intervalMonths: "", exemptionReason: "" };
}

/** Read live form controls (including disabled fields FormData would omit). */
export function reviewScheduleFormValues(form) {
  const mode = String(form?.elements?.scheduleMode?.value ?? "").trim();
  const interval = String(form?.elements?.reviewIntervalMonths?.value ?? "").trim();
  const exemptionReason = String(form?.elements?.reviewExemptionReason?.value ?? "").trim();
  return { mode, intervalMonths: interval, exemptionReason };
}

export function reviewScheduleIsDirty(formOrValues, detail) {
  const baseline = reviewScheduleBaseline(detail);
  const current =
    formOrValues && typeof formOrValues === "object" && formOrValues.elements
      ? reviewScheduleFormValues(formOrValues)
      : {
          mode: String(formOrValues?.get?.("scheduleMode") ?? formOrValues?.mode ?? "").trim(),
          intervalMonths: String(
            formOrValues?.get?.("reviewIntervalMonths") ?? formOrValues?.intervalMonths ?? "",
          ).trim(),
          exemptionReason: String(
            formOrValues?.get?.("reviewExemptionReason") ?? formOrValues?.exemptionReason ?? "",
          ).trim(),
        };
  if (current.mode !== baseline.mode) return true;
  if (current.mode === "override" && current.intervalMonths !== baseline.intervalMonths) {
    return true;
  }
  if (current.mode === "exempt" && current.exemptionReason !== baseline.exemptionReason) {
    return true;
  }
  return false;
}

/**
 * Show interval/exemption only for the matching schedule mode and enable Update
 * only when the form differs from the document's saved schedule.
 */
export function syncReviewScheduleForm(form) {
  if (!form) return;
  const mode = String(form.elements.scheduleMode?.value ?? "inherit");
  const intervalWrap = form.querySelector('[data-review-schedule-field="interval"]');
  const exemptionWrap = form.querySelector('[data-review-schedule-field="exemption"]');
  const intervalInput = form.elements.reviewIntervalMonths;
  const exemptionInput = form.elements.reviewExemptionReason;
  const submit = form.querySelector('button[type="submit"]');

  const showInterval = mode === "override";
  const showExemption = mode === "exempt";
  if (intervalWrap) intervalWrap.hidden = !showInterval;
  if (exemptionWrap) exemptionWrap.hidden = !showExemption;
  if (intervalInput) {
    intervalInput.disabled = !showInterval;
    intervalInput.required = showInterval;
  }
  if (exemptionInput) {
    exemptionInput.disabled = !showExemption;
    exemptionInput.required = showExemption;
  }

  const baseline = {
    mode: form.dataset.baselineMode ?? "inherit",
    intervalMonths: form.dataset.baselineInterval ?? "",
    exemptionReason: form.dataset.baselineExemption ?? "",
  };
  const current = reviewScheduleFormValues(form);
  let dirty = current.mode !== baseline.mode;
  if (!dirty && current.mode === "override") {
    dirty = current.intervalMonths !== baseline.intervalMonths;
  }
  if (!dirty && current.mode === "exempt") {
    dirty = current.exemptionReason !== baseline.exemptionReason;
  }
  if (submit) submit.disabled = !dirty;
}

export function bindReviewScheduleForm(form) {
  if (!form) return;
  if (form.dataset.reviewScheduleBound === "1") {
    syncReviewScheduleForm(form);
    return;
  }
  form.dataset.reviewScheduleBound = "1";
  const onChange = () => syncReviewScheduleForm(form);
  form.addEventListener("change", onChange);
  form.addEventListener("input", onChange);
  syncReviewScheduleForm(form);
}

/** Parse current released MAJOR.MINOR from detail (string or {major,minor}). */
export function parseCurrentReleaseVersion(detail) {
  const raw = detail?.current_release?.version;
  if (raw == null || raw === "") return null;
  if (typeof raw === "object") {
    const major = Number(raw.major);
    const minor = Number(raw.minor);
    if (!Number.isInteger(major) || !Number.isInteger(minor) || major < 0 || minor < 0) return null;
    return { major, minor };
  }
  const match = String(raw).trim().match(/^(\d+)\.(\d+)$/);
  if (!match) return null;
  return { major: Number(match[1]), minor: Number(match[2]) };
}

export function formatVersionLabel(version) {
  if (!version || !Number.isInteger(version.major) || !Number.isInteger(version.minor)) return "";
  return `V${version.major}.${version.minor}`;
}

/**
 * Preview versions for Next minor / Next major per CAP-0002.
 * Never-released documents resolve both modes to V1.0 (first release).
 * Later next-minor advances the minor component by 1 (V1.0 → V1.1).
 */
export function previewTargetVersions(detail) {
  const current = parseCurrentReleaseVersion(detail);
  if (!current) {
    return {
      current: null,
      next_minor: { major: 1, minor: 0 },
      next_major: { major: 1, minor: 0 },
      first_release: true,
    };
  }
  return {
    current,
    next_minor: { major: current.major, minor: current.minor + 1 },
    next_major: { major: current.major + 1, minor: 0 },
    first_release: false,
  };
}

export function effectiveCandidateTarget(detail, targetMode, manualMajor, manualMinor) {
  const preview = previewTargetVersions(detail);
  if (targetMode === "next_minor") return preview.next_minor;
  if (targetMode === "next_major") return preview.next_major;
  if (targetMode === "manual") {
    const major = String(manualMajor ?? "").trim();
    const minor = String(manualMinor ?? "").trim();
    if (!/^\d+$/.test(major) || !/^\d+$/.test(minor)) return null;
    return { major: Number(major), minor: Number(minor) };
  }
  return null;
}

export function candidateTargetHelpText(detail, targetMode, manualMajor, manualMinor) {
  const target = effectiveCandidateTarget(detail, targetMode, manualMajor, manualMinor);
  const preview = previewTargetVersions(detail);
  if (!target) {
    if (targetMode === "manual") return "Effective target: enter Manual major and minor";
    return "Effective target: choose a target version";
  }
  const label = formatVersionLabel(target);
  if (preview.first_release && targetMode !== "manual") {
    return `Effective target: ${label} (first release · approval required)`;
  }
  if (targetMode === "next_minor") {
    return `Effective target: ${label} · stays in draft for direct PDF export`;
  }
  if (targetMode === "next_major") {
    return `Effective target: ${label} · opens approver review after notification`;
  }
  const current = preview.current;
  const approvalRequired = !current
    ? target.major === 1 && target.minor === 0
    : target.major > current.major;
  if (approvalRequired) {
    return `Effective target: ${label} · opens approver review after notification`;
  }
  return `Effective target: ${label} · stays in draft for direct PDF export`;
}

/** Enable manual fields only for Manual target; refresh effective-target label. */
export function syncCandidateTargetForm(form) {
  if (!form) return;
  const mode = String(form.elements.targetMode?.value ?? "next_minor");
  const manual = mode === "manual";
  for (const wrap of form.querySelectorAll("[data-candidate-manual-field]")) {
    wrap.hidden = !manual;
  }
  const majorInput = form.elements.manualMajor;
  const minorInput = form.elements.manualMinor;
  if (majorInput) {
    majorInput.disabled = !manual;
    majorInput.required = manual;
  }
  if (minorInput) {
    minorInput.disabled = !manual;
    minorInput.required = manual;
  }
  const label = form.querySelector("[data-candidate-effective-target]");
  if (!label) return;
  const detail = {
    current_release: form.dataset.currentReleaseVersion
      ? { version: form.dataset.currentReleaseVersion }
      : null,
  };
  label.textContent = candidateTargetHelpText(detail, mode, majorInput?.value, minorInput?.value);
}

export function bindCandidateTargetForm(form) {
  if (!form) return;
  if (form.dataset.candidateTargetBound === "1") {
    syncCandidateTargetForm(form);
    return;
  }
  form.dataset.candidateTargetBound = "1";
  const onChange = () => syncCandidateTargetForm(form);
  form.addEventListener("change", onChange);
  form.addEventListener("input", onChange);
  syncCandidateTargetForm(form);
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
    throw new Error("Begin revision is no longer available. Edit the draft to leave released.");
  }
  if (action === "submit_candidate") {
    const targetMode = String(values?.get("targetMode") ?? "").trim();
    const requesterObjectId = String(values?.get("requesterObjectId") ?? "").trim();
    const changelog = String(values?.get("changelog") ?? "").trim();
    const effectiveDate = String(values?.get("effectiveDate") ?? "").trim();
    const manualMajor = String(values?.get("manualMajor") ?? "").trim();
    const manualMinor = String(values?.get("manualMinor") ?? "").trim();
    const stagedOwnerObjectId = String(values?.get("stagedOwnerObjectId") ?? "").trim();
    const stagedEditorObjectId = String(values?.get("stagedEditorObjectId") ?? "").trim();
    if (!["next_minor", "next_major", "manual"].includes(targetMode)) {
      throw new Error("Choose a target version.");
    }
    if (!requesterObjectId) throw new Error("Choose the requesting editor.");
    if (!changelog) throw new Error("A release changelog is required.");
    if (!validIsoDate(effectiveDate)) throw new Error("Effective date must use YYYY-MM-DD.");
    if (targetMode === "manual" && (!/^\d+$/.test(manualMajor) || !/^\d+$/.test(manualMinor))) {
      throw new Error("Manual target version needs whole major and minor numbers.");
    }
    if (detail.requires_identity_handover && (!stagedOwnerObjectId || !stagedEditorObjectId)) {
      throw new Error("Choose the real Owner and Editor to apply with this release.");
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
          effectiveDate,
          requesterObjectId,
          stagedOwnerObjectId: stagedOwnerObjectId || null,
          stagedEditorObjectId: stagedEditorObjectId || null,
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

export function libraryIcon(name) {
  const paths = {
    folder: '<path d="M3 6h7l2 2h9v11H3z"/><path d="M3 6V4h7l2 2"/>',
    file: '<path d="M6 3h8l4 4v14H6z"/><path d="M14 3v5h5"/>',
    chevron_right: '<path d="m9 6 6 6-6 6"/>',
    chevron_down: '<path d="m6 9 6 6 6-6"/>',
    back: '<path d="m15 5-7 7 7 7"/>',
    forward: '<path d="m9 5 7 7-7 7"/>',
    up: '<path d="m5 14 7-7 7 7"/>',
    refresh: '<path d="M20 6v5h-5"/><path d="M18.5 16a8 8 0 1 1 .5-8l1 3"/>',
    panel_left: '<rect x="3" y="4" width="18" height="16" rx="1.5"/><path d="M9 4v16"/>',
    panel_right: '<rect x="3" y="4" width="18" height="16" rx="1.5"/><path d="M15 4v16"/>',
    panel_left_collapsed: '<rect x="3" y="4" width="18" height="16" rx="1.5"/><path d="M9 4v16"/><path d="m13 9 3 3-3 3"/>',
    panel_right_collapsed: '<rect x="3" y="4" width="18" height="16" rx="1.5"/><path d="M15 4v16"/><path d="m11 9-3 3 3 3"/>',
  };
  const path = paths[name];
  if (!path) throw new Error(`Unknown Library icon: ${name}`);
  return `<svg class="library-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">${path}</svg>`;
}

function membershipLabel(entry) {
  const membership = membershipKind(entry);
  if (membership === "in_library") return "In library";
  if (membership === "lost_source") return "Lost source";
  if (membership === "not_in_library") return "Not in library";
  if (membership === "unsupported") return "Unsupported draft";
  return "Folder";
}

function lifecycleLabel(entry) {
  if (membershipKind(entry) === "lost_source") return "Lost source";
  return entry.document?.lifecycle ?? "—";
}

function counterChipsMarkup(counters) {
  const chips = [
    ["draft_documents", "~", "draft document", "draft documents"],
    ["available_to_add", "+", "file available to add", "files available to add"],
    ["unsupported_files", "!", "unsupported file", "unsupported files"],
    ["moved_documents", "?", "(re-)moved document", "(re-)moved documents"],
  ];
  return chips
    .filter(([key]) => Number(counters?.[key]) > 0)
    .map(([key, symbol, singular, plural]) => {
      const count = Number(counters[key]);
      const label = `${count} ${count === 1 ? singular : plural}`;
      return `<span class="folder-counter ${escapeHtml(key)}" title="${escapeHtml(label)}" aria-label="${escapeHtml(label)}">${symbol}${count}</span>`;
    })
    .join("");
}

function treeMarkup(tree, currentPath, expandedFolders) {
  const expanded = new Set(expandedFolders ?? ["."]);
  const nodeMarkup = (node, level) => {
    const hasChildren = node.children.length > 0;
    const isExpanded = expanded.has(node.path);
    const current = node.path === currentPath;
    const branchState = hasChildren ? ` aria-expanded="${isExpanded}"` : "";
    const toggle = hasChildren
      ? `<button class="tree-toggle" type="button" data-library-tree-toggle="${escapeHtml(node.path)}" aria-expanded="${isExpanded}" aria-label="${isExpanded ? "Collapse" : "Expand"} ${escapeHtml(node.name)}">${libraryIcon(isExpanded ? "chevron_down" : "chevron_right")}</button>`
      : '<span class="tree-toggle-spacer" aria-hidden="true"></span>';
    const children = hasChildren
      ? `<ul class="tree-group" role="group"${isExpanded ? "" : " hidden"}>${node.children.map((child) => nodeMarkup(child, level + 1)).join("")}</ul>`
      : "";
    return `<li class="tree-item${current ? " current" : ""}" role="treeitem" aria-level="${level}" aria-selected="${current}"${current ? ' aria-current="page"' : ""}${branchState}><div class="tree-row">${toggle}<button class="tree-label" type="button" data-library-folder="${escapeHtml(node.path)}">${libraryIcon("folder")}<span>${escapeHtml(node.name)}</span>${counterChipsMarkup(node.counters)}</button></div>${children}</li>`;
  };
  return `<ul class="tree-root" role="tree">${buildFolderTree(tree).map((node) => nodeMarkup(node, 1)).join("")}</ul>`;
}

const LIBRARY_COLUMNS = [
  { key: "col-name", label: "Name", defaultWidth: 220, minWidth: 80 },
  { key: "col-title", label: "Title", defaultWidth: 180, minWidth: 80 },
  { key: "col-lib-state", label: "Library state", defaultWidth: 130, minWidth: 80 },
  { key: "col-lifecycle", label: "Lifecycle", defaultWidth: 110, minWidth: 70 },
  { key: "col-next-review", label: "Next review", defaultWidth: 120, minWidth: 80 },
  { key: "col-editor", label: "Editor", defaultWidth: 140, minWidth: 80 },
  { key: "col-approver", label: "Approver", defaultWidth: 140, minWidth: 80 },
  { key: "col-confidentiality", label: "Confidentiality", defaultWidth: 130, minWidth: 80 },
];

function libraryTableHeaders(library) {
  const widths = library.column_widths ?? {};
  return LIBRARY_COLUMNS.map((col) => {
    const width = widths[col.key] ?? col.defaultWidth;
    return `<th class="${escapeHtml(col.key)}" style="width:${width}px" data-col-resize="${escapeHtml(col.key)}" data-col-min-width="${col.minWidth}">${escapeHtml(col.label)}<span class="col-resize-grip" role="separator" aria-orientation="vertical" aria-label="Resize ${escapeHtml(col.label)} column"></span></th>`;
  }).join("");
}

export function setColumnWidth(library, colKey, newWidth) {
  const widths = { ...library.column_widths };
  widths[colKey] = newWidth;
  return { ...library, column_widths: widths };
}

function rowsMarkup(library, entries, emptyMessage) {
  const filteredEntries = filterLibraryEntries(entries, library);
  const allEntries = sortLibraryEntries(filteredEntries, library.sort);
  const page = paginateLibraryEntries(allEntries, library.page_size, library.page);
  const rows = page.entries.length === 0
    ? `<tr><td colspan="8" class="empty-table">${escapeHtml(entries.length > 0 ? "No entries match Show in folder." : emptyMessage)}</td></tr>`
    : page.entries.map((entry) => {
        const path = normalizeLibraryPath(entry.relative_path);
        const selected = library.selection.includes(path) ? " selected" : "";
        const isFile = entry.kind === "file";
        const doc = entry.document;
        const nextReview = isFile && doc?.next_review_due
            ? `<span>${escapeHtml(doc.next_review_due)}</span>`
            : "";
        const editor = isFile && doc?.editor
            ? `<span>${escapeHtml(doc.editor)}</span>`
            : "";
        const approver = isFile && doc?.approver
            ? `<span>${escapeHtml(doc.approver)}</span>`
            : "";
        const confidentiality = isFile && doc?.confidentiality
            ? `<span>${escapeHtml(doc.confidentiality)}</span>`
            : "";
        return `<tr class="library-row${selected}${membershipKind(entry) === "lost_source" ? " lost-source" : ""}" tabindex="0" data-library-entry="${escapeHtml(path)}" data-library-kind="${escapeHtml(entry.kind)}"><td><span class="entry-name">${libraryIcon(entry.kind === "folder" ? "folder" : "file")}<span>${escapeHtml(entry.name)}</span>${entry.kind === "folder" ? counterChipsMarkup(entry.folder_counters) : ""}</span></td><td>${escapeHtml(entry.document?.control?.title ?? (isFile ? "—" : ""))}</td><td>${escapeHtml(membershipLabel(entry))}</td><td>${escapeHtml(lifecycleLabel(entry))}</td><td>${nextReview}</td><td>${editor}</td><td>${approver}</td><td>${confidentiality}</td></tr>`;
      }).join("");
  const paging = page.total > library.page_size
    ? `<button class="text-button" type="button" data-library-page="previous" ${page.page === 0 ? "disabled" : ""}>Previous</button><span>Page ${page.page + 1} of ${page.page_count}</span><button class="text-button" type="button" data-library-page="next" ${page.page + 1 === page.page_count ? "disabled" : ""}>Next</button>`
    : `<span>${page.total} entries</span>`;
  return `${rows}<tr class="pagination-row"><td colspan="8"><div><label>Rows per page <select data-library-page-size><option value="10" ${library.page_size === 10 ? "selected" : ""}>10</option><option value="25" ${library.page_size === 25 ? "selected" : ""}>25</option><option value="50" ${library.page_size === 50 ? "selected" : ""}>50</option><option value="100" ${library.page_size === 100 ? "selected" : ""}>100</option></select></label><span>${paging}</span></div></td></tr>`;
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
  const external = externalLifecycleMarkup(library, detail);
  return `<div class="lifecycle-panel" aria-label="Revision cycle actions"><div class="lifecycle-actions">${external}${form("cancel_review", "Cancel review", cancel.available, cancel.reason)}${form("mark_obsolete", "Mark obsolete", obsolete.available, obsolete.reason)}</div><details class="workflow-evidence" data-library-evidence ${library.evidence_open ? "open" : ""}><summary>Canonical workflow evidence · ${escapeHtml(verification)}</summary>${events}</details></div>`;
}

function externalLifecycleMarkup(library, detail) {
  const candidate = detail.active_candidate;
  const mailConfirmation = (label) => `<label class="confirmation"><input type="checkbox" name="mailtoConfirmed" value="yes"> I confirm the host mail message for ${escapeHtml(label)} was sent.</label>`;
  const peopleOptions = (detail.eligible_people ?? [])
    .map((person) => `<option value="${escapeHtml(person.object_id)}">${escapeHtml(person.display_name)} · ${escapeHtml(person.email)}</option>`)
    .join("");
  const peopleAvailable = (detail.eligible_people ?? []).length > 0;
  const placeholderRequestingEditor = detail.eligible_people_state === "successful_empty"
    ? '<label>Requesting editor<input value="&lt;editor&gt;" readonly aria-readonly="true"></label>'
    : `<label>Requesting editor<select name="requesterObjectId" required><option value="">Choose person</option>${peopleOptions}</select></label>`;
  const handover = detail.requires_identity_handover && peopleAvailable
    ? `<fieldset><legend>Apply real identities with successful release</legend><p class="source-path">These values remain staged through review and failed export. Current document and release identity do not change until release succeeds.</p><label>Owner<select name="stagedOwnerObjectId" required><option value="">Choose person</option>${peopleOptions}</select></label><label>Editor<select name="stagedEditorObjectId" required><option value="">Choose person</option>${peopleOptions}</select></label></fieldset>`
    : "";
  const placeholderBlock = detail.requires_identity_handover && !peopleAvailable
    ? '<p class="library-detail-error" role="status">Candidate submission and release are blocked while &lt;owner&gt; and &lt;editor&gt; remain unresolved. Refresh the identity source after real people are available.</p>'
    : "";
  const failedSignIn = Boolean(library.approver_sign_in?.challenge && library.detail_error);
  const signIn = library.approver_sign_in?.challenge
    ? failedSignIn
      ? '<p class="source-path"><strong>Previous sign-in failed.</strong> Generate a new Microsoft Entra device code before continuing.</p><button class="button secondary" type="button" data-library-approver-sign-in>Sign in again</button>'
      : `<p class="source-path">Complete Microsoft sign-in with code ${escapeHtml(library.approver_sign_in.challenge.user_code)}.</p><button class="button secondary" type="button" data-open-external="${escapeHtml(library.approver_sign_in.challenge.verification_uri)}">Open sign-in page</button><button class="button secondary" type="button" data-library-approver-sign-in-complete="${escapeHtml(library.approver_sign_in.challenge.challenge_id)}">Complete approver sign-in</button>`
    : library.approver_sign_in?.actor
      ? '<p class="source-path">Approver sign-in ready. Recording a decision will verify this actor against the assigned approver.</p>'
      : '<button class="button secondary" type="button" data-library-approver-sign-in>Sign in as approver</button>';
  const preview = previewTargetVersions(detail);
  const nextMinorLabel = formatVersionLabel(preview.next_minor);
  const nextMajorLabel = formatVersionLabel(preview.next_major);
  const currentReleaseVersion = preview.current
    ? `${preview.current.major}.${preview.current.minor}`
    : "";
  const nextMinorOption = preview.first_release
    ? `Next minor · ${nextMinorLabel} (first release)`
    : `Next minor · ${nextMinorLabel} (approval optional)`;
  const nextMajorOption = preview.first_release
    ? `Next major · ${nextMajorLabel} (first release · approval required)`
    : `Next major · ${nextMajorLabel} (approval required)`;
  const effectiveHelp = candidateTargetHelpText(detail, "next_minor");
  const submit = detail.lifecycle === "draft" && !candidate
    ? `<form class="lifecycle-action" data-library-lifecycle-form="submit_candidate" data-current-release-version="${escapeHtml(currentReleaseVersion)}" data-first-release="${preview.first_release ? "1" : "0"}"><strong>Create release candidate</strong><p class="source-path">Records target version, effective date, and changelog for this draft in the workspace. It does not send a file elsewhere. Next minor stays in draft so you can export the PDF next. Next major and first release open approver review after the notification is sent.</p>${placeholderBlock}<label>Target version<select name="targetMode"><option value="next_minor" selected>${escapeHtml(nextMinorOption)}</option><option value="next_major">${escapeHtml(nextMajorOption)}</option><option value="manual">Manual target</option></select></label><p class="source-path" data-candidate-effective-target>${escapeHtml(effectiveHelp)}</p><label data-candidate-manual-field hidden>Manual major<input name="manualMajor" inputmode="numeric" disabled></label><label data-candidate-manual-field hidden>Manual minor<input name="manualMinor" inputmode="numeric" disabled></label><label>Effective date<input name="effectiveDate" type="date" required></label>${placeholderRequestingEditor}${handover}<label>Changelog<textarea name="changelog" required></textarea></label><label>Review content-check override reason (only when needed)<textarea name="reviewOverrideReason"></textarea></label><button class="button" type="submit" ${detail.requires_identity_handover && !peopleAvailable ? "disabled" : ""}>Create release candidate</button></form>`
    : "";
  const reviewRetry = candidate?.status === "review_delivery_failed"
    ? `<form class="lifecycle-action" data-library-lifecycle-form="retry_review_notification"><strong>Confirm review request delivery</strong><small>The host mail handler opened without advancing the review.</small>${mailConfirmation("the review request")}<button class="button" type="submit">Confirm review message sent</button></form>`
    : "";
  const decision = candidate?.status === "in_review"
    ? `<form class="lifecycle-action" data-library-lifecycle-form="decide_review"><strong>Record review decision</strong>${signIn}<label>Decision<select name="decision" required><option value="">Choose decision</option><option value="approved">Approve</option><option value="rejected">Reject</option><option value="changes_requested">Request changes</option></select></label><label>Comment<textarea name="comment"></textarea></label><button class="button" type="submit">Record decision</button></form>`
    : "";
  const release = candidate && ((candidate.approval_required && candidate.status === "approved") || (!candidate.approval_required && candidate.status === "draft"))
    ? `<form class="lifecycle-action" data-library-lifecycle-form="release_candidate"><strong>Export and release ${escapeHtml(`V${candidate.version.major}.${candidate.version.minor}`)}</strong><small>Uses installed Office for .docx; Markdown is assembled into a temporary DOCX from the configured workspace Word template, then uses the same installed-Word export path. The candidate release profile and any staged handover apply only after export and release succeed.</small><label>Release content-check override reason (only when needed)<textarea name="releaseOverrideReason"></textarea></label><button class="button" type="submit" ${candidate.identity_handover_blocked ? "disabled" : ""}>Export PDF and release</button></form>`
    : "";
  const decisionRetry = detail.retryable_decision_candidate
    ? `<form class="lifecycle-action" data-library-lifecycle-form="retry_decision_notification"><strong>Confirm decision notification delivery</strong>${mailConfirmation("the decision outcome")}<button class="button secondary" type="submit">Confirm decision message sent</button></form>`
    : "";
  const minorRetry = detail.retryable_minor_publication
    ? `<form class="lifecycle-action" data-library-lifecycle-form="retry_minor_publication_notification"><strong>Confirm minor-publication delivery</strong>${mailConfirmation("the minor publication")}<button class="button secondary" type="submit">Confirm publication message sent</button></form>`
    : "";
  return `${submit}${reviewRetry}${decision}${release}${decisionRetry}${minorRetry}`;
}

function selectionScroll(body, footer = "") {
  return `<div class="selection-scroll">${body}</div>${footer}`;
}

function selectionMarkup(library) {
  const selected = selectedEntries(library);
  if (selected.length === 0) {
    return selectionScroll('<div class="selection-empty"><h3>Selection</h3><p>Select a folder or file to see its identity and available actions.</p></div>');
  }
  if (selected.length > 1) {
    const allAddable = selected.every((entry) => membershipKind(entry) === "not_in_library");
    const allRegistered = selected.every((entry) => {
      const membership = membershipKind(entry);
      return membership === "in_library" || membership === "lost_source";
    });
    const identities = selected.slice(0, 5).map((entry) => `<li>${escapeHtml(entry.name)}</li>`).join("");
    return selectionScroll(`<div class="selection-header"><span class="badge">${selected.length} selected</span><button class="text-button" type="button" data-library-clear-selection>Clear</button></div><ul class="identity-list">${identities}</ul><div class="selection-actions">${allAddable ? `<button class="button" type="button" data-library-add>Add ${selected.length} documents to library</button>` : ""}${allRegistered ? `<button class="button danger" type="button" data-library-unregister>Unregister ${selected.length} documents</button>` : ""}${!allAddable && !allRegistered ? "<p>Mixed selections have no common action.</p>" : ""}</div>`);
  }
  const entry = selected[0];
  if (entry.kind === "folder") {
    return selectionScroll(`<h3>${escapeHtml(entry.name)}</h3><p class="source-path">${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><button class="button" type="button" data-library-open-selected>Open folder</button>`);
  }
  const membership = membershipKind(entry);
  if (membership === "not_in_library") {
    return selectionScroll(`<span class="badge">Not in library</span><h3>${escapeHtml(entry.name)}</h3><p class="source-path">${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><button class="button" type="button" data-library-add>Add to library</button>`);
  }
  if (membership === "unsupported") {
    return selectionScroll(`<span class="badge muted">Unsupported draft</span><h3>${escapeHtml(entry.name)}</h3><p class="source-path">${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><p>This file remains visible but cannot be registered.</p>`);
  }
  const detail = library.detail;
  if (!detail || detail.document_id !== entryDocumentId(entry)) {
    return selectionScroll(`<h3>${escapeHtml(entry.document?.control?.title ?? entry.name)}</h3><p class="source-path">${escapeHtml(entry.name)}<br>${escapeHtml(normalizeLibraryPath(entry.relative_path))}</p><p>Loading document control data…</p>`);
  }
  const confidentiality = detail.effective_confidentiality;
  const roles = detail.effective_workflow_roles;
  const role = (value, placeholder) => identityLabel(value, placeholder);
  const sourceAvailable = detail.source_exists && detail.source_state === "registered";
  const sourceLost = detail.source_state === "registered" && !detail.source_exists;
  const release = detail.current_release;
  const releaseProfile = release?.profile ?? release?.document_control_snapshot ?? null;
  const currentReleaseIdentity = release
    ? `<div class="current-release-profile"><strong>Current released PDF · V${escapeHtml(release.version)}</strong><span>${escapeHtml(release.relative_pdf_path)}</span><small>${release.pdf_exists ? "Available" : "Missing PDF"}</small><h4>Immutable current release profile</h4>${releaseProfile ? `<dl class="selection-details"><dt>Effective date</dt><dd>${escapeHtml(release.effective_date ?? releaseProfile.effective_date ?? "Unknown")}</dd><dt>Title</dt><dd>${escapeHtml(releaseProfile.title)}</dd><dt>Document number</dt><dd>${escapeHtml(releaseProfile.document_number ?? "Not set")}</dd><dt>Document type</dt><dd>${escapeHtml(releaseProfile.document_type ?? "Not set")}</dd><dt>Owner</dt><dd>${escapeHtml(identityLabel(releaseProfile.owner, "Unknown"))}</dd></dl>` : '<p class="source-path">Unknown · this legacy release has no stored profile or effective date.</p>'}</div>`
    : '<div class="current-release-profile"><strong>Current released PDF</strong><span>No active release</span></div>';
  const documentTypeOptions = (detail.document_types ?? [])
    .filter((type) => type.enabled || type.id === detail.control.document_type)
    .map((type) => `<option value="${escapeHtml(type.id)}" ${type.id === detail.control.document_type ? "selected" : ""}>${escapeHtml(type.label)}</option>`)
    .join("");
  const confidentialityOptions = (detail.confidentiality_types ?? [])
    .filter((type) => type.enabled || type.id === detail.confidentiality_override)
    .map((type) => `<option value="${escapeHtml(type.id)}" ${type.id === detail.confidentiality_override ? "selected" : ""}>${escapeHtml(type.label)}</option>`)
    .join("");
  const currentOwner = detail.current_owner ?? detail.control.owner;
  const currentOwnerId = identityObjectId(currentOwner);
  const ownerOptions = (detail.eligible_people ?? [])
    .map((person) => `<option value="${escapeHtml(person.object_id)}" ${person.object_id === currentOwnerId ? "selected" : ""}>${escapeHtml(person.display_name)} · ${escapeHtml(person.email)}</option>`)
    .join("");
  const ownerSelectable = ownerOptions.length > 0;
  const unresolvedOwnerOption = !currentOwnerId
    ? `<option value="" selected disabled>${escapeHtml(identityLabel(currentOwner, "Choose a person"))}</option>`
    : "";
  const schedule = detail.review_schedule ?? {};
  const scheduleBaseline = reviewScheduleBaseline(detail);
  const scheduleMode = scheduleBaseline.mode;
  const scheduleMarkup = sourceLost
    ? '<p class="source-path">Review schedule editing is unavailable while the source is Lost source. Reassociate the source first.</p>'
    : `<form id="library-review-schedule-form" class="confidentiality-editor" data-baseline-mode="${escapeHtml(scheduleBaseline.mode)}" data-baseline-interval="${escapeHtml(scheduleBaseline.intervalMonths)}" data-baseline-exemption="${escapeHtml(scheduleBaseline.exemptionReason)}"><p class="source-path">Next review is derived from the current release effective date. An exemption records a reason and creates no due date.</p><label>Schedule<select name="scheduleMode"><option value="inherit" ${scheduleMode === "inherit" ? "selected" : ""}>Use workspace interval (${escapeHtml(schedule.workspace_interval_months ?? "—")} months)</option><option value="override" ${scheduleMode === "override" ? "selected" : ""}>Document interval override</option><option value="exempt" ${scheduleMode === "exempt" ? "selected" : ""}>Exempt document</option></select></label><label data-review-schedule-field="interval"${scheduleMode === "override" ? "" : " hidden"}>Review interval months<input name="reviewIntervalMonths" type="number" min="1" max="120" value="${escapeHtml(schedule.interval_months ?? "")}"${scheduleMode === "override" ? " required" : " disabled"}></label><label data-review-schedule-field="exemption"${scheduleMode === "exempt" ? "" : " hidden"}>Exemption reason<textarea name="reviewExemptionReason"${scheduleMode === "exempt" ? " required" : " disabled"}>${escapeHtml(schedule.exemption_reason ?? "")}</textarea></label><p class="source-path">Next review due: ${escapeHtml(schedule.next_due_date ?? (schedule.exemption_reason ? "Exempt" : "Unknown until release date is known"))}</p><button class="button secondary" type="submit" disabled>Update review schedule</button></form>`;
  const editor = sourceLost
    ? '<p class="source-path">Document control and confidentiality changes are unavailable while the source is Lost source. Reassociate the source first.</p>'
    : `<div class="document-control-editor" aria-labelledby="document-control-editor-heading"><h4 id="document-control-editor-heading">Edit document control data</h4><p class="source-path">Applies to ${escapeHtml(detail.source_name)} · ${escapeHtml(detail.relative_path)}</p>${library.detail_error ? `<p class="library-detail-error" role="alert">${escapeHtml(library.detail_error)}</p>` : ""}${ownerSelectable ? "" : '<p class="library-detail-error" role="status">No eligible Microsoft Entra owner is available. Identity placeholders and legacy owner text are display-only.</p>'}<form id="library-document-control-form"><div class="document-control-fields"><label>Title<input name="title" required value="${escapeHtml(detail.control.title)}"></label><label>Document number<input name="documentNumber" value="${escapeHtml(detail.control.document_number ?? "")}"></label><label>Document type<select name="documentType"><option value="">Not set</option>${documentTypeOptions}</select></label><label>Owner<select name="ownerObjectId" required ${ownerSelectable ? "" : "disabled"}>${unresolvedOwnerOption}${ownerOptions}</select></label></div><button class="button" type="submit" ${ownerSelectable ? "" : "disabled"}>Save document control</button></form><form id="library-confidentiality-form" class="confidentiality-editor"><label>Confidentiality override<select name="confidentialityTypeId"><option value="">Use inherited folder policy</option>${confidentialityOptions}</select></label><button class="button secondary" type="submit">Apply confidentiality</button></form></div>`;
  const openAttr = (key) => (selectionSectionOpen(library, key) ? " open" : "");
  const section = (key, title, body, extraClass = "") =>
    `<details class="selection-section${extraClass ? ` ${extraClass}` : ""}" data-library-section="${key}"${openAttr(key)}><summary><span class="selection-section-chevron" aria-hidden="true"></span><span class="selection-section-title">${title}</span><span class="selection-section-hint" aria-hidden="true"></span></summary><div class="selection-section-body">${body}</div></details>`;
  const controlSummary = `<dl class="selection-details"><dt>Document type</dt><dd>${escapeHtml(detail.control.document_type ?? "Not set")}</dd><dt>Owner</dt><dd>${escapeHtml(identityLabel(currentOwner, "Not set"))}</dd><dt>Confidentiality</dt><dd>${escapeHtml(confidentiality?.label ?? "Not configured")}${confidentiality ? ` · ${escapeHtml(confidentiality.document_override ? "override" : `from ${confidentiality.source_folder}`)}` : ""}</dd><dt>Editor</dt><dd>${escapeHtml(role(roles?.editor, "<editor>"))}</dd><dt>Approver</dt><dd>${escapeHtml(role(roles?.approver, "Not configured"))}</dd></dl>`;
  const controlBody = `${controlSummary}${editor}`;
  const reassociatePath = library.reassociate_path || detail.relative_path;
  const reassociateError = library.detail_error
    ? `<p class="library-detail-error" role="alert">${escapeHtml(library.detail_error)}</p>`
    : "";
  const reassociateMarkup = sourceLost
    ? `${reassociateError}<p class="source-path">Choose another supported file under the edit root.</p><form id="library-reassociate-form" class="reassociate-form"><label for="library-reassociate-path">Reassociate source</label><div class="directory-field"><input id="library-reassociate-path" name="path" required value="${escapeHtml(reassociatePath)}" aria-label="New edit-root-relative source path"><button class="button secondary" type="button" data-reassociate-browse>Browse…</button></div><button class="button secondary" type="submit">Reassociate source</button></form>`
    : "";
  const actionsBody = `<div class="selection-actions"><button class="button" type="button" data-library-open-source ${sourceAvailable ? "" : "disabled"}>Open source draft</button><button class="button" type="button" data-library-open-release ${release?.pdf_exists ? "" : "disabled"}>Open current released PDF</button><button class="button" type="button" data-library-open-notes>Open notes</button><button class="button secondary" type="button" data-library-open-assistance ${sourceLost ? "disabled" : ""}>Evaluate changes with Claude</button><button class="button secondary" type="button" data-library-copy-permalink>Copy permalink</button><button class="button danger" type="button" data-library-unregister>Unregister</button></div>${reassociateMarkup}`;
  const actionsFooter = section("actions", "Actions", actionsBody, "selection-actions-footer");
  const releasesBody = currentReleaseIdentity
    || '<p class="source-path">No release evidence is recorded for this document.</p>';
  const membershipBadge = sourceLost
    ? '<span class="badge warn">Lost source</span>'
    : '<span class="badge">In library</span>';
  const lifecycleBadge = sourceLost
    ? '<span class="badge muted">Lost source</span>'
    : `<span class="badge muted">${escapeHtml(detail.lifecycle)}</span>`;
  const lostBanner = sourceLost
    ? '<p class="library-detail-error" role="status">The draft file is not at the stored path. Most actions stay disabled until you reassociate the source.</p>'
    : "";
  return selectionScroll(
    `<div class="selection-header"><div class="selection-header-badges">${membershipBadge}${lifecycleBadge}</div><button class="text-button" type="button" data-library-clear-selection>Clear</button></div><h3>${escapeHtml(detail.control.title)}</h3>${detail.control.document_number ? `<p class="document-number">${escapeHtml(detail.control.document_number)}</p>` : ""}${lostBanner}<div class="source-identity"><strong>Source file</strong><span>${escapeHtml(detail.source_name)}</span><small>${escapeHtml(detail.relative_path)}</small></div>${section("control", "Document control data", controlBody)}${section("schedule", "Document review schedule", scheduleMarkup)}${section("revision", "Revision cycle", sourceLost ? '<p class="source-path">Revision cycle actions are unavailable while the source is Lost source.</p>' : lifecyclePanelMarkup(library, detail))}${section("releases", "Releases", releasesBody)}`,
    actionsFooter,
  );
}

export function libraryMarkup(workspace, activity, library, error = "") {
  const folder = normalizeLibraryPath(library.folder?.relative_path ?? activity?.route_state?.folder);
  const breadcrumbs = breadcrumbSegments(folder, workspace.edit_root.split(/[\\/]/).filter(Boolean).at(-1) ?? "Library")
    .map((segment) => `<button type="button" data-library-folder="${escapeHtml(segment.path)}">${escapeHtml(segment.label)}</button>`)
    .join(`<span class="breadcrumb-chevron">${libraryIcon("chevron_right")}</span>`);
  const searchScope = library.entire_library ? "Entire library" : "Current folder";
  const sortOptions = `<option value="name" ${library.sort === "name" ? "selected" : ""}>Name</option><option value="title" ${library.sort === "title" ? "selected" : ""}>Title</option><option value="number" ${library.sort === "number" ? "selected" : ""}>Document number</option><option value="lifecycle" ${library.sort === "lifecycle" ? "selected" : ""}>Lifecycle</option>`;
  const entries = library.results ?? library.folder.entries ?? [];
  const visibleTotal = filterLibraryEntries(entries, library).length;
  const heading = library.results === null ? (folder === "." ? "Library root" : folder.split("/").at(-1)) : "Search results";
  const visibilityToggle = (key, label) => `<button type="button" data-library-visibility="${key}" aria-pressed="${library[key]}">${label}</button>`;
  const searchSummary = library.results === null
    ? ""
    : `<span class="search-result-summary">${escapeHtml(searchScope)} · ${visibleTotal} visible of ${library.results.length} results <button class="text-button" type="button" data-library-clear-search>Clear</button></span>`;
  const treeFolded = isLibraryPaneFolded(library, "tree");
  const detailFolded = isLibraryPaneFolded(library, "detail");
  const foldButton = (side, target) => {
    const folded = isLibraryPaneFolded(library, side);
    const label = folded ? `Expand ${target}` : `Fold ${target}`;
    const icon = side === "tree"
      ? (folded ? "panel_left_collapsed" : "panel_left")
      : (folded ? "panel_right_collapsed" : "panel_right");
    return `<button class="icon-button" type="button" data-library-fold="${side}" aria-pressed="${folded}" aria-label="${label}" title="${label}">${libraryIcon(icon)}</button>`;
  };
  const treeAside = treeFolded
    ? ""
    : `<aside class="folder-tree" aria-label="Library folders" style="width:${library.tree_width}px">${treeMarkup(library.tree, folder, library.expanded_folders)}</aside>`;
  const treeSplitter = treeFolded
    ? ""
    : `<div class="library-splitter tree-splitter" role="separator" aria-orientation="vertical" aria-label="Resize folder tree" aria-valuemin="170" aria-valuemax="420" aria-valuenow="${library.tree_width}" tabindex="0" data-tree-splitter></div>`;
  const detailSplitter = detailFolded
    ? ""
    : `<div class="library-splitter" role="separator" aria-orientation="vertical" aria-label="Resize document details" aria-valuemin="280" aria-valuemax="640" aria-valuenow="${library.detail_width}" tabindex="0" data-library-splitter></div>`;
  const detailAside = detailFolded
    ? ""
    : `<aside class="selection-pane" aria-live="polite" style="width:${library.detail_width}px">${selectionMarkup(library)}</aside>`;
  return `<section class="library-workspace">
    <div class="library-toolbar">
      <button class="icon-button" type="button" data-library-history="back" ${library.back.length ? "" : "disabled"} aria-label="Back" title="Back">${libraryIcon("back")}</button>
      <button class="icon-button" type="button" data-library-history="forward" ${library.forward.length ? "" : "disabled"} aria-label="Forward" title="Forward">${libraryIcon("forward")}</button>
      <button class="icon-button" type="button" data-library-up ${folder === "." ? "disabled" : ""} aria-label="Up" title="Up">${libraryIcon("up")}</button>
      <button class="icon-button" type="button" data-library-refresh aria-label="Refresh" title="Refresh">${libraryIcon("refresh")}</button>
      ${foldButton("tree", "folder tree")}
      ${foldButton("detail", "selection pane")}
      <nav class="breadcrumbs" aria-label="Current folder">${breadcrumbs}</nav>
      <form id="library-search-form" class="library-search">
        <input name="query" value="${escapeHtml(library.query)}" aria-label="Search library" placeholder="Search files, paths, titles, numbers">
        <label><input type="checkbox" name="entireLibrary" ${library.entire_library ? "checked" : ""}> Entire library</label>
        <button class="button secondary" type="submit">Search</button>
      </form>
      <label class="sort-control">Sort <select data-library-sort>${sortOptions}</select></label>
    </div>
    ${error ? `<p class="library-error" role="alert">${escapeHtml(error)}</p>` : ""}
    <div class="library-grid${treeFolded ? " tree-folded" : ""}${detailFolded ? " detail-folded" : ""}">
      ${treeAside}
      ${treeSplitter}
      <section class="folder-contents">
        <header><div><span class="eyebrow">${library.results === null ? "Current folder" : "Explorer search"}</span><h2>${escapeHtml(heading)}</h2></div><span>${visibleTotal} visible entries</span></header>
        ${searchSummary}
        <div class="library-visibility" role="group" aria-label="Show in folder">
          <span>Show in folder</span>
          ${visibilityToggle("show_draft_documents", "Draft documents")}
          ${visibilityToggle("show_available_to_add", "Available to add")}
          ${visibilityToggle("show_unsupported_files", "Unsupported files")}
          ${visibilityToggle("show_moved_documents", "(Re-)Moved documents")}
        </div>
        <div class="table-scroll"><table><thead><tr>${libraryTableHeaders(library)}</tr></thead><tbody>${rowsMarkup(library, entries, library.results === null ? "This folder has no visible entries." : "No files match this search.")}</tbody></table></div>
      </section>
      ${detailSplitter}
      ${detailAside}
    </div>
  </section>`;
}
