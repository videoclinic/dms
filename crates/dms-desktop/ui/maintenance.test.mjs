import assert from "node:assert/strict";
import test from "node:test";

import {
  applyReleaseSnapshot,
  createReleaseState,
  filteredReleaseRows,
  periodicReviewMarkup,
  periodicReviewRequest,
  releaseMaintenanceMarkup,
  releaseWithdrawalRequest,
  workspaceMaintenanceMarkup,
  workspaceRestoreRequest,
} from "./maintenance.mjs";

const release = (overrides = {}) => ({
  document_id: "document-1",
  document_title: "Employee <Handbook>",
  release_id: "release-1",
  version: "1.0",
  relative_pdf_path: "Policies/Handbook_V1.0_internal.pdf",
  pdf_digest: "a".repeat(64),
  confidentiality_id: "internal",
  confidentiality_label: "Internal",
  workflow_chain_head: "b".repeat(64),
  approval_chain_head: "c".repeat(64),
  released_at: "2026-08-10T10:00:00Z",
  withdrawn: false,
  orphaned: false,
  verification: "match",
  ...overrides,
});

test("release maintenance filters titles, pages rows, and escapes evidence", () => {
  const state = applyReleaseSnapshot(createReleaseState(), {
    rows: [release(), release({ release_id: "release-2", document_title: "Quality Manual", verification: "mismatch" })],
  });
  assert.equal(filteredReleaseRows({ ...state, query: "quality" }).length, 1);
  const markup = releaseMaintenanceMarkup({ ...state, page_size: 1 });
  assert.match(markup, /Employee &lt;Handbook&gt;/);
  assert.match(markup, /Verify entire publish tree/);
  assert.match(markup, /Verify this release/);
  assert.match(markup, /Page 1 of 2/);
  assert.doesNotMatch(markup, /Employee <Handbook>/);
});

test("release integrity failures are visible and verification is described as read-only", () => {
  const state = applyReleaseSnapshot(createReleaseState(), {
    rows: [release({ verification: "missing_file" })],
  });
  const markup = releaseMaintenanceMarkup(state);
  assert.match(markup, /Missing PDF/);
  assert.match(markup, /data-release-open="release-1"[^>]*disabled/);
  assert.match(markup, /never repairs, replaces, or deletes/);
});

test("release history keeps withdrawn and orphaned records explicit", () => {
  const markup = releaseMaintenanceMarkup(applyReleaseSnapshot(createReleaseState(), {
    rows: [release({ withdrawn: true, orphaned: true })],
  }));
  assert.match(markup, /Withdrawn/);
  assert.match(markup, /Orphaned source record/);
  assert.match(markup, /Open PDF/);
  assert.doesNotMatch(markup, /data-release-withdraw-form/);
});

test("release withdrawal maps reason and explicit confirmation to the narrow command", () => {
  assert.deepEqual(releaseWithdrawalRequest(new URLSearchParams({
    documentId: "document-1",
    releaseId: "release-1",
    reason: " Corrected publication ",
    confirmed: "on",
  })), {
    command: "withdraw_release",
    arguments: {
      documentId: "document-1",
      releaseId: "release-1",
      reason: "Corrected publication",
      confirmed: true,
    },
  });
});

test("periodic review shows due markers and blocks duplicate requests", () => {
  const markup = periodicReviewMarkup({
    loading: false,
    error: "",
    notice: "Reminder accepted.",
    markers: [{
      document_id: "document-1",
      title: "Policy",
      release_id: "release-1",
      version: { major: 1, minor: 2 },
      next_review_due: "2026-08-01",
      status: "overdue",
      open_review_id: "review-1",
    }],
  });
  assert.match(markup, /Overdue/);
  assert.match(markup, /Reminder accepted/);
  assert.match(markup, /Record result/);
  assert.match(markup, /Cancel review/);
  assert.match(markup, /Send reminder/);
  assert.match(markup, /name="confirmed" required/);
  assert.doesNotMatch(markup, /data-periodic-review-start/);
});

test("periodic review actions map explicit confirmation to narrow desktop commands", () => {
  const values = {
    documentId: "document-1",
    reviewId: "review-1",
    result: "changes_required",
    comment: "Responsibilities changed",
    confirmed: true,
  };

  assert.deepEqual(periodicReviewRequest("result", values), {
    command: "complete_periodic_review",
    arguments: {
      documentId: "document-1",
      reviewId: "review-1",
      result: "changes_required",
      comment: "Responsibilities changed",
      confirmed: true,
    },
  });
  assert.equal(
    periodicReviewRequest("cancel", values).command,
    "cancel_periodic_review",
  );
  assert.deepEqual(periodicReviewRequest("reminder", values), {
    command: "remind_periodic_review",
    arguments: {
      documentId: "document-1",
      reviewId: "review-1",
      confirmed: true,
    },
  });
});

test("workspace integrity renders lock owner, backup evidence, and restore outcome", () => {
  const markup = workspaceMaintenanceMarkup({
    error: "",
    notice: "Backup verified and restored.",
    lock_status: {
      state: "stale",
      stale_after_hours: 24,
      lock: {
        os_user: "operator <one>",
        hostname: "host-1",
        process_id: 42,
        acquired_at: "2026-08-10T10:00:00Z",
      },
    },
    backup_outcome: {
      archive_path: "/backup/workspace.zip",
      entry_count: 3,
      manifest_digest: "f".repeat(64),
    },
    restore_outcome: {
      workspace_id: "workspace-1",
      edit_root: "/replacement/edit",
      publish_root: "/replacement/publish",
      entry_count: 3,
      manifest_digest: "e".repeat(64),
    },
  });
  assert.match(markup, /operator &lt;one&gt;/);
  assert.match(markup, /Stale after 24 hours/);
  assert.match(markup, /Backup created/);
  assert.match(markup, /\/backup\/workspace\.zip/);
  assert.match(markup, /Workspace restored/);
  assert.match(markup, /name="confirmed" required/);
  assert.match(markup, /manifest SHA-256/);
});

test("workspace restore request carries every explicit recovery decision", () => {
  const values = new URLSearchParams({
    archivePath: " /backup/workspace.zip ",
    editRoot: " /replacement/edit ",
    publishRoot: " /replacement/publish ",
    replaceExisting: "on",
    takeOverStaleLock: "on",
    confirmed: "on",
  });
  assert.deepEqual(workspaceRestoreRequest(values), {
    archivePath: "/backup/workspace.zip",
    editRoot: "/replacement/edit",
    publishRoot: "/replacement/publish",
    replaceExisting: true,
    takeOverStaleLock: true,
    confirmed: true,
  });
});
