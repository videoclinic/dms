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

export function periodicReviewMarkup(state) {
  const rows = state.markers ?? [];
  const body = state.loading
    ? '<p class="status">Loading periodic-review markers…</p>'
    : rows.length
      ? rows.map((row) => {
          const canStart = row.release_id && row.status !== "exempt" && !row.open_review_id;
          const action = row.open_review_id
            ? '<span class="integrity ok">Review requested</span>'
            : `<button class="button secondary" type="button" data-periodic-review-start="${escapeHtml(row.document_id)}" ${canStart ? "" : "disabled"}>Request review</button>`;
          return `<article class="review-row"><div><strong>${escapeHtml(row.title)}</strong><span class="badge">${row.version ? `V${escapeHtml(`${row.version.major}.${row.version.minor}`)}` : "No release"}</span><span class="integrity ${row.status === "overdue" ? "problem" : "ok"}">${escapeHtml(reviewStatusLabel(row.status))}</span></div><div class="release-meta"><span>Next review: ${escapeHtml(row.next_review_due ?? "not scheduled")}</span></div>${action}</article>`;
        }).join("")
      : '<p class="empty-state">No released documents have periodic-review markers.</p>';
  return `<section class="card maintenance-card"><span class="badge">Audit & reports</span><h2>Periodic document review</h2><p>Review requests bind the current release ID, version, confidentiality snapshot, approver, and PDF digest. A mismatched or missing PDF blocks the request.</p><p class="status" role="alert">${escapeHtml(state.error)}</p><div class="release-list">${body}</div></section>`;
}

export function workspaceMaintenanceMarkup(state) {
  const outcome = state.outcome
    ? `<div class="success-panel"><strong>Backup created</strong><span>${escapeHtml(state.outcome.archive_path)}</span><span>${escapeHtml(state.outcome.entry_count)} files · manifest SHA-256 ${escapeHtml(state.outcome.manifest_digest)}</span></div>`
    : "";
  return `<section class="card maintenance-card"><span class="badge">Maintenance</span><h2>Full workspace backup</h2><p>The archive contains workspace metadata, every registered source draft, every recorded release PDF, and a SHA-256 manifest. Existing archives are never overwritten.</p><form id="workspace-backup-form" class="form-row"><div class="field"><label for="backup-archive-path">Archive path</label><input id="backup-archive-path" name="archivePath" required autocomplete="off" placeholder="/backups/dms-workspace.zip or C:\\Backups\\dms-workspace.zip"></div><button class="button" type="submit">Create backup</button></form><p class="status" role="alert">${escapeHtml(state.error)}</p>${outcome}</section>`;
}
