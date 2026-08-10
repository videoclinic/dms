function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function createAuditReportState() {
  return {
    rows: [],
    query: "",
    page: 0,
    page_size: 10,
    loading: false,
    error: "",
    notice: "",
    evidence_chain: "valid",
  };
}

export function applyAuditReportSnapshot(state, snapshot, notice = "") {
  return {
    ...state,
    rows: [...(snapshot?.rows ?? [])],
    evidence_chain: snapshot?.evidence_chain ?? "missing",
    loading: false,
    error: "",
    notice,
    page: 0,
  };
}

export function filteredAuditReportRows(state) {
  const query = state.query.trim().toLocaleLowerCase();
  if (!query) return state.rows;
  return state.rows.filter((row) => [
    row.relative_path,
    row.generated_at,
    row.format,
    row.verification,
    row.local_os_user,
    JSON.stringify(row.filter ?? {}),
  ].some((value) => String(value ?? "").toLocaleLowerCase().includes(query)));
}

function valueOf(values, name) {
  return typeof values.get === "function" ? values.get(name) : values[name];
}

function listValue(values, name) {
  return String(valueOf(values, name) ?? "")
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}

function boundary(value, endOfDay) {
  const text = String(value ?? "").trim();
  if (!text) return null;
  return `${text}T${endOfDay ? "23:59:59" : "00:00:00"}Z`;
}

export function auditReportRequest(values) {
  const format = String(valueOf(values, "format") ?? "pdf");
  const output = String(valueOf(values, "output") ?? "").trim();
  return {
    command: "generate_audit_report",
    arguments: {
      request: {
        format,
        relative_path: output || null,
        filter: {
          document_ids: listValue(values, "documents"),
          approver_object_ids: listValue(values, "approvers"),
          confidentiality_type_ids: listValue(values, "confidentiality"),
          from: boundary(valueOf(values, "from"), false),
          through: boundary(valueOf(values, "through"), true),
        },
      },
    },
  };
}

function verificationLabel(status) {
  return {
    match: "Verified",
    mismatch: "Checksum mismatch",
    missing_file: "Missing report",
    invalid_evidence: "Invalid evidence",
  }[status] ?? "Not checked";
}

function chainLabel(status) {
  if (status === "valid") return "Evidence chain valid";
  if (status === "missing") return "No report evidence yet";
  return "Evidence chain invalid";
}

function filterSummary(filter = {}) {
  const parts = [];
  if (filter.document_ids?.length) parts.push(`${filter.document_ids.length} document(s)`);
  if (filter.approver_object_ids?.length) parts.push(`${filter.approver_object_ids.length} approver(s)`);
  if (filter.confidentiality_type_ids?.length) {
    parts.push(`confidentiality: ${filter.confidentiality_type_ids.join(", ")}`);
  }
  if (filter.from || filter.through) parts.push(`${filter.from ?? "start"} – ${filter.through ?? "now"}`);
  return parts.join(" · ") || "Entire workspace";
}

function reportRowMarkup(row) {
  const integrity = row.verification === "match" ? "ok" : "problem";
  return `<article class="report-row" data-report-event="${escapeHtml(row.event_id)}">
    <div class="release-main"><strong>${escapeHtml(row.relative_path)}</strong><span class="badge">${escapeHtml(String(row.format).toUpperCase())}</span><span class="integrity ${integrity}">${escapeHtml(verificationLabel(row.verification))}</span></div>
    <div class="release-meta"><span>${escapeHtml(new Date(row.generated_at).toLocaleString())}</span><span>${escapeHtml(filterSummary(row.filter))}</span><span>OS user: ${escapeHtml(row.local_os_user)}</span></div>
    <div class="release-evidence"><span title="${escapeHtml(row.sha256)}">SHA-256 ${escapeHtml(row.sha256.slice(0, 12))}…</span><span>${escapeHtml(row.size)} bytes</span></div>
    <div class="release-actions"><button class="button secondary" type="button" data-report-verify="${escapeHtml(row.event_id)}">Verify</button><button class="button secondary" type="button" data-report-open-folder="${escapeHtml(row.event_id)}">Open folder</button></div>
  </article>`;
}

export function auditReportsMarkup(state) {
  const rows = filteredAuditReportRows(state);
  const pageCount = Math.max(1, Math.ceil(rows.length / state.page_size));
  const page = Math.min(state.page, pageCount - 1);
  const visible = rows.slice(page * state.page_size, (page + 1) * state.page_size);
  const body = state.loading
    ? '<p class="status">Loading audit reports…</p>'
    : visible.length
      ? visible.map(reportRowMarkup).join("")
      : '<p class="empty-state">No generated reports match this filter.</p>';
  const chainProblem = state.evidence_chain === "valid" || state.evidence_chain === "missing" ? "ok" : "problem";
  return `<section class="card maintenance-card audit-report-card">
    <div class="section-heading"><div><span class="badge">Audit export</span><h2>Generate audit report</h2></div><span class="integrity ${chainProblem}">${escapeHtml(chainLabel(state.evidence_chain))}</span></div>
    <p>Reports contain control metadata, workflow evidence, release records, and verification verdicts. Source drafts and release PDF bytes are never embedded.</p>
    <form id="audit-report-generate-form" class="audit-report-form">
      <div class="field"><label for="audit-report-format">Format</label><select id="audit-report-format" name="format"><option value="pdf">PDF</option><option value="csv">CSV</option></select></div>
      <div class="field"><label for="audit-report-output">Output file <small>(optional)</small></label><input id="audit-report-output" name="output" autocomplete="off" placeholder=".dms/exports/audit-report.pdf"></div>
      <div class="field"><label for="audit-report-documents">Document UUIDs <small>(comma-separated)</small></label><input id="audit-report-documents" name="documents" autocomplete="off"></div>
      <div class="field"><label for="audit-report-approvers">Approver UUIDs <small>(comma-separated)</small></label><input id="audit-report-approvers" name="approvers" autocomplete="off"></div>
      <div class="field"><label for="audit-report-confidentiality">Confidentiality type IDs <small>(comma-separated)</small></label><input id="audit-report-confidentiality" name="confidentiality" autocomplete="off"></div>
      <div class="field"><label for="audit-report-from">From</label><input id="audit-report-from" name="from" type="date"></div>
      <div class="field"><label for="audit-report-through">Through</label><input id="audit-report-through" name="through" type="date"></div>
      <button class="button" type="submit">Generate report</button>
    </form>
    <p class="status" role="alert">${escapeHtml(state.error)}</p>${state.notice ? `<p class="success-panel">${escapeHtml(state.notice)}</p>` : ""}
    <div class="section-heading"><div><span class="badge">Recent reports</span><h2>Report history and integrity</h2></div></div>
    <form id="audit-report-filter-form" class="form-row"><div class="field"><label for="audit-report-filter">Filter by name, generated time, format, confidentiality, user, or state</label><input id="audit-report-filter" name="query" value="${escapeHtml(state.query)}" autocomplete="off"></div><button class="button secondary" type="submit">Apply filter</button></form>
    <div class="release-list">${body}</div>
    <div class="pagination"><button type="button" class="button secondary" data-report-page="previous" ${page === 0 ? "disabled" : ""}>Previous</button><span>Page ${page + 1} of ${pageCount} · ${rows.length} reports</span><label>Rows <select data-report-page-size>${[10, 25, 50, 100].map((size) => `<option value="${size}" ${size === state.page_size ? "selected" : ""}>${size}</option>`).join("")}</select></label><button type="button" class="button secondary" data-report-page="next" ${page + 1 >= pageCount ? "disabled" : ""}>Next</button></div>
  </section>`;
}
