function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function createReleaseState() {
  return {
    rows: [],
    query: "",
    page: 0,
    page_size: 20,
    loading: false,
    error: "",
  };
}

export function applyReleaseSnapshot(state, snapshot) {
  return {
    ...state,
    rows: [...(snapshot?.rows ?? [])],
    loading: false,
    error: "",
  };
}

export function filteredReleaseRows(state) {
  const query = state.query.trim().toLocaleLowerCase();
  return query
    ? state.rows.filter((row) => row.document_title.toLocaleLowerCase().includes(query))
    : state.rows;
}

function statusLabel(status) {
  return {
    match: "Verified",
    mismatch: "Checksum mismatch",
    missing_file: "Missing PDF",
  }[status] ?? "Not checked";
}

function releaseRowMarkup(row) {
  const integrity = row.verification === "match" ? "ok" : "problem";
  const approval = row.approval_chain_head
    ? `<span title="${escapeHtml(row.approval_chain_head)}">approval ${escapeHtml(row.approval_chain_head.slice(0, 12))}…</span>`
    : "<span>minor release · no approval event</span>";
  return `<article class="release-row" data-release-id="${escapeHtml(row.release_id)}">
    <div class="release-main"><strong>${escapeHtml(row.document_title)}</strong><span class="badge">V${escapeHtml(row.version)}</span><span class="integrity ${integrity}">${escapeHtml(statusLabel(row.verification))}</span></div>
    <div class="release-meta"><span>${escapeHtml(row.relative_pdf_path)}</span><span>${escapeHtml(row.confidentiality_label)}</span><span>${escapeHtml(new Date(row.released_at).toLocaleString())}</span></div>
    <div class="release-evidence"><span title="${escapeHtml(row.pdf_digest)}">PDF ${escapeHtml(row.pdf_digest.slice(0, 12))}…</span><span title="${escapeHtml(row.workflow_chain_head)}">workflow ${escapeHtml(row.workflow_chain_head.slice(0, 12))}…</span>${approval}</div>
    <div class="release-actions"><button class="button secondary" type="button" data-release-verify="${escapeHtml(row.release_id)}" data-document-id="${escapeHtml(row.document_id)}">Verify this release</button></div>
  </article>`;
}

export function releaseMaintenanceMarkup(state) {
  const rows = filteredReleaseRows(state);
  const pageCount = Math.max(1, Math.ceil(rows.length / state.page_size));
  const page = Math.min(state.page, pageCount - 1);
  const visible = rows.slice(page * state.page_size, (page + 1) * state.page_size);
  const body = state.loading
    ? '<p class="status">Loading releases…</p>'
    : visible.length
      ? visible.map(releaseRowMarkup).join("")
      : '<p class="empty-state">No releases match this title filter.</p>';
  return `<section class="card maintenance-card">
    <div class="section-heading"><div><span class="badge">Publish tree</span><h2>Release history and integrity</h2></div><button class="button" type="button" data-release-verify-all>Verify entire publish tree</button></div>
    <p>Verification reads the recorded PDF and compares its SHA-256 digest. It never repairs, replaces, or deletes release bytes.</p>
    <form id="release-filter-form" class="form-row"><div class="field"><label for="release-title-filter">Filter by document title</label><input id="release-title-filter" name="query" value="${escapeHtml(state.query)}" autocomplete="off"></div><button class="button secondary" type="submit">Apply filter</button></form>
    <p class="status" role="alert">${escapeHtml(state.error)}</p>
    <div class="release-list">${body}</div>
    <div class="pagination"><button type="button" class="button secondary" data-release-page="previous" ${page === 0 ? "disabled" : ""}>Previous</button><span>Page ${page + 1} of ${pageCount} · ${rows.length} releases</span><label>Rows <select data-release-page-size>${[10, 20, 50].map((size) => `<option value="${size}" ${size === state.page_size ? "selected" : ""}>${size}</option>`).join("")}</select></label><button type="button" class="button secondary" data-release-page="next" ${page + 1 >= pageCount ? "disabled" : ""}>Next</button></div>
  </section>`;
}

function reviewStatusLabel(status) {
  return {
    current: "Current",
    due_soon: "Due soon",
    overdue: "Overdue",
    exempt: "Exempt",
  }[status] ?? status;
}

function reviewActionValue(values, name) {
  return typeof values.get === "function" ? values.get(name) : values[name];
}

export function periodicReviewRequest(action, values) {
  const documentId = String(reviewActionValue(values, "documentId") ?? "");
  const reviewId = String(reviewActionValue(values, "reviewId") ?? "");
  const confirmation = reviewActionValue(values, "confirmed");
  const confirmed = confirmation === true || confirmation === "on";
  const common = { documentId, reviewId };
  if (action === "result") {
    return {
      command: "complete_periodic_review",
      arguments: {
        ...common,
        result: String(reviewActionValue(values, "result") ?? ""),
        comment: String(reviewActionValue(values, "comment") ?? "").trim(),
        confirmed,
      },
    };
  }
  if (action === "cancel") {
    return {
      command: "cancel_periodic_review",
      arguments: {
        ...common,
        comment: String(reviewActionValue(values, "comment") ?? "").trim(),
        confirmed,
      },
    };
  }
  if (action === "reminder") {
    return {
      command: "remind_periodic_review",
      arguments: { ...common, confirmed },
    };
  }
  throw new Error(`Unknown periodic-review action: ${action}`);
}

function openReviewActions(row) {
  const documentId = escapeHtml(row.document_id);
  const reviewId = escapeHtml(row.open_review_id);
  return `<div class="periodic-review-actions">
    <form data-periodic-review-form class="periodic-review-form">
      <input type="hidden" name="documentId" value="${documentId}"><input type="hidden" name="reviewId" value="${reviewId}">
      <div class="field"><label>Result <select name="result"><option value="confirmed_current">Confirmed current</option><option value="changes_required">Changes required</option><option value="obsolete">Obsolete</option></select></label></div>
      <div class="field"><label>Required comment <textarea name="comment" required rows="2"></textarea></label></div>
      <label class="confirm-field"><input type="checkbox" name="confirmed" required> Confirm this periodic-review action.</label>
      <div class="release-actions"><button class="button" type="submit" name="action" value="result">Record result</button><button class="button secondary danger-text" type="submit" name="action" value="cancel">Cancel review</button></div>
    </form>
    <form data-periodic-review-form class="periodic-review-reminder-form">
      <input type="hidden" name="documentId" value="${documentId}"><input type="hidden" name="reviewId" value="${reviewId}">
      <label class="confirm-field"><input type="checkbox" name="confirmed" required> Confirm sending a reminder to the snapshotted approver.</label>
      <button class="button secondary" type="submit" name="action" value="reminder">Send reminder</button>
    </form>
  </div>`;
}

export function periodicReviewMarkup(state) {
  const rows = state.markers ?? [];
  const body = state.loading
    ? '<p class="status">Loading periodic-review markers…</p>'
    : rows.length
      ? rows.map((row) => {
          const canStart = row.release_id && row.status !== "exempt" && !row.open_review_id;
          const action = row.open_review_id
            ? openReviewActions(row)
            : `<button class="button secondary" type="button" data-periodic-review-start="${escapeHtml(row.document_id)}" ${canStart ? "" : "disabled"}>Request review</button>`;
          return `<article class="review-row"><div><strong>${escapeHtml(row.title)}</strong><span class="badge">${row.version ? `V${escapeHtml(`${row.version.major}.${row.version.minor}`)}` : "No release"}</span><span class="integrity ${row.status === "overdue" ? "problem" : "ok"}">${escapeHtml(reviewStatusLabel(row.status))}</span></div><div class="release-meta"><span>Next review: ${escapeHtml(row.next_review_due ?? "not scheduled")}</span></div>${action}</article>`;
        }).join("")
      : '<p class="empty-state">No released documents have periodic-review markers.</p>';
  return `<section class="card maintenance-card"><span class="badge">Audit & reports</span><h2>Periodic document review</h2><p>Review requests bind the current release ID, version, confidentiality snapshot, approver, and PDF digest. A mismatched or missing PDF blocks the request.</p><p class="status" role="alert">${escapeHtml(state.error)}</p>${state.notice ? `<p class="success-panel">${escapeHtml(state.notice)}</p>` : ""}<div class="release-list">${body}</div></section>`;
}

export function workspaceMaintenanceMarkup(state) {
  const backupOutcome = state.backup_outcome
    ? `<div class="success-panel"><strong>Backup created</strong><span>${escapeHtml(state.backup_outcome.archive_path)}</span><span>${escapeHtml(state.backup_outcome.entry_count)} files · manifest SHA-256 ${escapeHtml(state.backup_outcome.manifest_digest)}</span></div>`
    : "";
  const restoreOutcome = state.restore_outcome
    ? `<div class="success-panel"><strong>Workspace restored</strong><span>${escapeHtml(state.restore_outcome.workspace_id)}</span><span>edit: ${escapeHtml(state.restore_outcome.edit_root)}</span><span>publish: ${escapeHtml(state.restore_outcome.publish_root)}</span><span>${escapeHtml(state.restore_outcome.entry_count)} verified files · manifest SHA-256 ${escapeHtml(state.restore_outcome.manifest_digest)}</span></div>`
    : "";
  const lock = state.lock_status;
  const lockOwner = lock?.lock
    ? `${escapeHtml(lock.lock.os_user)} @ ${escapeHtml(lock.lock.hostname)} · PID ${escapeHtml(lock.lock.process_id)} · ${escapeHtml(lock.lock.acquired_at)}`
    : "No owner";
  const lockStatus = lock
    ? `<div class="integrity ${lock.state === "stale" ? "problem" : "ok"}"><strong>${escapeHtml(lock.state)}</strong><span>${lockOwner}</span><span>Stale after ${escapeHtml(lock.stale_after_hours)} hours</span></div>`
    : '<p class="status">Lock status has not been loaded.</p>';
  return `<section class="card maintenance-card"><span class="badge">Maintenance</span><h2>Workspace integrity and recovery</h2><p>Advisory locks expose concurrent-open risk. Backups contain workspace metadata, registered source drafts, release PDFs, and a SHA-256 manifest; lock files are excluded.</p><p class="status" role="alert">${escapeHtml(state.error)}</p>${state.notice ? `<p class="success-panel">${escapeHtml(state.notice)}</p>` : ""}<section class="maintenance-section"><h3>Advisory workspace lock</h3>${lockStatus}<form id="workspace-lock-status-form"><button class="button secondary" type="submit">Refresh status</button></form><form id="workspace-lock-config-form" class="form-row"><div class="field"><label for="lock-stale-hours">Stale after hours</label><input id="lock-stale-hours" name="hours" required min="1" type="number" value="${escapeHtml(lock?.stale_after_hours ?? 24)}"></div><label class="confirm-field"><input type="checkbox" name="confirmed" required> Confirm this workspace setting.</label><button class="button secondary" type="submit">Save threshold</button></form></section><section class="maintenance-section"><h3>Full workspace backup</h3><form id="workspace-backup-form" class="form-row"><div class="field"><label for="backup-archive-path">Archive path</label><input id="backup-archive-path" name="archivePath" required autocomplete="off" placeholder="/backups/dms-workspace.zip or C:\\Backups\\dms-workspace.zip"></div><button class="button" type="submit">Create backup</button></form>${backupOutcome}</section><section class="maintenance-section"><h3>Restore verified backup</h3><p>The selected roots must already exist. Files are written only after the manifest, paths, file types, sizes, digests, and workspace identity pass verification.</p><form id="workspace-restore-form" class="setup-form"><div class="field"><label for="restore-archive-path">Backup archive</label><input id="restore-archive-path" name="archivePath" required autocomplete="off"></div><div class="field"><label for="restore-edit-root">Replacement edit root</label><div class="directory-field"><input id="restore-edit-root" name="editRoot" required autocomplete="off"><button class="button secondary" type="button" data-directory-target="restore-edit-root">Browse…</button></div></div><div class="field"><label for="restore-publish-root">Replacement publish root</label><div class="directory-field"><input id="restore-publish-root" name="publishRoot" required autocomplete="off"><button class="button secondary" type="button" data-directory-target="restore-publish-root">Browse…</button></div></div><label class="confirm-field"><input type="checkbox" name="replaceExisting"> Replace manifest-listed files that already exist.</label><label class="confirm-field"><input type="checkbox" name="takeOverStaleLock"> Remove a stale destination lock. Current locks always block restore.</label><label class="confirm-field"><input type="checkbox" name="confirmed" required> Confirm restore into these exact roots.</label><button class="button danger" type="submit">Verify and restore</button></form>${restoreOutcome}</section></section>`;
}

export function workspaceRestoreRequest(values) {
  return {
    archivePath: String(values.get("archivePath") ?? "").trim(),
    editRoot: String(values.get("editRoot") ?? "").trim(),
    publishRoot: String(values.get("publishRoot") ?? "").trim(),
    replaceExisting: values.get("replaceExisting") === "on",
    takeOverStaleLock: values.get("takeOverStaleLock") === "on",
    confirmed: values.get("confirmed") === "on",
  };
}
