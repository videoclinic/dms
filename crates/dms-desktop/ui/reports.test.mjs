import assert from "node:assert/strict";
import test from "node:test";

import {
  applyAuditReportSnapshot,
  auditReportRequest,
  auditReportsMarkup,
  createAuditReportState,
  filteredAuditReportRows,
} from "./reports.mjs";

const report = (overrides = {}) => ({
  event_id: "event-1",
  generated_at: "2026-08-10T10:00:00Z",
  local_os_user: "raphael",
  event_hash: "e".repeat(64),
  format: "pdf",
  relative_path: ".dms/exports/Audit-2026.pdf",
  filter: {
    document_ids: ["document-1"],
    approver_object_ids: [],
    confidentiality_type_ids: ["internal"],
    from: null,
    through: null,
  },
  sha256: "a".repeat(64),
  size: 1024,
  verification: "match",
  ...overrides,
});

test("audit report requests normalize all filters and explicit output", () => {
  const request = auditReportRequest({
    format: "csv",
    output: ".dms/exports/audit.csv",
    documents: "doc-2, doc-1",
    approvers: "approver-1",
    confidentiality: "restricted, internal",
    from: "2026-01-01",
    through: "2026-01-31",
  });

  assert.deepEqual(request, {
    command: "generate_audit_report",
    arguments: {
      request: {
        format: "csv",
        relative_path: ".dms/exports/audit.csv",
        filter: {
          document_ids: ["doc-2", "doc-1"],
          approver_object_ids: ["approver-1"],
          confidentiality_type_ids: ["restricted", "internal"],
          from: "2026-01-01T00:00:00Z",
          through: "2026-01-31T23:59:59Z",
        },
      },
    },
  });
});

test("audit report filtering happens before pagination and searches report metadata", () => {
  const rows = Array.from({ length: 12 }, (_, index) => report({
    event_id: `event-${index}`,
    relative_path: `.dms/exports/${index < 6 ? "Policy" : "Quality"}-${index}.pdf`,
  }));
  const state = applyAuditReportSnapshot(createAuditReportState(), {
    rows,
    evidence_chain: "valid",
  });
  const filtered = filteredAuditReportRows({ ...state, query: "quality" });
  const markup = auditReportsMarkup({ ...state, query: "quality", page_size: 5 });

  assert.equal(filtered.length, 6);
  assert.match(markup, /Page 1 of 2 · 6 reports/);
  assert.match(markup, /Quality-6\.pdf/);
  assert.doesNotMatch(markup, /Policy-0\.pdf/);
});

test("audit report history exposes honest integrity and host actions with escaped values", () => {
  const state = applyAuditReportSnapshot(createAuditReportState(), {
    evidence_chain: { tampered_at: "event-0" },
    rows: [report({
      event_id: "event-<unsafe>",
      relative_path: ".dms/exports/<unsafe>.pdf",
      verification: "missing_file",
      local_os_user: "name<script>",
    })],
  });
  const markup = auditReportsMarkup(state);

  assert.match(markup, /Missing report/);
  assert.match(markup, /Evidence chain invalid/);
  assert.match(markup, /data-report-verify="event-&lt;unsafe&gt;"/);
  assert.match(markup, /data-report-open-folder="event-&lt;unsafe&gt;"/);
  assert.match(markup, /Source drafts and release PDF bytes are never embedded/);
  assert.doesNotMatch(markup, /<unsafe>/);
  assert.doesNotMatch(markup, /<script>/);
});
