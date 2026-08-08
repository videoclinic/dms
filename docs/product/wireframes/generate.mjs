#!/usr/bin/env node
/**
 * Generates static HTML wireframes for every CAP using the visual system of
 * ../../../../rb/shadcn-admin-2.2.0 (shadcn/ui 4 + Tailwind tokens, sidebar shell).
 * Self-contained (Tailwind CDN) so pages open without a build step.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.join(__dirname, "html");
const EXPORTS = path.join(__dirname, "exports");

const NAV = [
  { id: "library", label: "Library", icon: "📚" },
  { id: "releases", label: "Releases", icon: "📦" },
  { id: "audit", label: "Audit & Reports", icon: "📋" },
  { id: "maintenance", label: "Maintenance", icon: "🧰" },
  { id: "config", label: "Configuration", icon: "⚙️" },
];

/** @type {Array<{id:string,file:string,title:string,nav:string,subtitle:string,actions?:string[],bookmarked?:boolean,body:string}>} */
const CAPS = [
  {
    id: "CAP-0001",
    file: "CAP-0001-local-folder-dms",
    title: "Workspace configuration",
    nav: "config",
    subtitle: "Dual roots, workspace ID, and .dms metadata under the edit root.",
    actions: ["Choose edit root", "Choose publish root", "Reveal .dms"],
    body: `
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Workspace roots</h3>
          <p class="muted">Stored in <code>&lt;edit-root&gt;/.dms/</code>. Edit and publish roots may coincide only if deliberately set equal.</p>
          ${kv([
            ["Workspace ID", "ws-9c3b7d1a (assigned at init)"],
            ["Edit root", "/dms/edit"],
            ["Publish root", "/dms/publish"],
            ["Storage folder", "<edit-root>/.dms/ (hidden)"],
          ])}
          <p class="hint">Closing and reopening restores roots, library, document control data, and process state from .dms.</p>
        </section>
        <section class="card">
          <h3 class="card-title">Catalogue snapshot in .dms</h3>
          <ul class="list">
            <li>Confidentiality catalogue</li>
            <li>Document-type catalogue</li>
            <li>Workflow-person roster (no secrets)</li>
            <li>Relative folder policies</li>
            <li>Document control data</li>
            <li>Release records &amp; checksums</li>
          </ul>
          <div class="callout warn">No SMTP password is stored here — relay credentials live in the OS credential store.</div>
        </section>
      </div>`,
  },
  {
    id: "CAP-0002",
    file: "CAP-0002-document-lifecycle",
    title: "Document lifecycle",
    nav: "library",
    subtitle: "Draft → in_review → approved → released. Change class + approval evidence required.",
    actions: ["Begin revision", "Submit for review"],
    body: `
      <section class="card">
        <div class="row between">
          <div class="row gap-2">
            <h2 class="doc-title">HR Data Privacy Policy</h2>
            ${badge("in_review", "info")}
            <span class=\"muted\">V1.3 released</span>
          </div>
        </div>
        ${kv([
          ["Document ID", "doc-77a12bce"],
          ["Relative draft path", "policies/HR/Handbook.docx"],
          ["Effective editor / approver", "Lukas Roth / Anna Berg"],
          ["Effective confidentiality", "Internal (inherited from /policies/HR)"],
        ])}
      </section>
      <section class="card">
        <h3 class="card-title">Lifecycle pipeline</h3>
        <div class="pipeline">
          ${["draft", "in_review", "approved", "released", "obsolete"]
            .map((s, i) => `<div class="step ${i < 2 ? "active" : ""}"><span class="dot"></span>${s}</div>`)
            .join("")}
        </div>
      </section>
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Submit for review (required fields)</h3>
          ${kv([
            ["Change summary *", "<span class=\"muted\">(required, non-empty)</span>"],
            ["Change class", "<span class=\"muted\">(cosmetic/minor or substantive/major; required after first release)</span>"],
            ["Rationale", "<span class=\"muted\">(required with change class)</span>"],
            ["Draft SHA-256", "<span class=\"muted\">(computed from current draft bytes on submit)</span>"],
            ["Requester", "Lukas Roth <lukas@vc.de> <span class=\"muted\">(snapshotted on submit)</span>"],
            ["Approver (derived)", "Anna Berg <anna@vc.de>"],
            ["Transport", "SMTP relay (password from OS credential store)"],
          ])}
          <p class="hint">Document enters <code>in_review</code> only after the transport step succeeds. Empty or missing required fields fail closed.</p>
        </section>
        <section class="card">
          <h3 class="card-title">Workflow chain (canonical event types)</h3>
          <ul class="timeline">
            <li><strong>review_requested</strong> — Lukas Roth — 2025-08-01 09:14 UTC</li>
            <li><strong>review_decision_approved</strong> — Anna Berg — 09:42 UTC</li>
            <li><strong>release</strong> (V1.3) — 09:44 UTC</li>
            <li><strong>revision_begun</strong> — 2025-08-02 11:02 UTC</li>
          </ul>
          <p class="hint">Chain head 5b3a…ffe2 — verify recomputes from canonical body (CAP-0011).</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0003",
    file: "CAP-0003-document-notes",
    title: "Document notes",
    nav: "library",
    subtitle: "Free-text notes by stable document ID. Survive renames and lifecycle transitions.",
    actions: ["Add note"],
    body: `
      <section class="card">
        <h3 class="card-title">Notes — HR Data Privacy Policy <span class="muted">(doc-77a12bce)</span></h3>
        <div class="stack">
          ${note("2025-08-02 11:08 — Lukas Roth", "Confirmed retention table updated to 24 months; double-check audit log entry.")}
          ${note("2025-07-29 16:22 — Anna Berg", "Need legal review wording on §4 before next release.")}
          ${note("2025-07-21 09:05 — Lukas Roth", "Renamed draft locally — locator updated, ID preserved.")}
        </div>
        <div class="composer">
          <label class="label">New note</label>
          <div class="textarea">Plain text — line breaks preserved. UTF-8.</div>
          <div class="row between">
            <span class="muted">Author: Lukas Roth (OS user)</span>
            <button class="btn">Save note</button>
          </div>
        </div>
        <p class="hint">Deleting a note never deletes the document or workflow evidence comments (CAP-0011).</p>
      </section>`,
  },
  {
    id: "CAP-0004",
    file: "CAP-0004-release-integrity",
    title: "Verify integrity",
    nav: "releases",
    subtitle: "SHA-256 over each released PDF vs release record. Never silent on mismatch.",
    actions: ["Verify all releases", "Export report"],
    body: `
      <section class="card">
        <h3 class="card-title">Released versions — SHA-256 verification</h3>
        ${table(
          ["Version", "Relative path", "Released", "SHA-256", "Result", "Action"],
          [
            ["V2.0", "policies/HR/Handbook_V2.0_restricted.pdf", "2025-08-02 09:44", "9f2c…b1e0", badge("match", "ok"), "Reveal"],
            ["V1.7", "policies/HR/Handbook_V1.7_restricted.pdf", "2025-07-12 12:01", "73b1…4cd2", badge("match", "ok"), "Reveal"],
            ["V1.6", "policies/HR/Handbook_V1.6_restricted.pdf", "2025-06-05 14:30", "2a91…77ee", badge("mismatch", "danger"), "Reveal"],
            ["V1.5", "policies/HR/Handbook_V1.5_restricted.pdf", "2025-05-09 10:12", "—", badge("missing file", "warn"), "Reveal"],
          ]
        )}
        <p class="hint">Per-version outcomes; verification never rewrites PDF bytes.</p>
      </section>`,
  },
  {
    id: "CAP-0005",
    file: "CAP-0005-desktop-shell",
    title: "Desktop shell overview",
    nav: "library",
    activity: "shell",
    bookmarked: true,
    subtitle: "Tauri 2 shell on Windows and macOS. Foldable left menu, session-only open activities, and explicit per-user saved views.",
    body: `
      <section class="card">
        <h3 class="card-title">Chrome contract</h3>
        <div class="stack">
          ${layer("Foldable left menu", "Primary destinations, Saved views, and Open panes. Expanded/collapsed preference persists per OS user (not in .dms).")}
          ${layer("Hamburger when folded", "Header control re-opens the menu as temporary expand/overlay; pin expanded to keep it open.")}
          ${layer("Open activity panes/tabs", "Automatic, session-only quicklinks. Labels state task + target: Audit · HR Data Privacy Policy · DOC-014 for a document or Library · policies/HR for a folder. Opening the same task + document focuses its existing pane; × closes that activity only.")}
          ${layer("Saved views", "Use ☆ Bookmark this view in the header. ★ Bookmarked is an explicit, per-user shortcut restored after relaunch; it is not a .dms workflow record.")}
          ${layer("Permalink handler", "OS-registered dms:// URI resolves workspace + document IDs (CAP-0020); opens/focuses matching activity tab.")}
        </div>
      </section>
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Expanded left menu</h3>
          <div class="mini-shell">
            <div class="mini-side">
              <div class="mini-brand">DMS Desktop <span class="fold-btn" title="Collapse">«</span></div>
              <div class="mini-nav">
                <div class="on">📚 Library</div>
                <div>📦 Releases</div>
                <div>📋 Audit</div>
                <div>🧰 Maintenance</div>
                <div>⚙️ Config</div>
              </div>
              <div class="mini-sec">Saved views</div>
              <div class="mini-tabs">
                <div>★ Library · policies/HR <span>−</span></div>
                <div>★ Shell chrome <span>−</span></div>
              </div>
              <div class="mini-sec">Open panes</div>
              <div class="mini-tabs">
                <div class="on">Library · policies/HR <span>×</span></div>
                <div>Audit · HR Data Privacy Policy · DOC-014 <span>×</span></div>
                <div>Review · HR Data Privacy Policy · DOC-014 <span>×</span></div>
              </div>
              <div class="mini-foot">ws-9c3b7d1a<br/>edit: /dms/edit<br/>publish: /dms/publish</div>
            </div>
            <div class="mini-main"><div class="mini-header">Library · policies/HR <span class="mini-bookmark">☆ Bookmark this view</span></div><p class="muted" style="padding:0.75rem;margin:0;font-size:0.75rem">The selected Open pane and this header identify the current surface.</p></div>
          </div>
        </section>
        <section class="card">
          <h3 class="card-title">Collapsed + hamburger</h3>
          <div class="mini-shell collapsed">
            <div class="mini-rail">
              <div class="ham">☰</div>
              <div class="on" title="Library">📚</div>
              <div title="Releases">📦</div>
              <div title="Audit">📋</div>
              <div title="Maintenance">🧰</div>
              <div title="Config">⚙️</div>
            </div>
            <div class="mini-main">
              <div class="mini-header"><span class="ham">☰</span> Audit · HR Data Privacy Policy · DOC-014</div>
              <p class="muted" style="padding:0.75rem;margin:0;font-size:0.8rem">Hamburger expands the left menu. Icon rail still switches primary destinations. Saved views and Open panes appear when expanded; Bookmark this view stays in the header.</p>
            </div>
          </div>
        </section>
      </div>
      <section class="card">
        <h3 class="card-title">Backend command surface</h3>
        <div class="tags">
          ${["Configure roots", "Open workspace", "Library add/remove", "Lifecycle transitions", "Release + verify", "Copy/resolve permalink", "Audit export", "Backup/restore", "Claude handoff"]
            .map((t) => `<span class="tag">${t}</span>`)
            .join("")}
        </div>
      </section>`,
  },
  {
    id: "CAP-0006",
    file: "CAP-0006-library-explorer",
    title: "Folder-first library explorer",
    nav: "library",
    subtitle: "Persistent folder tree + Explorer-like path controls + exact source file names. DMS-managed document data stays visibly separate in the list and selection pane.",
    actions: [],
    body: `
      <style>
        .app[data-cap="CAP-0006"] .list-card th,
        .app[data-cap="CAP-0006"] .list-card td { padding: 0.55rem 0.45rem; font-size: 0.75rem; }
      </style>
      <section class="card" style="padding:0.75rem 0.9rem">
        <div class="row gap-2">
          <button class="icon-btn" title="Back">←</button>
          <button class="icon-btn" title="Forward">→</button>
          <button class="icon-btn" title="Up one folder">↑</button>
          <button class="icon-btn" title="Refresh current folder (F5)">↻</button>
          <div class="row" style="height:2rem;min-width:0;flex:1;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.75rem;font-size:0.82rem;gap:0.45rem">
            <span class="muted">DMS Workspace</span><span>›</span><span>policies</span><span>›</span><strong>HR</strong>
          </div>
          <div style="height:2rem;width:16rem;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0.42rem 0.7rem;font-size:0.78rem;color:var(--muted-foreground)">Search HR and subfolders</div>
        </div>
        <p class="hint" style="margin-top:0.45rem">Back / Forward / Up and clickable breadcrumbs stay synchronized with the tree and current-folder contents.</p>
      </section>
      <div class="grid-explorer-detail" style="grid-template-columns:17.5rem minmax(0,1fr) 18.5rem;align-items:stretch">
        <aside class="card tree">
          <div class="row between mb">
            <h3 class="card-title" style="margin:0">Folders</h3>
            <span class="muted" style="font-size:0.72rem">resize ↔</span>
          </div>
          <p class="hint" style="margin-top:0">Edit-root folders · <code>.dms</code> hidden</p>
          <ul class="tree-root">
            <li>
              <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">📂 DMS Workspace</span></div>
              <ul>
                <li>
                  <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">📂 policies</span></div>
                  <ul>
                    <li>
                      <div class="tree-node active"><span class="tree-twisty">▾</span><span class="tree-label">📂 HR</span></div>
                      <ul>
                        <li><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 Recruiting</span></div></li>
                        <li><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 Templates</span></div></li>
                      </ul>
                    </li>
                    <li>
                      <div class="tree-node"><span class="tree-twisty">▸</span><span class="tree-label">📁 IT</span></div>
                    </li>
                  </ul>
                </li>
                <li>
                  <div class="tree-node"><span class="tree-twisty">▸</span><span class="tree-label">📁 procedures</span></div>
                </li>
                <li>
                  <div class="tree-node"><span class="tree-twisty">▸</span><span class="tree-label">📁 records</span></div>
                </li>
                <li>
                  <div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 Archive (empty)</span></div>
                </li>
              </ul>
            </li>
          </ul>
          <p class="hint">The folder pane stays visible by default, including empty folders and folders without registered documents.</p>
        </aside>
        <section class="card list-card">
          <div class="row between mb">
            <div>
              <h3 class="card-title" style="margin:0">HR</h3>
              <span class="muted" style="font-size:0.74rem">2 folders · 6 files · 3 in library</span>
            </div>
            <button class="btn outline">Details view</button>
          </div>
          <div class="row gap-2 mb" style="flex-wrap:wrap">
            ${badge("3 in library", "info")}
            ${badge("2 not in library", "warn")}
            <span class="muted grow" style="text-align:right">All immediate children · Search: HR + descendants</span>
          </div>
          <div class="table-wrap"><table>
            <thead><tr>
              <th></th><th>Name</th><th>Library</th><th>Document</th><th>State</th><th>Released</th>
            </tr></thead>
            <tbody>
              <tr>
                <td></td><td><strong>📁 Recruiting</strong></td><td>—</td><td>Folder</td><td>—</td><td>—</td>
              </tr>
              <tr>
                <td></td><td><strong>📁 Templates</strong></td><td>—</td><td>Folder</td><td>—</td><td>—</td>
              </tr>
              <tr>
                <td><span class="check">☐</span></td>
                <td>Handbook.docx</td>
                <td>${badge("In library", "ok")}</td>
                <td>HR Data Privacy Policy · DOC-014</td>
                <td>${badge("in_review", "info")}</td>
                <td>V1.3 ${badge("newer", "warn")}</td>
              </tr>
              <tr>
                <td><span class="check">☐</span></td>
                <td>Code_of_conduct.docx</td>
                <td>${badge("In library", "ok")}</td>
                <td>Code of Conduct · DOC-018</td>
                <td>${badge("draft", "warn")}</td>
                <td>V2.0</td>
              </tr>
              <tr>
                <td><span class="check">☐</span></td>
                <td>Leave_policy.docx</td>
                <td>${badge("In library", "ok")}</td>
                <td>Leave Policy · DOC-025</td>
                <td>${badge("released", "ok")}</td>
                <td>V1.1</td>
              </tr>
              <tr class="selected">
                <td><span class="check on">☑</span></td>
                <td>Risk assessment.md</td>
                <td>${badge("Not in library", "warn")}</td>
                <td>Markdown draft</td>
                <td>—</td>
                <td>—</td>
              </tr>
              <tr class="selected">
                <td><span class="check on">☑</span></td>
                <td>Employee onboarding.docx</td>
                <td>${badge("Not in library", "warn")}</td>
                <td>Office draft</td>
                <td>—</td>
                <td>—</td>
              </tr>
              <tr>
                <td><span class="check">☐</span></td>
                <td>HR checklist.pdf</td>
                <td>${badge("Not a draft", "muted")}</td>
                <td>PDF file</td>
                <td>—</td>
                <td>—</td>
              </tr>
            </tbody>
          </table></div>
          <p class="hint"><strong>Name is the source file:</strong> it always shows the exact filesystem name, including the extension. Registered files show the independent DMS title and number under Document.</p>
        </section>
        <aside class="card detail-pane">
          <div class="row between mb">
            <h3 class="card-title" style="margin:0">2 selected</h3>
            <button class="btn outline">Clear</button>
          </div>
          <details class="selection-section" open>
            <summary>Files</summary>
            <div class="selection-section-body"><ul class="list"><li>Risk assessment.md <span class="muted">· policies/HR</span></li><li>Employee onboarding.docx <span class="muted">· policies/HR</span></li></ul></div>
          </details>
          <details class="selection-section" open>
            <summary>Actions <span>1 available</span></summary>
            <div class="selection-section-body stack-btns"><button class="btn">Add 2 documents to library</button></div>
          </details>
          <p class="hint">Batch add is available because every selected row is an in-root supported source draft, including Markdown. A mixed or unsupported selection has no incompatible action. Registered documents instead show Source file identity, Document control data, and lifecycle actions here.</p>
        </aside>
      </div>
      ${batchSelectionPane()}`,
  },
  {
    id: "CAP-0007",
    file: "CAP-0007-draft-pdf-export",
    title: "Source draft → PDF export",
    nav: "releases",
    subtitle: "Office exports through host Office; Markdown renders locally through native WebView PDF APIs. Classified filename → temp PDF → validate → SHA-256 → atomic rename.",
    body: `
      <section class="card">
        <h3 class="card-title">Export pipeline</h3>
        <div class="pipeline">
          ${["Identify source format", "Office or local Markdown render", "Export to temp PDF", "Validate header", "SHA-256 digest", "Atomic rename"]
            .map((s) => `<div class="step active"><span class="dot"></span>${s}</div>`)
            .join("")}
        </div>
      </section>
      <section class="card">
        <h3 class="card-title">Fail-closed conditions</h3>
        <ul class="list danger-list">
          <li>Office missing or unlicensed for an Office draft → abort, no partial version</li>
          <li>Markdown render failure → abort, no partial version</li>
          <li>Unsupported draft extension → abort with clear message</li>
          <li>Temp empty or missing %PDF header → remove temp, no release record</li>
          <li>Target path occupied by non-app file → fail (ADR-0007)</li>
        </ul>
      </section>
      <section class="card">
        <h3 class="card-title">Atomic release transaction (CAP-0007 outcome 3)</h3>
        ${kv([
          ["Final filename", "Handbook_V2.0_restricted.pdf"],
          ["Classification snapshot", "Restricted (type ID: restricted)"],
        ])}
        <p class="muted">A successful release record only exists when: export produced a valid, non-empty PDF, its SHA-256 was computed, and the atomic rename to the versioned path succeeded. Failure at any step removes the temp file when possible and never commits a release record.</p>
      </section>`,
  },
  {
    id: "CAP-0008",
    file: "CAP-0008-confidentiality-classification",
    title: "Confidentiality policies",
    nav: "config",
    subtitle: "Select a folder, save its direct policy, or remove a non-root policy to inherit again.",
    body: `
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Workspace catalogue</h3>
          ${typeCard("Public", "marketing, general communications", "ok", "public")}
          ${typeCard("Internal", "default for most operations", "info", "internal")}
          ${typeCard("Restricted", "HR, finance, security", "warn", "restricted")}
          ${typeCard("Confidential", "legal, board, M&A", "danger", "confidential")}
        </section>
        <section class="card">
          <h3 class="card-title">Folder policy editor</h3>
          <div class="type-card mb">
            ${kv([
              ["Selected folder", "policies/HR/"],
              ["Direct policy", "Restricted"],
              ["After removal", "Internal from edit-root"],
            ])}
            <div class="row gap-2" style="margin-top:0.75rem"><button class="btn">Save folder policy</button><button class="btn danger">Remove folder policy</button></div>
            <p class="hint">Save creates or replaces a policy at this folder only. It does not change ancestor or child policies.</p>
          </div>
          <div class="callout warn"><strong>Remove only removes this non-root policy.</strong> The folder remains. Its documents and descendants inherit from the nearest remaining ancestor unless they have a nearer folder policy or document override. The root policy can be changed but not removed.</div>
          <h3 class="card-title" style="margin-top:1rem">Direct folder policies (edit-root relative)</h3>
          ${table(
            ["Path", "Type", "Status"],
            [
              ["edit-root", "Internal", "required root policy"],
              ["policies/HR/", "Restricted", "direct policy"],
              ["records/", "Confidential", "direct policy"],
            ]
          )}
          <p class="hint">All other folders inherit the nearest direct policy. Snapshots written into review requests and release records do not change.</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0009",
    file: "CAP-0009-release-editor",
    title: "Open draft in host editor",
    nav: "library",
    subtitle: "Host-registered Office handler or default text editor. No embed, no auto-save, file handle released immediately.",
    actions: ["Open draft"],
    body: `
      <section class="card">
        <h3 class="card-title">Registered handlers</h3>
        ${table(
          ["Format", "Handler", "Version", "Default", "Status"],
          [
            [".docx", "Microsoft Word", "16.92 (Microsoft 365)", "Yes", badge("registered", "ok")],
            [".xlsx", "Microsoft Excel", "16.92 (Microsoft 365)", "Yes", badge("registered", "ok")],
            [".pptx", "Microsoft PowerPoint", "16.92 (Microsoft 365)", "Yes", badge("registered", "ok")],
            [".md", "Default text editor", "OS registered", "Yes", badge("registered", "ok")],
          ]
        )}
      </section>
      <div class="callout warn">
        <strong>Missing handler:</strong> surfaces a clear message naming the missing handler and install location. No silent fallback to a different editor.
      </div>`,
  },
  {
    id: "CAP-0010",
    file: "CAP-0010-notification-transport",
    title: "Notification transport",
    nav: "config",
    subtitle: "SMTP relay (OS credential store) or mailto: with operator-confirmed send.",
    body: `
      <div class="grid-2">
        <section class="card">
          <div class="row gap-2 mb"><h3 class="card-title">SMTP relay</h3>${badge("active", "ok")}</div>
          ${kv([
            ["Host", "smtp.videoclinic.de"],
            ["Port", "587 (STARTTLS)"],
            ["Username", "dms@videoclinic.de"],
            ["Password", "•••••••• (OS credential store)"],
            ["From", "dms@videoclinic.de"],
            ["Recipient (snapshot)", "anna@videoclinic.de"],
          ])}
          <p class="hint">Permalink: <code>dms://open?workspace=ws-9c3b7d1a&amp;document=doc-77a12bce&amp;target=review&amp;review=r-21</code> — IDs only; survives rename and version bump.</p>
        </section>
        <section class="card">
          <div class="row gap-2 mb"><h3 class="card-title">mailto: fallback</h3>${badge("available", "warn")}</div>
          ${kv([
            ["Default handler", "Microsoft Outlook (Windows)"],
            ["Recipient", "anna@videoclinic.de"],
            ["Subject", "[DMS] Review HR Data Privacy Policy"],
            ["Body", "Relative path, action, confidentiality, CAP-0020 permalink. No document content."],
          ])}
          <p class="hint">State does not advance to <code>in_review</code> until operator confirms send. Delivery failure never reverses a decision.</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0011",
    file: "CAP-0011-approval-evidence",
    title: "Workflow chain & evidence",
    nav: "audit",
    activity: "audit-doc-77a12bce",
    subtitle: "Canonical event body, hash chain, required change + decision comments.",
    actions: ["Verify workflow", "Export chain"],
    body: `
      <section class="card">
        <h3 class="card-title">Chain — HR Data Privacy Policy</h3>
        <div class="stack">
          ${event("review_requested", "2025-08-01 09:14 UTC", "Lukas Roth", "Replaced retention table with 24-month rule.", "5b3a…ffe1", "—")}
          ${event("review_decision_approved", "2025-08-01 09:42 UTC", "Anna Berg", "Formatting only; obligations unchanged.", "5b3a…ffe2", "5b3a…ffe1")}
          ${event("release", "2025-08-01 09:44 UTC", "—", "Substantive change: 24-month retention obligation.", "5b3a…ffe3", "5b3a…ffe2")}
          ${event("revision_begun", "2025-08-02 11:02 UTC", "Lukas Roth", "Starting next change cycle.", "5b3a…ffe4", "5b3a…ffe3")}
        </div>
        <div class="callout ok">${badge("chain valid", "ok")} Verify workflow recomputed each event hash from its canonical body.</div>
      </section>`,
  },
  {
    id: "CAP-0012",
    file: "CAP-0012-audit-export",
    title: "Audit export",
    nav: "audit",
    subtitle: "Operator-triggered PDF/CSV reports. Generating a report is itself a workflow event.",
    actions: ["Generate PDF", "Generate CSV"],
    body: `
      <section class="card">
        <h3 class="card-title">Filters</h3>
        ${kv([
          ["Date range", "2025-07-01 → 2025-08-05"],
          ["Approver", "Anna Berg"],
          ["Confidentiality", "Restricted, Confidential"],
          ["Documents", "All"],
        ])}
      </section>
      <section class="card">
        <h3 class="card-title">Recent reports</h3>
        ${table(
          ["Report", "Generated", "Filter", "SHA-256", "Verify", "Size"],
          [
            ["Audit-2025-08.pdf", "2025-08-05 08:30 UTC", "All", "f1a0…d223", badge("valid", "ok"), "412 KB"],
            ["Audit-2025-07.pdf", "2025-08-01 08:15 UTC", "Confidential only", "b73e…9c44", badge("valid", "ok"), "1.2 MB"],
            ["Audit-2025-07.csv", "2025-08-01 08:15 UTC", "All", "1199…aa01", badge("valid", "ok"), "84 KB"],
            ["Audit-2025-06.pdf", "2025-07-01 08:00 UTC", "Approver: Anna Berg", "—", badge("missing file", "warn"), "—"],
          ]
        )}
        <p class="hint">Reports never embed draft or PDF bytes — metadata, digests, and the event chain only.</p>
      </section>`,
  },
  {
    id: "CAP-0013",
    file: "CAP-0013-library-maintenance",
    title: "Library maintenance",
    nav: "maintenance",
    subtitle: "Rename/move with preserved ID, missing handling, rescan for recovery or batch work, roster & catalogues, withdraw.",
    body: `
      <div class="grid-explorer">
        <aside class="card">
          <h3 class="card-title">Actions</h3>
          <div class="stack-btns">
            <button class="btn">Rename / move draft (in-root)</button>
            <button class="btn outline">Mark missing</button>
            <button class="btn outline">Rescan library</button>
            <button class="btn outline">Approver roster</button>
            <button class="btn outline">Confidentiality catalogue</button>
            <button class="btn outline">Document-type catalogue</button>
            <button class="btn danger">Withdraw release</button>
            <button class="btn danger">Reject draft in review</button>
          </div>
        </aside>
        <section class="card">
          <h3 class="card-title">Drafts requiring attention</h3>
          ${table(
            ["Document", "Old path", "Status", "Suggestion"],
            [
              ["Acceptable Use", "policies/IT/AUP.docx", badge("renamed", "warn"), "Match: policies/IT/AUP-v2.docx"],
              ["Backup Config", "policies/IT/Backup.docx", badge("missing", "danger"), "No candidate — restore from backup"],
              ["Vendor Onboarding", "procedures/Onboarding.docx", badge("candidate", "info"), "Match by last digest"],
              ["Office lock ignored", "~$AUP.docx", badge("ignored", "muted"), "Lock/temp sidecar — never a candidate"],
            ]
          )}
        </section>
      </div>`,
  },
  {
    id: "CAP-0014",
    file: "CAP-0014-workspace-integrity",
    title: "Workspace integrity",
    nav: "maintenance",
    subtitle: "Advisory lock, atomic writes, backup/restore, schema migration, corruption detection.",
    actions: ["Backup workspace", "Verify lock", "Restore from backup"],
    body: `
      <div class="grid-2">
        <div class="stack">
          <section class="card">
            <h3 class="card-title"><code>&lt;edit-root&gt;/.dms/lock</code></h3>
            ${kv([
              ["OS user", "raphael"],
              ["Hostname", "vc-host-04"],
              ["PID", "18432"],
              ["Acquired", "2025-08-05 09:11:42 UTC"],
              ["Stale threshold", "24 h (configurable)"],
            ])}
            <p class="hint">Advisory only — never blocks read access by other tools.</p>
          </section>
          <section class="card">
            <h3 class="card-title">Atomic metadata write</h3>
            <p>Every authoritative .dms file is written to a sibling temporary file and atomically replaced. Crash before rename leaves the previous valid file intact.</p>
          </section>
        </div>
        <section class="card">
          <h3 class="card-title">Backup archives</h3>
          <p class="muted mb">.dms + controlled drafts + release PDFs + manifest (paths, sizes, SHA-256). Restore verifies the manifest first.</p>
          ${table(
            ["Backup", "Created", "Files", "Manifest SHA-256"],
            [
              ["dms-backup-2025-08-05.zip", "2025-08-05 08:00 UTC", "1,284", "9f2c…b1e0"],
              ["dms-backup-2025-07-29.zip", "2025-07-29 08:00 UTC", "1,279", "3a91…77ee"],
              ["dms-backup-2025-07-22.zip", "2025-07-22 08:00 UTC", "1,272", "5b3a…ffe2"],
            ]
          )}
          <p class="hint">Retention is operator-managed. No automatic expiry or off-machine upload.</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0015",
    file: "CAP-0015-document-control-data",
    title: "Document control data",
    nav: "library",
    activity: "document-home-doc-77a12bce",
    subtitle: "Source file facts come from the filesystem. Document control data is managed by DMS Desktop under .dms, never synchronized from Office properties. All sections are expanded here for review.",
    actions: [],
    body: `
      <div class="grid-explorer-detail">
        <aside class="card tree">
          <h3 class="card-title">Folders</h3>
          <ul class="tree-root">
            <li>
              <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">📁 policies</span></div>
              <ul>
                <li>
                  <div class="tree-node active"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 HR</span></div>
                </li>
                <li>
                  <div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 IT</span></div>
                </li>
              </ul>
            </li>
            <li>
              <div class="tree-node"><span class="tree-twisty">▸</span><span class="tree-label">📁 procedures</span></div>
            </li>
          </ul>
          <p class="hint">Selected: policies / HR</p>
        </aside>
        <section class="card list-card">
          <div class="row gap-2 mb">
            ${badge("1 selected", "info")}
            <span class="muted grow">List truncated — focus is the selection pane</span>
          </div>
          <div class="table-wrap"><table>
            <thead><tr><th></th><th>Name</th><th>Document</th><th>State</th><th>Released</th></tr></thead>
            <tbody>
              <tr class="selected">
                <td><span class="check on">☑</span></td>
                <td>Handbook.docx</td>
                <td>HR Data Privacy Policy · DOC-014</td>
                <td>${badge("in_review", "info")}</td>
                <td>V1.3 ${badge("draft newer", "warn")}</td>
              </tr>
              <tr>
                <td><span class="check">☐</span></td>
                <td>AUP.docx</td>
                <td>Acceptable Use · DOC-002</td>
                <td>${badge("draft", "warn")}</td>
                <td>V2.0</td>
              </tr>
            </tbody>
          </table></div>
        </section>
        ${documentMasterDataSelectionPane()}
      </div>`,
  },
  {
    id: "CAP-0016",
    file: "CAP-0016-publish-tree-maintenance",
    title: "Publish tree maintenance",
    nav: "releases",
    subtitle: "List known releases, verify-all, reveal in host file manager, archive orphans.",
    actions: ["Verify all releases", "Reveal publish folder", "Archive orphans"],
    body: `
      <section class="card">
        <h3 class="card-title">Publish-tree</h3>
        ${table(
          ["Doc", "Version", "Publish path", "Released", "SHA-256", "State", "Verify"],
          [
            ["HR Data Privacy Policy", "V2.0", "policies/HR/Handbook_V2.0_restricted.pdf", "2025-08-01 09:44", "9f2c…b1e0", badge("current", "ok"), badge("match", "ok")],
            ["Acceptable Use", "V2.0", "policies/IT/AUP_V2.0_internal.pdf", "2025-07-29 14:12", "3a91…77ee", badge("current", "ok"), badge("match", "ok")],
            ["Incident Response", "V3.1", "procedures/IRP_V3.1_restricted.pdf", "2025-06-30 11:20", "1199…aa01", badge("current", "ok"), badge("match", "ok")],
            ["Vendor Onboarding", "V1.0", "procedures/Onboarding_V1.0_internal.pdf", "2024-11-04 09:00", "—", badge("orphaned", "warn"), badge("missing file", "danger")],
            ["Backup Config", "V1.4", "policies/IT/Backup_V1.4_internal.pdf", "2025-05-12 16:45", "5b3a…ffe2", badge("withdrawn", "muted"), badge("match", "ok")],
          ]
        )}
        <p class="hint">Release records are immutable. Correction = withdraw + Begin revision + new approval + new version.</p>
      </section>`,
  },
  {
    id: "CAP-0017",
    file: "CAP-0017-periodic-document-review",
    title: "Periodic review",
    nav: "audit",
    subtitle: "Default interval, per-document exemption, outcomes: confirmed / changes / obsolete. Reuses the document's effective approver.",
    actions: ["Start periodic review", "Send reminder"],
    body: `
      <div class="grid-2">
        <div class="stack">
          <section class="card">
            <h3 class="card-title">Workspace defaults</h3>
            ${kv([
              ["Default interval", "12 months"],
              ["Reminder window", "30 days before due"],
              ["Calendar months", "clamped to last valid day"],
            ])}
          </section>
          <section class="card">
            <h3 class="card-title">Routing</h3>
            ${kv([
              ["Reviewer", "Effective approver (CAP-0019)"],
              ["Per-document override", "—"],
            ])}
            <p class="hint">Periodic review reuses the document's effective approver. Exemption requires a reason comment in the workflow chain.</p>
          </section>
        </div>
        <section class="card">
          <h3 class="card-title">Due &amp; overdue</h3>
          ${table(
            ["Document", "Current release", "Next due", "Status", "Action"],
            [
              ["Acceptable Use", "V2.0", "2025-07-15", badge("overdue", "danger"), "Start review / Remind"],
              ["Backup Config", "V1.4", "2025-08-22", badge("due ≤30d", "warn"), "Start review / Remind"],
              ["Incident Response", "V3.1", "2025-08-30", badge("due ≤30d", "warn"), "Start review / Remind"],
              ["Vendor Onboarding", "V1.0", "2025-09-14", badge("due", "muted"), "Start review / Remind"],
              ["Code of Conduct", "V1.2", "2025-12-01", badge("ok", "ok"), "Start review / Remind"],
            ]
          )}
        </section>
      </div>`,
  },
  {
    id: "CAP-0018",
    file: "CAP-0018-claude-desktop-change-assistance",
    title: "Claude Desktop handoff",
    nav: "library",
    subtitle: "Optional, operator-mediated. Deterministic local text diff first; never calls Claude directly.",
    actions: ["Evaluate changes with Claude"],
    body: `
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Payload preview (local-only)</h3>
          <p class="muted mb">Source-draft + PDF digests identify the revision. Binary files, .dms, approver data, external paths, and credentials are excluded.</p>
          <div class="diff ok"><span class="ctx">§3.2 Retention</span>+ Retention for personal data shall not exceed 24 months.</div>
          <div class="diff warn"><span class="ctx">§3.2 Retention</span>− Retention for personal data shall not exceed 36 months.</div>
          <div class="diff info"><span class="ctx">§5.1 Approver</span>+ Effective approver: Anna Berg &lt;anna@vc.de&gt;.</div>
        </section>
        <section class="card">
          <h3 class="card-title">Operator handoff</h3>
          <div class="callout warn mb">Claude Desktop is a local client, but model processing may send the displayed payload to Anthropic. Confirm before handoff. Cancellation sends nothing.</div>
          <p><strong>Suggested class:</strong> cosmetic / minor (formatting only).</p>
          <p><strong>Suggested summary:</strong> Replaced retention table with 24-month rule; clarified §5.1 approver role.</p>
          <p class="hint">AI output is untrusted. Operator edits before acceptance. Workflow records <code>assistance_used: true</code>, provider <code>Claude Desktop</code>.</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0019",
    file: "CAP-0019-inherited-workflow-role-routing",
    title: "Workflow roles",
    nav: "config",
    subtitle: "Editor and approver inherited from nearest configured ancestor; document override beats folder.",
    body: `
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Workflow roster</h3>
          <div class="stack">
            ${person("wp-001", "Lukas Roth", "lukas@vc.de", "ok")}
            ${person("wp-002", "Anna Berg", "anna@vc.de", "ok")}
            ${person("wp-003", "Mira Klein", "mira@vc.de", "warn")}
          </div>
        </section>
        <section class="card">
          <h3 class="card-title">Folder policies (effective)</h3>
          ${table(
            ["Path", "Editor", "Approver", "Source"],
            [
              ["edit-root", "Lukas Roth", "Anna Berg", "root policy"],
              ["policies/", "Lukas Roth", "Anna Berg", "inherited"],
              ["policies/HR/", "Lukas Roth", "Anna Berg", "inherited"],
              ["Handbook.docx", "Lukas Roth (override)", "Anna Berg (override)", "document override"],
              ["policies/IT/", "Mira Klein", "Anna Berg", "folder override"],
              ["procedures/", "Lukas Roth", "Anna Berg", "inherited"],
            ]
          )}
        </section>
      </div>`,
  },
  {
    id: "CAP-0020",
    file: "CAP-0020-document-permalinks",
    title: "Document permalinks",
    nav: "library",
    activity: "document-home-doc-77a12bce",
    subtitle: "Stable local-app URI: workspace ID + document ID. Survives rename and version bumps. Never keys off path or VMAJOR.MINOR.",
    actions: ["Copy permalink", "Open from URI"],
    body: `
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Canonical URI</h3>
          ${kv([
            ["Document home", "<code>dms://open?workspace=ws-9c3b7d1a&amp;document=doc-77a12bce</code>"],
            ["Review target", "<code>…&amp;target=review&amp;review=r-21</code>"],
            ["Notes target", "<code>…&amp;target=notes</code>"],
            ["Identity keys", "workspace ID + document ID only"],
            ["Not in URI", "file name, relative path, V1.3, PDF stem, absolute paths"],
          ])}
          <div class="callout ok" style="margin-top:0.75rem">${badge("stable", "ok")} Same URI after rename Handbook.docx → Privacy-Policy.docx and after release V1.3 → V2.0.</div>
        </section>
        <section class="card">
          <h3 class="card-title">Resolve path</h3>
          <ol class="list">
            <li>OS hands URI to registered DMS handler</li>
            <li>Match registered accessible workspace by ID</li>
            <li>Look up document by stable ID in <code>.dms</code></li>
            <li>Open library navigator, select document, show selection pane</li>
            <li>Focus or create open-activity tab (CAP-0005)</li>
          </ol>
          <p class="hint">Unknown workspace/document → fail closed message. URI never records approve/reject.</p>
        </section>
      </div>
      <section class="card">
        <h3 class="card-title">Copy from selection</h3>
        ${kv([
          ["Title (display only)", "HR Data Privacy Policy"],
          ["Current locator", "policies/HR/Privacy-Policy.docx"],
          ["Latest release", "V2.0"],
          ["Clipboard", "dms://open?workspace=ws-9c3b7d1a&document=doc-77a12bce"],
        ])}
        <p class="hint">Display labels may change; clipboard URI does not include them as keys.</p>
      </section>`,
  },
];

function badge(text, kind = "muted") {
  return `<span class="badge ${kind}">${text}</span>`;
}
function kv(rows) {
  return `<dl class="kv">${rows
    .map(([k, v]) => `<div><dt>${k}</dt><dd>${v}</dd></div>`)
    .join("")}</dl>`;
}
function table(headers, rows) {
  return `<div class="table-wrap"><table>
    <thead><tr>${headers.map((h) => `<th>${h}</th>`).join("")}</tr></thead>
    <tbody>${rows
      .map((r) => `<tr>${r.map((c) => `<td>${c}</td>`).join("")}</tr>`)
      .join("")}</tbody>
  </table></div>`;
}
function note(head, body) {
  return `<article class="note"><header>${head}</header><p>${body}</p></article>`;
}
function layer(name, desc) {
  return `<div class="layer"><strong>${name}</strong><p class="muted">${desc}</p></div>`;
}
function typeCard(name, desc, kind, typeId) {
  return `<div class="type-card"><div class="row gap-2">${badge(name, kind)}</div><p class="muted">${desc}</p><p class="hint">Type ID: <code>${typeId}</code> · stable filename token · In-use — cannot be deleted.</p></div>`;
}
function event(type, ts, who, cmt, hash, pred) {
  return `<article class="event">
    <div class="row gap-2">${badge(type, "info")}<span class="muted">${ts}</span><span class="muted">by ${who}</span></div>
    <p>${cmt}</p>
    <p class="hint mono">hash ${hash} ← pred ${pred}</p>
  </article>`;
}
function ver(v, date, cls, state, kind) {
  return `<div class="ver"><div class="row gap-2"><strong>${v}</strong>${badge(state, kind)}</div><p class="muted">${date}</p><p>${cls}</p></div>`;
}
function documentMasterDataSelectionPane() {
  // CAP-0015 owns this shared selection-pane content; CAP-0006 owns its placement.
  return `<aside class="card detail-pane">
    <div class="row between mb">
      <h3 class="card-title" style="margin:0">HR Data Privacy Policy</h3>
      ${badge("in_review", "info")}
    </div>
    <p class="muted">Document number: <span class="mono">DOC-014</span></p>
    <div style="border:1px solid var(--border);border-radius:calc(var(--radius) - 2px);padding:0.65rem 0.75rem;background:var(--muted)">
      <div class="label">Source file <span style="font-weight:400">· from filesystem</span></div>
      <div class="mono" style="margin-top:0.3rem">Handbook.docx</div>
      <div class="muted" style="font-size:0.75rem;margin-top:0.2rem">Folder: policies/HR</div>
    </div>
    <details class="selection-section" open>
      <summary>Document control data <span>Managed in DMS Desktop</span></summary>
      <div class="selection-section-body">${kv([
        ["Title", "HR Data Privacy Policy"],
        ["Document number", "DOC-014 (unique in workspace)"],
        ["Document type", "policy"],
        ["Owner", "Lukas Roth"],
        ["Effective date", "2025-08-01"],
        ["Next review due", "2026-08-01"],
        ["Released", "V1.3 (current)"],
        ["Draft", badge("newer than last release", "warn")],
        ["Effective editor", "Lukas Roth"],
        ["Effective approver", "Anna Berg"],
        ["Confidentiality", "Internal (inherited)"],
      ])}<p class="hint">Stored in workspace metadata under <code>.dms</code>. Not read from or synchronized with Office document properties. Renaming the source file does not change these values.</p></div>
    </details>
    <details class="selection-section" open>
      <summary>Actions <span>15 available</span></summary>
      <div class="selection-section-body stack-btns">
        <button class="btn outline">Open draft</button>
        <button class="btn outline">Open latest released PDF</button>
        <button class="btn outline">Edit document control data</button>
        <button class="btn outline">Submit for review</button>
        <button class="btn outline">Begin revision</button>
        <button class="btn outline">Cancel review</button>
        <button class="btn danger">Mark obsolete</button>
        <button class="btn outline">Notes</button>
        <button class="btn outline">Workflow chain</button>
        <button class="btn outline">Verify integrity</button>
        <button class="btn outline">Periodic review</button>
        <button class="btn outline">Rename / reassociate</button>
        <button class="btn outline">Copy permalink</button>
        <button class="btn outline">Claude handoff</button>
        <button class="btn danger">Unregister</button>
      </div>
    </details>
    <details class="selection-section" open>
      <summary>Revision cycle</summary>
      <div class="selection-section-body">
        <p class="muted" style="font-size:0.85rem;margin:0">The current released PDF remains available while this newer draft is in review. After release, <strong>Begin revision</strong> returns the document to <code>draft</code>; PDFs and history remain preserved.</p>
        <p class="hint">Document-control-data changes while a review is open invalidate that review. Copy permalink uses workspace + document IDs only.</p>
      </div>
    </details>
    <details class="selection-section" open>
      <summary>Releases <span>4 recorded</span></summary>
      <div class="selection-section-body stack">
        ${ver("V1.3", "2025-08-01 09:44 UTC", "Substantive / major", "current", "ok")}
        ${ver("V1.2", "2025-07-12 12:01 UTC", "Cosmetic / minor", "superseded", "muted")}
        ${ver("V1.1", "2025-06-05 14:30 UTC", "Substantive / major", "superseded", "muted")}
        ${ver("V1.0", "2025-05-09 10:12 UTC", "Cosmetic / minor", "withdrawn", "warn")}
      </div>
    </details>
  </aside>`;
}
function batchSelectionPane() {
  return `<section class="batch-selection-demo">
    <div>
      <h3 class="card-title">In-library multi-select state</h3>
      <p class="muted" style="margin:0">Checking two or more in-library documents replaces the single-document sections in this same right pane.</p>
    </div>
    <aside class="card detail-pane batch-detail-pane">
      <div class="row between mb">
        <h3 class="card-title" style="margin:0">2 selected</h3>
        <button class="btn outline">Clear</button>
      </div>
      <details class="selection-section" open>
        <summary>Batch summary</summary>
        <div class="selection-section-body">
          <ul class="list">
            <li>HR Data Privacy Policy (DOC-014)</li>
            <li>Code of Conduct (DOC-018)</li>
          </ul>
        </div>
      </details>
      <details class="selection-section" open>
        <summary>Batch actions <span>2 available</span></summary>
        <div class="selection-section-body stack-btns">
          <button class="btn outline">Verify integrity</button>
          <button class="btn danger">Unregister…</button>
        </div>
      </details>
      <p class="hint">Batch actions only. Single-document actions remain hidden until exactly one document is selected.</p>
    </aside>
  </section>`;
}
function person(id, name, email, kind) {
  return `<div class="person"><div class="row gap-2"><strong>${name}</strong><span class="muted">${id}</span>${badge(kind === "ok" ? "active" : "disabled", kind)}</div><p class="muted">${email}</p></div>`;
}

function shell(cap) {
  const navHtml = NAV.map((n) => {
    const active = n.id === cap.nav ? " active" : "";
    return `<a class="nav-item${active}" href="#">${n.icon} <span>${n.label}</span></a>`;
  }).join("");
  const illustratedActivity = {
    "CAP-0002": "review-doc-77a12bce",
    "CAP-0003": "notes-doc-77a12bce",
  };
  const activityKey = cap.activity || illustratedActivity[cap.id] || cap.nav || "library";
  const activities = [
    { id: "library", label: "Library · policies/HR" },
    { id: "document-home-doc-77a12bce", label: "Document · HR Data Privacy Policy · DOC-014" },
    { id: "audit-doc-77a12bce", label: "Audit · HR Data Privacy Policy · DOC-014" },
    { id: "review-doc-77a12bce", label: "Review · HR Data Privacy Policy · DOC-014" },
    { id: "notes-doc-77a12bce", label: "Notes · HR Data Privacy Policy · DOC-014" },
    { id: "shell", label: "Shell chrome" },
    { id: "releases", label: "Releases" },
    { id: "audit", label: "Audit" },
    { id: "maintenance", label: "Maintenance" },
    { id: "config", label: "Configuration" },
  ];
  const activeActivity = activities.find((a) => a.id === activityKey) || {
    id: activityKey,
    label: cap.title,
  };
  const isBookmarked = cap.bookmarked === true;
  const bookmarkControl = `<button class="btn outline bookmark-btn${isBookmarked ? " saved" : ""}" title="${isBookmarked ? "Remove bookmark" : "Bookmark this view"}" aria-pressed="${isBookmarked}">${isBookmarked ? "★ Bookmarked" : "☆ Bookmark this view"}</button>`;
  const savedViews = [
    { label: "Library · policies/HR" },
    ...(isBookmarked ? [{ label: activeActivity.label }] : []),
  ];
  const savedViewTabs = savedViews
    .map(
      (view) =>
        `<div class="pane-tab saved-view"><span class="bookmark-mark">★</span><span class="pane-label" title="${view.label}">${view.label}</span><span class="pane-remove" title="Remove saved view">−</span></div>`
    )
    .join("");
  // Show a stable sample set; mark the one matching this screen active.
  const matchingActivity = activities.find((a) => a.id === activityKey);
  const openSet = [
    { id: "library", label: "Library · policies/HR" },
    { id: "audit-doc-77a12bce", label: "Audit · HR Data Privacy Policy · DOC-014" },
    { id: "review-doc-77a12bce", label: "Review · HR Data Privacy Policy · DOC-014" },
    { id: "notes-doc-77a12bce", label: "Notes · HR Data Privacy Policy · DOC-014" },
    ...(matchingActivity
      ? [{ id: matchingActivity.id, label: matchingActivity.label }]
      : [{ id: activityKey, label: cap.title }]),
  ];
  const seen = new Set();
  const openTabs = openSet
    .filter((t) => {
      if (seen.has(t.id)) return false;
      seen.add(t.id);
      return true;
    })
    .map((t) => {
      const isActive = t.id === activityKey;
      return `<div class="pane-tab${isActive ? " active" : ""}"><span class="pane-label" title="${t.label}">${t.label}</span><span class="pane-close">×</span></div>`;
    })
    .join("");
  const actions = (cap.actions || [])
    .map((a, i) => {
      const cls = i === 0 ? "btn" : a.toLowerCase().includes("obsolete") || a.toLowerCase().includes("restore") || a.toLowerCase().includes("archive") || a.toLowerCase().includes("withdraw") ? "btn danger" : "btn outline";
      return `<button class="${cls}">${a}</button>`;
    })
    .join("");

  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>${cap.id} — ${cap.title} · DMS Desktop</title>
<link rel="preconnect" href="https://fonts.googleapis.com"/>
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet"/>
<style>
:root {
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.145 0 0);
  --primary: oklch(0.205 0 0);
  --primary-foreground: oklch(0.985 0 0);
  --secondary: oklch(0.97 0 0);
  --secondary-foreground: oklch(0.205 0 0);
  --muted: oklch(0.97 0 0);
  --muted-foreground: oklch(0.556 0 0);
  --accent: oklch(0.97 0 0);
  --accent-foreground: oklch(0.205 0 0);
  --destructive: oklch(0.577 0.245 27.325);
  --border: oklch(0.922 0 0);
  --input: oklch(0.922 0 0);
  --ring: oklch(0.708 0 0);
  --sidebar: oklch(0.985 0 0);
  --sidebar-foreground: oklch(0.145 0 0);
  --sidebar-accent: oklch(0.97 0 0);
  --sidebar-border: oklch(0.922 0 0);
  --success: oklch(0.545 0.166 156.743);
  --warning: oklch(0.754 0.184 85.869);
  --radius: 0.625rem;
  --info: oklch(0.546 0.245 262.881);
  font-family: Inter, ui-sans-serif, system-ui, sans-serif;
  color: var(--foreground);
  background: var(--background);
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--background); color: var(--foreground); }
a { color: inherit; text-decoration: none; }
code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.85em; background: var(--muted); padding: 0.1em 0.35em; border-radius: 0.25rem; }
.app { display: flex; min-height: 100vh; width: 1440px; max-width: 100%; margin: 0 auto; border-left: 1px solid var(--border); border-right: 1px solid var(--border); }
.sidebar { width: 16rem; flex-shrink: 0; background: var(--sidebar); border-right: 1px solid var(--sidebar-border); display: flex; flex-direction: column; padding: 0.75rem; gap: 0.5rem; }
.brand { display: flex; align-items: center; gap: 0.625rem; padding: 0.5rem 0.625rem; font-weight: 600; font-size: 0.9rem; }
.brand-mark { width: 1.75rem; height: 1.75rem; border-radius: 0.4rem; background: var(--primary); color: var(--primary-foreground); display: grid; place-items: center; font-size: 0.7rem; font-weight: 700; }
.brand-actions { margin-left: auto; display: flex; gap: 0.25rem; }
.fold-ctl { appearance: none; border: 1px solid var(--input); background: var(--background); width: 1.5rem; height: 1.5rem; border-radius: calc(var(--radius) - 2px); font: inherit; font-size: 0.75rem; line-height: 1; cursor: default; color: var(--muted-foreground); }
.nav { display: flex; flex-direction: column; gap: 0.15rem; margin-top: 0.35rem; }
.nav-item { display: flex; align-items: center; gap: 0.6rem; padding: 0.45rem 0.625rem; border-radius: calc(var(--radius) - 2px); font-size: 0.875rem; color: var(--sidebar-foreground); }
.nav-item.active { background: var(--sidebar-accent); font-weight: 600; }
.pane-sec { margin-top: 0.85rem; padding: 0 0.35rem; font-size: 0.68rem; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; color: var(--muted-foreground); }
.pane-tabs { display: flex; flex-direction: column; gap: 0.2rem; margin-top: 0.35rem; }
.pane-tab { display: flex; align-items: center; gap: 0.35rem; padding: 0.4rem 0.55rem; border-radius: calc(var(--radius) - 2px); font-size: 0.78rem; border: 1px solid transparent; background: transparent; color: var(--sidebar-foreground); }
.pane-tab.active { background: color-mix(in oklch, var(--info) 12%, white); border-color: color-mix(in oklch, var(--info) 28%, var(--border)); font-weight: 600; }
.pane-tab.saved-view { color: var(--muted-foreground); }
.bookmark-mark { color: var(--warning); font-size: 0.72rem; }
.pane-label { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
.pane-close { color: var(--muted-foreground); font-size: 0.85rem; line-height: 1; padding: 0 0.15rem; }
.pane-remove { color: var(--muted-foreground); font-size: 0.95rem; line-height: 1; padding: 0 0.15rem; }
.sidebar-foot { margin-top: auto; padding: 0.75rem 0.625rem; font-size: 0.75rem; color: var(--muted-foreground); line-height: 1.5; border-top: 1px solid var(--sidebar-border); }
.main { flex: 1; min-width: 0; display: flex; flex-direction: column; background: var(--background); }
.header { height: 3.5rem; border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 0.75rem; padding: 0 1.25rem; }
.header h1 { font-size: 0.95rem; font-weight: 600; margin: 0; }
.header .grow { flex: 1; }
.header-actions { display: flex; gap: 0.5rem; align-items: center; }
.bookmark-btn { white-space: nowrap; }
.bookmark-btn.saved { border-color: color-mix(in oklch, var(--warning) 50%, var(--border)); color: oklch(0.45 0.1 85); background: color-mix(in oklch, var(--warning) 14%, white); }
.ham-btn { appearance: none; border: 1px solid var(--input); background: var(--background); width: 2rem; height: 2rem; border-radius: calc(var(--radius) - 2px); font: inherit; font-size: 1rem; line-height: 1; cursor: default; color: var(--foreground); display: grid; place-items: center; }
.ham-btn .tip { display: none; }
.ham-note { font-size: 0.7rem; color: var(--muted-foreground); }
.content { padding: 1.25rem 1.5rem 2rem; display: flex; flex-direction: column; gap: 1rem; }
.cap-head { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 0.75rem; }
.cap-head h2 { margin: 0; font-size: 1.25rem; font-weight: 700; }
.subtitle { margin: 0; color: var(--muted-foreground); font-size: 0.875rem; max-width: 60rem; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: var(--radius); padding: 1rem 1.1rem; }
.card-title { margin: 0 0 0.6rem; font-size: 0.95rem; font-weight: 600; }
.grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.grid-explorer { display: grid; grid-template-columns: 16rem 1fr; gap: 1rem; }
.grid-explorer-detail { display: grid; grid-template-columns: 13.5rem minmax(0, 1fr) 18.5rem; gap: 0.85rem; align-items: start; }
.list-card { position: relative; z-index: 1; min-width: 0; }
.detail-pane { position: relative; z-index: 1; min-width: 0; }
.detail-pane .kv > div { grid-template-columns: 7.5rem 1fr; gap: 0.45rem; font-size: 0.8rem; }
.detail-pane .doc-title { font-size: 1rem; margin-bottom: 0.15rem; }
.selection-section { margin-top: 0.8rem; border-top: 1px solid var(--border); padding-top: 0.65rem; }
.selection-section summary { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; cursor: default; font-size: 0.88rem; font-weight: 600; list-style: none; }
.selection-section summary::-webkit-details-marker { display: none; }
.selection-section summary::after { content: "⌄"; color: var(--muted-foreground); font-size: 0.9rem; }
.selection-section:not([open]) summary::after { content: "›"; }
.selection-section summary span { margin-left: auto; color: var(--muted-foreground); font-size: 0.7rem; font-weight: 500; }
.selection-section-body { margin-top: 0.65rem; }
.batch-selection-demo { display: grid; grid-template-columns: minmax(0, 1fr) 18.5rem; gap: 0.85rem; align-items: start; margin-top: 0.25rem; }
.batch-detail-pane { min-height: 0; }
.selection-bar { display: flex; flex-wrap: nowrap; align-items: center; gap: 0.4rem; margin-bottom: 0.75rem; padding: 0.5rem 0.65rem; border: 1px solid color-mix(in oklch, var(--info) 28%, var(--border)); background: color-mix(in oklch, var(--info) 8%, white); border-radius: calc(var(--radius) - 2px); font-size: 0.8rem; overflow-x: auto; }
.selection-bar .btn { height: 1.75rem; font-size: 0.75rem; padding: 0 0.65rem; flex-shrink: 0; }
.selection-bar .sel-sep { width: 1px; height: 1.1rem; background: var(--border); margin: 0 0.15rem; flex-shrink: 0; }
.row-menu-demo td { background: color-mix(in oklch, var(--info) 6%, white); padding: 0.55rem 0.75rem 0.75rem; border-bottom: 1px solid var(--border); }
.row-menu-panel { border: 1px solid var(--border); background: var(--card); border-radius: calc(var(--radius) - 2px); padding: 0.55rem 0.65rem; }
.row-menu-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 0.25rem 0.75rem; font-size: 0.75rem; }
.row-menu-grid .danger { color: var(--destructive); }
.row-menu-grid .muted { color: var(--muted-foreground); }
.stack { display: flex; flex-direction: column; gap: 0.75rem; }
.stack-btns { display: flex; flex-direction: column; gap: 0.4rem; }
.row { display: flex; align-items: center; }
.row.between { justify-content: space-between; }
.gap-2 { gap: 0.5rem; }
.mb { margin-bottom: 0.75rem; }
.grow { flex: 1; }
.muted { color: var(--muted-foreground); font-size: 0.875rem; }
.hint { color: var(--muted-foreground); font-size: 0.78rem; margin: 0.75rem 0 0; }
.danger { color: var(--destructive); }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.btn { appearance: none; border: 1px solid transparent; background: var(--primary); color: var(--primary-foreground); font: inherit; font-size: 0.8rem; font-weight: 600; height: 2rem; padding: 0 0.85rem; border-radius: calc(var(--radius) - 2px); cursor: default; }
.btn.outline { background: var(--background); color: var(--foreground); border-color: var(--input); }
.btn.danger { background: color-mix(in oklch, var(--destructive) 12%, white); color: var(--destructive); border-color: color-mix(in oklch, var(--destructive) 30%, white); }
.icon-btn { appearance: none; border: 1px solid var(--input); background: var(--background); width: 1.75rem; height: 1.75rem; border-radius: calc(var(--radius) - 2px); font: inherit; font-weight: 700; line-height: 1; color: var(--foreground); cursor: default; }
.icon-btn.active { border-color: var(--info); background: color-mix(in oklch, var(--info) 12%, white); color: var(--info); }
.check { color: var(--muted-foreground); font-size: 0.95rem; }
.check.on { color: var(--info); font-weight: 700; }
tr.selected td { background: color-mix(in oklch, var(--info) 8%, white); }
th.col-actions, td.col-actions { width: 2.5rem; text-align: center; padding-left: 0.35rem; padding-right: 0.35rem; }
.menu-label { font-size: 0.68rem; font-weight: 700; letter-spacing: 0.02em; text-transform: uppercase; color: var(--muted-foreground); padding: 0 0 0.4rem; }
.badge { display: inline-flex; align-items: center; border-radius: 999px; padding: 0.1rem 0.55rem; font-size: 0.7rem; font-weight: 600; background: var(--muted); color: var(--foreground); white-space: nowrap; }
.badge.ok { background: color-mix(in oklch, var(--success) 18%, white); color: var(--success); }
.badge.warn { background: color-mix(in oklch, var(--warning) 22%, white); color: oklch(0.5 0.12 85); }
.badge.danger { background: color-mix(in oklch, var(--destructive) 14%, white); color: var(--destructive); }
.badge.info { background: color-mix(in oklch, var(--info) 16%, white); color: var(--info); }
.badge.muted { background: var(--muted); color: var(--muted-foreground); }
.kv { margin: 0; display: grid; gap: 0.45rem; }
.kv > div { display: grid; grid-template-columns: 10.5rem 1fr; gap: 0.75rem; font-size: 0.875rem; }
.kv dt { color: var(--muted-foreground); margin: 0; }
.kv dd { margin: 0; }
.table-wrap { overflow: auto; border: 1px solid var(--border); border-radius: calc(var(--radius) - 2px); }
table { width: 100%; border-collapse: collapse; font-size: 0.8rem; }
th, td { text-align: left; padding: 0.55rem 0.75rem; border-bottom: 1px solid var(--border); vertical-align: middle; }
th { background: var(--muted); color: var(--muted-foreground); font-weight: 600; }
tr:last-child td { border-bottom: 0; }
.pipeline { display: grid; grid-template-columns: repeat(auto-fit, minmax(7rem, 1fr)); gap: 0.5rem; }
.step { background: var(--muted); border-radius: calc(var(--radius) - 2px); padding: 0.65rem 0.5rem; text-align: center; font-size: 0.75rem; font-weight: 600; display: flex; flex-direction: column; align-items: center; gap: 0.35rem; }
.step.active { background: color-mix(in oklch, var(--info) 12%, white); color: var(--info); }
.dot { width: 0.55rem; height: 0.55rem; border-radius: 999px; background: currentColor; }
.list { margin: 0; padding-left: 1.1rem; font-size: 0.875rem; line-height: 1.6; }
.danger-list li { color: var(--destructive); }
.callout { border-radius: var(--radius); padding: 0.75rem 0.9rem; font-size: 0.85rem; }
.callout.warn { background: color-mix(in oklch, var(--warning) 18%, white); color: oklch(0.45 0.1 85); }
.callout.ok { background: color-mix(in oklch, var(--success) 14%, white); color: var(--success); display: flex; gap: 0.5rem; align-items: center; }
.note, .event, .layer, .type-card, .ver, .person { border: 1px solid var(--border); border-radius: calc(var(--radius) - 2px); padding: 0.75rem 0.85rem; }
.note header, .event .row { font-size: 0.78rem; color: var(--muted-foreground); margin-bottom: 0.35rem; }
.note p, .event p, .layer p, .type-card p, .ver p, .person p { margin: 0.15rem 0 0; font-size: 0.875rem; }
.composer { margin-top: 0.9rem; border: 1px dashed var(--border); border-radius: var(--radius); padding: 0.85rem; display: flex; flex-direction: column; gap: 0.5rem; }
.label { font-size: 0.75rem; font-weight: 600; color: var(--muted-foreground); }
.textarea { min-height: 4rem; border: 1px solid var(--input); border-radius: calc(var(--radius) - 2px); padding: 0.6rem 0.75rem; color: var(--muted-foreground); font-size: 0.875rem; background: var(--background); }
.tree-list { list-style: none; margin: 0; padding: 0; font-size: 0.875rem; }
.tree-list li { padding: 0.35rem 0.5rem; border-radius: calc(var(--radius) - 2px); }
.tree-list li.active { background: color-mix(in oklch, var(--info) 12%, white); font-weight: 600; }
.tree { --tree-indent: 0.85rem; }
.tree-root { list-style: none; margin: 0; padding: 0; font-size: 0.84rem; }
.tree-root ul { list-style: none; margin: 0; padding: 0 0 0 var(--tree-indent); border-left: 1px solid var(--border); margin-left: 0.55rem; }
.tree-node { display: flex; align-items: center; gap: 0.35rem; padding: 0.28rem 0.4rem; border-radius: calc(var(--radius) - 2px); }
.tree-node.active { background: color-mix(in oklch, var(--info) 12%, white); font-weight: 600; }
.tree-twisty { width: 0.9rem; color: var(--muted-foreground); font-size: 0.7rem; flex-shrink: 0; text-align: center; }
.tree-twisty.empty { visibility: hidden; }
.tree-label { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tags { display: flex; flex-wrap: wrap; gap: 0.4rem; }
.tag { border: 1px solid var(--border); background: var(--background); border-radius: 999px; padding: 0.25rem 0.7rem; font-size: 0.78rem; }
.diff { border-radius: calc(var(--radius) - 2px); padding: 0.55rem 0.7rem; margin-bottom: 0.4rem; font-size: 0.85rem; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.diff .ctx { display: block; font-size: 0.7rem; color: var(--muted-foreground); margin-bottom: 0.15rem; font-family: Inter, sans-serif; }
.diff.ok { background: color-mix(in oklch, var(--success) 12%, white); color: var(--success); }
.diff.warn { background: color-mix(in oklch, var(--warning) 16%, white); color: oklch(0.5 0.12 85); }
.diff.info { background: color-mix(in oklch, var(--info) 12%, white); color: var(--info); }
.doc-title { margin: 0; font-size: 1.1rem; font-weight: 700; }
.timeline { margin: 0; padding-left: 1.1rem; font-size: 0.85rem; line-height: 1.7; }
.wire-meta { font-size: 0.7rem; color: var(--muted-foreground); }
.mini-shell { display: flex; border: 1px solid var(--border); border-radius: calc(var(--radius) - 2px); overflow: hidden; min-height: 12rem; background: var(--background); }
.mini-shell.collapsed { min-height: 11rem; }
.mini-side { width: 14rem; background: var(--sidebar); border-right: 1px solid var(--sidebar-border); padding: 0.5rem; display: flex; flex-direction: column; gap: 0.35rem; font-size: 0.72rem; }
.mini-brand { display: flex; align-items: center; justify-content: space-between; font-weight: 600; padding: 0.15rem 0.2rem; }
.fold-btn { border: 1px solid var(--input); border-radius: 0.25rem; padding: 0 0.3rem; color: var(--muted-foreground); background: var(--background); }
.mini-nav div, .mini-tabs div { padding: 0.28rem 0.35rem; border-radius: 0.3rem; }
.mini-nav div.on, .mini-tabs div.on, .mini-rail div.on { background: color-mix(in oklch, var(--info) 12%, white); font-weight: 600; }
.mini-sec { margin-top: 0.35rem; font-size: 0.62rem; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; color: var(--muted-foreground); }
.mini-tabs div { display: flex; justify-content: space-between; gap: 0.25rem; border: 1px solid transparent; }
.mini-tabs div.on { border-color: color-mix(in oklch, var(--info) 28%, var(--border)); }
.mini-bookmark { margin-left: auto; padding: 0.15rem 0.3rem; border: 1px solid var(--input); border-radius: 0.25rem; color: oklch(0.45 0.1 85); font-size: 0.62rem; font-weight: 500; }
.mini-foot { margin-top: auto; color: var(--muted-foreground); font-size: 0.65rem; padding-top: 0.4rem; border-top: 1px solid var(--sidebar-border); }
.mini-main { flex: 1; background: var(--card); display: flex; flex-direction: column; color: var(--muted-foreground); font-size: 0.8rem; align-items: stretch; justify-content: center; text-align: center; }
.mini-rail { width: 2.75rem; background: var(--sidebar); border-right: 1px solid var(--sidebar-border); display: flex; flex-direction: column; align-items: center; gap: 0.35rem; padding: 0.45rem 0.25rem; }
.mini-rail div, .ham { width: 1.75rem; height: 1.75rem; display: grid; place-items: center; border-radius: 0.35rem; font-size: 0.85rem; }
.ham { border: 1px solid var(--input); background: var(--background); }
.mini-header { display: flex; align-items: center; gap: 0.5rem; padding: 0.55rem 0.75rem; border-bottom: 1px solid var(--border); font-weight: 600; color: var(--foreground); text-align: left; }
</style>
</head>
<body>
<div class="app" data-cap="${cap.id}">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">DMS</div>
      <span>DMS Desktop</span>
      <div class="brand-actions"><button class="fold-ctl" title="Collapse left menu">«</button></div>
    </div>
    <nav class="nav">${navHtml}</nav>
    <div class="pane-sec">Saved views</div>
    <div class="pane-tabs">${savedViewTabs}</div>
    <div class="pane-sec">Open panes</div>
    <div class="pane-tabs">${openTabs}</div>
    <div class="sidebar-foot">
      ws-9c3b7d1a<br/>
      edit: /dms/edit<br/>
      publish: /dms/publish
    </div>
  </aside>
  <div class="main">
    <header class="header">
      <button class="ham-btn" title="Open left menu when folded">☰</button>
      <h1>${activeActivity.label}</h1>
      <div class="grow"></div>
      <span class="ham-note">☰ when menu folded</span>
      ${badge("Workspace healthy", "ok")}
      <div class="header-actions">${bookmarkControl}${actions}</div>
    </header>
    <main class="content">
      <div class="cap-head">
        ${badge(cap.id, "info")}
        <h2>${cap.title}</h2>
        ${badge("Status: not implemented", "muted")}
        <span class="wire-meta">Wireframe · shadcn-admin 2.2.0 visual base</span>
      </div>
      <p class="subtitle">${cap.subtitle}</p>
      ${cap.body}
    </main>
  </div>
</div>
</body>
</html>`;
}

fs.mkdirSync(OUT, { recursive: true });
fs.mkdirSync(EXPORTS, { recursive: true });

const indexRows = [];
for (const cap of CAPS) {
  const html = shell(cap);
  const name = `${cap.file}.html`;
  fs.writeFileSync(path.join(OUT, name), html);
  indexRows.push({ ...cap, html: name, png: `${cap.file}.png` });
  console.log("wrote", name);
}

const index = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"/><title>DMS capability wireframes</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap" rel="stylesheet"/>
<style>
body{font-family:Inter,system-ui,sans-serif;margin:2rem;color:#111}
table{border-collapse:collapse;width:100%;max-width:56rem}
th,td{border:1px solid #e5e7eb;padding:.55rem .7rem;text-align:left;font-size:.9rem}
th{background:#f4f4f5}
a{color:#2563eb}
.muted{color:#6b7280;font-size:.85rem}
</style></head><body>
<h1>DMS capability wireframes</h1>
<p class="muted">Visual base: <code>shadcn-admin-2.2.0</code> (shadcn/ui 4 tokens + sidebar shell). Static HTML for CAP contracts.</p>
<table>
<thead><tr><th>ID</th><th>Title</th><th>HTML</th><th>PNG</th></tr></thead>
<tbody>
${indexRows
  .map(
    (r) =>
      `<tr><td>${r.id}</td><td>${r.title}</td><td><a href="./html/${r.html}">${r.html}</a></td><td><a href="./exports/${r.png}">${r.png}</a></td></tr>`
  )
  .join("\n")}
</tbody></table>
</body></html>`;
fs.writeFileSync(path.join(__dirname, "index.html"), index);

const manifest = {
  base: "shadcn-admin-2.2.0",
  source: "../../../../rb/shadcn-admin-2.2.0",
  screens: indexRows.map((r) => ({
    id: r.id,
    title: r.title,
    html: `html/${r.html}`,
    png: `exports/${r.png}`,
    nav: r.nav,
  })),
};
fs.writeFileSync(path.join(__dirname, "manifest.json"), JSON.stringify(manifest, null, 2));
console.log("index + manifest written");
console.log(JSON.stringify(manifest.screens.map((s) => s.id)));
