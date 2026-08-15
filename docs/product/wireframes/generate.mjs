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

/** @type {Array<{id:string,file:string,title:string,nav:string,subtitle:string,status?:string,actions?:string[],bookmarked?:boolean,body:string}>} */
const CAPS = [
  {
    id: "CAP-0001",
    file: "CAP-0001-local-folder-dms",
    title: "Workspace configuration",
    nav: "config",
    configSection: "workspace",
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
            <li>Microsoft Entra tenant/group binding</li>
            <li>Read-only Entra display cache (no user roster)</li>
            <li>Relative folder policies</li>
            <li>Document control data</li>
            <li>Release records &amp; checksums</li>
          </ul>
          <div class="callout warn">No SMTP password is stored here — relay credentials live in the OS credential store.</div>
        </section>
      </div>
      <section class="card">
        <div class="row between" style="gap:1rem;flex-wrap:wrap">
          <div><h3 class="card-title" style="margin-bottom:0.25rem">No workspace yet</h3><p class="muted" style="margin:0">At first launch, Configuration offers only this entry. Document defaults, Workflow, and Notifications become available after the operator confirms workspace initialization.</p></div>
          <button class="btn">Set up workspace</button>
        </div>
      </section>`,
  },
  {
    id: "CAP-0002",
    file: "CAP-0002-document-lifecycle",
    title: "Document lifecycle",
    nav: "library",
    subtitle: "Next minor is the default target. Major changes require approval; minor changes release directly and notify the assigned approver after publication.",
    actions: ["Begin revision", "Release V1.4 minor version", "Preview V2.0 review request"],
    body: `
      <section class="card">
        <div class="row between">
          <div class="row gap-2">
            <h2 class="doc-title">HR Data Privacy Policy</h2>
            ${badge("draft", "warn")}
            <span class=\"muted\">V1.3 released · candidate V1.4 is not occupied</span>
          </div>
        </div>
        ${kv([
          ["Document ID", "doc-77a12bce"],
          ["Relative draft path", "policies/HR/Handbook.docx"],
          ["Owner", "Lukas Roth · object ID 8a1f…"],
          ["Effective editor", "Lukas Roth · object ID 8a1f…"],
          ["Effective approver", "Anna Berg · object ID 41c2…"],
          ["Workflow identity", "Entra group: VC DMS Workflow Users"],
          ["Effective confidentiality", "Internal (inherited from /policies/HR)"],
        ])}
      </section>
      <section class="card">
        <h3 class="card-title">Lifecycle pipeline</h3>
        <div class="pipeline">
          ${["draft", "in_review", "approved", "released", "obsolete"]
            .map((s, i) => `<div class="step ${i < 1 ? "active" : ""}"><span class="dot"></span>${s}</div>`)
            .join("")}
        </div>
      </section>
      <div class="grid-2">
        <section class="card">
          <h3 class="card-title">Release candidate (required fields)</h3>
          ${kv([
            ["Changelog *", "Updated retention table to 24 months."],
            ["Effective date *", "2025-08-15 <span class=\"muted\">(captured only by successful release)</span>"],
            ["Owner *", "Lukas Roth · lukas@vc.de · object ID 8a1f…"],
            ["Requesting editor *", "Lukas Roth · object ID 8a1f…"],
            ["Target version *", "Next minor V1.4 <span class=\"muted\">(default)</span> · Next major V2.0 · Manual V&lt;major&gt;.&lt;minor&gt;"],
            ["Manual validation", "<span class=\"muted\">Greater unused target required when manual is selected</span>"],
            ["Candidate", "V1.4 <span class=\"muted\">(minor release; no approval required)</span>"],
            ["Draft SHA-256", "<span class=\"muted\">(computed from current draft bytes before release)</span>"],
            ["Released by", "Lukas Roth <lukas@vc.de> <span class=\"muted\">(snapshotted on release)</span>"],
            ["Approver notification", "Anna Berg <anna@vc.de> · effective approver snapshot"],
            ["Approval rule", "V1.0 and targets that increase the major component require Entra-verified approval"],
          ])}
          <form class="stack" style="margin-top:0.75rem">
            <label class="label">Effective date * <input type="date" value="2025-08-15" required></label>
            <label class="label">Owner * <select required><option value="8a1f">Lukas Roth · lukas@vc.de</option><option value="41c2">Anna Berg · anna@vc.de</option></select></label>
            <label class="label">Requesting editor * <select required><option value="8a1f">Lukas Roth · lukas@vc.de</option></select></label>
            <label class="label">Target * <select required><option value="next_minor" selected>Next minor · V1.4</option><option value="next_major">Next major · V2.0</option><option value="manual">Manual</option></select></label>
            <div class="row gap-2" style="flex-wrap:wrap"><button class="btn outline">Preview V2.0 review request</button><button class="btn">Release V1.4 minor version</button></div>
          </form>
          <p class="hint">A successful empty people import shows literal <code>&lt;owner&gt;</code> and <code>&lt;editor&gt;</code> placeholders here and blocks submission. Minor release snapshots the requested profile, effective date, changelog, mode, editor, and approver; it stays in <code>draft</code> until successful atomic export.</p>
        </section>
        <section class="card">
          <h3 class="card-title">Major approval and minor publication rule</h3>
          <ul class="timeline">
            <li><strong>Current V1.3</strong> — effective 2025-08-01; owner Lukas Roth (object ID 8a1f…)</li>
            <li><strong>V1.4 minor</strong> — direct release; requested effective date and profile become immutable only after export commits</li>
            <li><strong>minor publication notice</strong> — notify Anna after V1.4 atomic export commits</li>
            <li><strong>V2.0 major</strong> — review request, Entra-verified decision, then release</li>
            <li><strong>rejected / changes requested</strong> — major candidate stays unoccupied; optional reason retained</li>
          </ul>
          <div class="callout warn">A rejected, cancelled, invalidated, or failed-export major review does not consume its candidate. A failed minor export does not consume V1.4 or apply staged Owner/Editor changes.</div>
          <p class="hint">Chain head 5b3a…ffe2 — verify recomputes from canonical body (CAP-0011).</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0003",
    file: "CAP-0003-document-notes",
    title: "Document notes",
    nav: "library",
    status: "implemented",
    subtitle: "Free-text notes by stable document ID. Newest first; New note field above the latest note.",
    actions: ["Add note"],
    body: `
      <section class="card">
        <div class="row between" style="margin-bottom:0.75rem;gap:1rem;flex-wrap:wrap">
          <button class="btn outline" aria-label="Back to Library with selected document HR Data Privacy Policy">← Back to Library</button>
          <span class="muted">Returns to Library · policies/HR with this document selected; Notes stays open.</span>
        </div>
        <h3 class="card-title">Notes — HR Data Privacy Policy <span class="muted">(doc-77a12bce)</span></h3>
        <div class="composer">
          <label class="label">New note</label>
          <div class="textarea">Plain text — line breaks preserved. UTF-8.</div>
          <div class="row between">
            <span class="muted">Author: Lukas Roth (OS user)</span>
            <button class="btn">Save note</button>
          </div>
        </div>
        <div class="stack">
          ${note("2025-08-02 11:08 — Lukas Roth", "Confirmed retention table updated to 24 months; double-check audit log entry.")}
          ${note("2025-07-29 16:22 — Anna Berg", "Need legal review wording on §4 before next release.")}
          ${note("2025-07-21 09:05 — Lukas Roth", "Renamed draft locally — locator updated, ID preserved.")}
        </div>
        <p class="hint">List is newest-first. The compose field stays above the latest note. Deleting a note never deletes the document or workflow evidence comments (CAP-0011).</p>
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
        ${growingTable({
          headers: ["Version", "Relative path", "Released", "SHA-256", "Result", "Action"],
          rows: [
            ["V2.0", "policies/HR/Handbook_V2.0_restricted.pdf", "2025-08-02 09:44", "9f2c…b1e0", badge("match", "ok"), "Reveal"],
            ["V1.7", "policies/HR/Handbook_V1.7_restricted.pdf", "2025-07-12 12:01", "73b1…4cd2", badge("match", "ok"), "Reveal"],
            ["V1.6", "policies/HR/Handbook_V1.6_restricted.pdf", "2025-06-05 14:30", "2a91…77ee", badge("mismatch", "danger"), "Reveal"],
            ["V1.5", "policies/HR/Handbook_V1.5_restricted.pdf", "2025-05-09 10:12", "—", badge("missing file", "warn"), "Reveal"],
          ],
          filterLabel: "Version or path",
          filterAriaLabel: "Filter integrity results",
          filterPlaceholder: "e.g. V2.0 or mismatch",
          matchingLabel: "verification results",
        })}
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
    subtitle: "Tauri 2 shell on Windows and macOS. Foldable left menu, icon-rail flyouts, session-only activities, explicit saved views, and viewport-contained activity scrolling.",
    body: `
      ${viewportScrollStyles("CAP-0005")}
      <section class="card">
        <h3 class="card-title">Chrome contract</h3>
        <div class="stack">
          ${layer("Foldable left menu", "Primary destinations, Saved views, and Open panes. Expanded/collapsed preference persists per OS user (not in .dms).")}
          ${layer("Hamburger when folded", "Header control re-opens the menu as temporary expand/overlay; pin expanded to keep it open.")}
          ${layer("Collapsed group flyouts", "Star and pane icons in the collapsed rail open only their Saved views or Open panes flyout; they do not expand the full left menu. Each flyout retains full labels plus open/remove or focus/close actions.")}
          ${layer("Open activity panes/tabs", "Automatic, session-only quicklinks. Labels state task + target: Audit · HR Data Privacy Policy · DOC-014 for a document or Library · policies/HR for a folder. Opening the same task + document focuses its existing pane; × closes that activity only.")}
          ${layer("Saved views", "Use ☆ Bookmark this view in the header. ★ Bookmarked is an explicit, per-user shortcut restored after relaunch; it is not a .dms workflow record.")}
          ${layer("Viewport-contained scrolling", "The sidebar and activity header stay available. Normal activities scroll inside main content; multi-pane workspaces give navigation, lists, and exhaustive details separate scroll regions.")}
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
              <div style="width:1.5rem;height:1px;background:var(--border);margin:0.1rem 0"></div>
              <div class="on" title="Saved views (2)">★</div>
              <div title="Open panes (3)">▤</div>
            </div>
            <div class="mini-main">
              <div class="mini-header"><span class="ham">☰</span> Audit · HR Data Privacy Policy · DOC-014</div>
              <div style="align-self:flex-start;width:12.5rem;margin:0.75rem;text-align:left;border:1px solid var(--border);border-radius:0.35rem;background:var(--background);box-shadow:0 0.25rem 0.75rem color-mix(in oklch,var(--foreground) 10%,transparent);font-size:0.7rem">
                <div style="display:flex;justify-content:space-between;padding:0.45rem 0.55rem;border-bottom:1px solid var(--border);font-weight:700;color:var(--foreground)"><span>Saved views</span><span class="muted" style="font-size:0.65rem">2</span></div>
                <div style="padding:0.42rem 0.55rem">★ Library · policies/HR <span class="muted" style="float:right">Open · −</span></div>
                <div style="padding:0.42rem 0.55rem">★ Shell chrome <span class="muted" style="float:right">Open · −</span></div>
                <div style="display:flex;justify-content:space-between;padding:0.45rem 0.55rem;border-top:1px solid var(--border);color:var(--muted-foreground)"><span>▤ Open panes</span><span>3 ›</span></div>
              </div>
              <p class="muted" style="padding:0 0.75rem;margin:0;font-size:0.75rem">★ and ▤ open only their group flyout. The hamburger still expands the full menu.</p>
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
    subtitle: "Persistent folder tree + Explorer-like path controls + exact source file names. The toolbar stays fixed while tree, list, and selection details scroll independently.",
    actions: [],
    body: `
      ${viewportScrollStyles("CAP-0006", `
        .app[data-cap="CAP-0006"] .list-card th,
        .app[data-cap="CAP-0006"] .list-card td { padding: 0.55rem 0.45rem; font-size: 0.75rem; }
        .app[data-cap="CAP-0006"] .explorer-toolbar { position: sticky; z-index: 4; top: 0; }
        .app[data-cap="CAP-0006"] .explorer-panes { height: min(36rem, calc(100vh - 14rem)); min-height: 24rem; align-items: stretch; }
        .app[data-cap="CAP-0006"] .explorer-panes > .card { min-height: 0; overflow-y: auto; overscroll-behavior: contain; scrollbar-gutter: stable; }
        .app[data-cap="CAP-0006"] .explorer-panes .list-card { display: flex; flex-direction: column; }
        .app[data-cap="CAP-0006"] .explorer-panes .table-wrap { min-height: 0; flex: 1; }
        .app[data-cap="CAP-0006"] .list-card table { table-layout: fixed; }
        .app[data-cap="CAP-0006"] .list-card th:nth-child(1) { width: 1.4rem; }
        .app[data-cap="CAP-0006"] .list-card th:nth-child(2) { width: 5.3rem; }
        .app[data-cap="CAP-0006"] .list-card th:nth-child(3) { width: 4.3rem; }
        .app[data-cap="CAP-0006"] .list-card th:nth-child(4) { width: 5.6rem; }
        .app[data-cap="CAP-0006"] .list-card th:nth-child(5),
        .app[data-cap="CAP-0006"] .list-card th:nth-child(6) { width: 3.5rem; }
      `)}
      <section class="card explorer-toolbar" style="padding:0.75rem 0.9rem">
        <div class="row gap-2">
          <button class="icon-btn" title="Back" aria-label="Back">${wireframeIcon("back")}</button>
          <button class="icon-btn" title="Forward" aria-label="Forward">${wireframeIcon("forward")}</button>
          <button class="icon-btn" title="Up one folder" aria-label="Up one folder">${wireframeIcon("up")}</button>
          <button class="icon-btn" title="Refresh current folder (F5)" aria-label="Refresh current folder">${wireframeIcon("refresh")}</button>
          <div class="row" style="height:2rem;min-width:0;flex:1;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.75rem;font-size:0.82rem;gap:0.45rem">
            <span class="muted">DMS Workspace</span><span>›</span><span>policies</span><span>›</span><strong>HR</strong>
          </div>
          <label class="muted" style="display:flex;align-items:center;gap:0.45rem">Search <input aria-label="Search current folder and descendants" placeholder="HR and subfolders" style="height:2rem;width:16rem;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.7rem;font:inherit;color:var(--foreground);background:var(--background)"/></label>
        </div>
        <p class="hint" style="margin-top:0.45rem">Back / Forward / Up and clickable breadcrumbs stay synchronized and remain available while any pane scrolls.</p>
      </section>
      <div class="grid-explorer-detail explorer-panes" style="grid-template-columns:17.5rem minmax(22.5rem,1fr) 0.45rem 20rem;align-items:stretch">
        <aside class="card tree">
          <div class="row between mb">
            <h3 class="card-title" style="margin:0">Folders</h3>
            <span class="muted" style="font-size:0.72rem">resize ↔</span>
          </div>
          <p class="hint" style="margin-top:0">Edit-root folders · <code>.dms</code> hidden</p>
          <ul class="tree-root">
            <li>
              <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">${wireframeIcon("folder")} DMS Workspace ${folderCounter("~2", "2 draft documents")} ${folderCounter("+2", "2 files available to add")} ${folderCounter("!1", "1 unsupported file")}</span></div>
              <ul>
                <li>
                  <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">${wireframeIcon("folder")} policies ${folderCounter("~2", "2 draft documents")} ${folderCounter("+2", "2 files available to add")} ${folderCounter("!1", "1 unsupported file")}</span></div>
                  <ul>
                    <li>
                      <div class="tree-node active"><span class="tree-twisty">▾</span><span class="tree-label">${wireframeIcon("folder")} HR ${folderCounter("~2", "2 draft documents")} ${folderCounter("+2", "2 files available to add")} ${folderCounter("!1", "1 unsupported file")}</span></div>
                      <ul>
                        <li><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">${wireframeIcon("folder")} Recruiting ${folderCounter("~1", "1 draft document")}</span></div></li>
                        <li><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">${wireframeIcon("folder")} Templates ${folderCounter("+1", "1 file available to add")}</span></div></li>
                      </ul>
                    </li>
                    <li>
                      <div class="tree-node"><span class="tree-twisty">▸</span><span class="tree-label">${wireframeIcon("folder")} IT</span></div>
                    </li>
                  </ul>
                </li>
                <li>
                  <div class="tree-node"><span class="tree-twisty">▸</span><span class="tree-label">${wireframeIcon("folder")} procedures</span></div>
                </li>
                <li>
                  <div class="tree-node"><span class="tree-twisty">▸</span><span class="tree-label">${wireframeIcon("folder")} records</span></div>
                </li>
                <li>
                  <div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">${wireframeIcon("folder")} Archive (empty)</span></div>
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
          <div class="row gap-2 mb" style="flex-wrap:wrap" aria-label="Show in folder">
            <strong style="font-size:0.75rem">Show in folder</strong>
            <button class="btn outline" aria-pressed="true">Draft documents</button>
            <button class="btn outline" aria-pressed="true">Available to add</button>
            <button class="btn outline" aria-pressed="true">Unsupported files</button>
            <span class="muted grow" style="text-align:right">All on · also applies to search results</span>
          </div>
          <div class="table-wrap"><table>
            <thead><tr>
              <th></th><th>Name</th><th>Library</th><th>Title</th><th>State</th><th>Released</th>
            </tr></thead>
            <tbody>
              <tr>
                <td></td><td><strong>${wireframeIcon("folder")} Recruiting ${badge("~1", "info")}</strong></td><td>—</td><td>Folder</td><td>—</td><td>—</td>
              </tr>
              <tr>
                <td></td><td><strong>${wireframeIcon("folder")} Templates ${badge("+1", "ok")}</strong></td><td>—</td><td>Folder</td><td>—</td><td>—</td>
              </tr>
              <tr>
                <td><span class="check">☐</span></td>
                <td>${wireframeIcon("file")} Handbook.docx</td>
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
          ${tablePagination({ ariaLabel: "Library rows per page", count: 8 })}
          <p class="hint"><strong>Name is the source file:</strong> it always shows the exact filesystem name, including the extension. Registered files show the independent DMS title and number under Document.</p>
        </section>
        <div role="separator" aria-label="Resize document details" aria-orientation="vertical" style="cursor:col-resize;background:var(--border);border-radius:999px" title="Drag or use Left/Right; Escape cancels"></div>
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
          <p class="hint">Batch add is available because every selected row is an in-root supported source draft, including Markdown. A mixed or unsupported selection has no incompatible action. The divider resizes this pane from 280–640 px for this session while preserving at least 360 px for the list; Escape cancels a drag.</p>
        </aside>
      </div>
      ${batchSelectionPane()}`,
  },
  {
    id: "CAP-0007",
    file: "CAP-0007-draft-pdf-export",
    title: "Source draft → PDF export",
    nav: "releases",
    subtitle: "Office via host Office (temp token fill); Markdown via CommonMark print shell + WebView PDF. Shared export chrome from .dms. Classified filename → temp PDF → validate → SHA-256 → atomic rename.",
    body: `
      <section class="card">
        <h3 class="card-title">Export pipeline</h3>
        <div class="pipeline">
          ${["Identify source format", "Build export chrome from .dms", "Office temp token fill or MD print shell", "Export to temp PDF", "Validate header", "SHA-256 digest", "Atomic rename"]
            .map((s) => `<div class="step active"><span class="dot"></span>${s}</div>`)
            .join("")}
        </div>
      </section>
      <section class="card">
        <h3 class="card-title">Markdown print shell (Option A)</h3>
        ${kv([
          ["Body", "CommonMark HTML; YAML front matter stripped"],
          ["Chrome source", "Release context only (not front matter / Office props)"],
          ["Footer captions", "Vertraulichkeitsstufe: <label> · Version: <major>.<minor>"],
          ["Assets", "Shipped shell.html + print.css + logo (Vorlage-derived)"],
        ])}
        <p class="muted">CAP-0002 marker checks still read the Markdown body on disk. Print-shell footers repeat chrome on the PDF and do not replace that gate.</p>
      </section>
      <section class="card">
        <h3 class="card-title">Fail-closed conditions</h3>
        <ul class="list danger-list">
          <li>Office missing or unlicensed for an Office draft → abort, no partial version</li>
          <li>Markdown render or print-shell failure → abort, no partial version</li>
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
          ["Export chrome version", "2.0"],
        ])}
        <p class="muted">A successful release record only exists when: export produced a valid, non-empty PDF, its SHA-256 was computed, and the atomic rename to the versioned path succeeded. Failure at any step removes the temp file when possible and never commits a release record.</p>
      </section>`,
  },
  {
    id: "CAP-0008",
    file: "CAP-0008-confidentiality-classification",
    title: "Confidentiality policies",
    nav: "config",
    configSection: "document-defaults",
    subtitle: "Set the root default, then add only the folder exceptions that need a different confidentiality type.",
    body: `
      ${defaultsFirstStyles()}
      <section class="config-summary">
        <div class="summary-copy"><strong>Workspace default ${badge("Internal", "info")}</strong><span>Applied from edit-root unless a nearer folder policy changes it.</span></div>
        <span class="badge muted">4 enabled types</span>
        <button class="btn outline">Manage confidentiality types…</button>
      </section>
      <div class="defaults-grid">
        <section class="card tree">
          <h3 class="card-title">Choose default or exception</h3>
          <ul class="tree-root">
            <li>
              <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">📂 edit-root <span class="badge info">Internal</span></span></div>
              <ul>
                <li>
                  <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">📂 policies</span></div>
                  <ul>
                    <li><div class="tree-node active"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 HR <span class="badge warn">Restricted</span></span></div></li>
                    <li><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 IT</span></div></li>
                  </ul>
                </li>
                <li><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 records <span class="badge danger">Confidential</span></span></div></li>
                <li><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 Archive (empty)</span></div></li>
              </ul>
            </li>
          </ul>
          <p class="hint">Select edit-root to change the workspace default. <code>.dms</code> is hidden; Library navigation does not set this selection.</p>
        </section>
        <section class="card defaults-editor">
          <div class="row between mb"><h3 class="card-title" style="margin:0">Default for policies/HR/</h3>${badge("direct exception", "warn")}</div>
          <div class="default-state"><strong>Parent default: Internal from edit-root</strong>Saving below changes this folder and inheriting descendants only; nearer policies and document overrides remain unchanged.</div>
          <div class="grid-2" style="margin-top:0.75rem">
            <div><div class="label" style="margin-bottom:0.35rem">Confidentiality type</div><select aria-label="Policy type" style="width:100%;height:2rem;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.5rem;background:var(--background);color:var(--foreground)"><option>Public</option><option>Internal</option><option selected>Restricted</option><option>Confidential</option></select></div>
            <div><div class="label" style="margin-bottom:0.35rem">After removal</div><div style="height:2rem;display:flex;align-items:center">Internal from edit-root</div></div>
          </div>
          <div class="row gap-2" style="margin-top:0.75rem"><button class="btn">Save folder exception</button><button class="btn danger">Remove exception</button></div>
          <h3 class="card-title" style="margin-top:1rem">Folder exceptions</h3>
          ${growingTable({
            headers: ["Path", "Type", "State"],
            rows: [
              ["policies/HR/", "Restricted", "direct"],
              ["records/", "Confidential", "direct"],
            ],
            filterLabel: "Path or type",
            filterAriaLabel: "Filter confidentiality folder exceptions",
            filterPlaceholder: "e.g. HR or Restricted",
            matchingLabel: "folder exceptions",
          })}
          <p class="hint">Remove restores the nearest remaining default. The edit-root policy is required and cannot be removed; review and release snapshots do not change.</p>
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
    configSection: "notifications",
    subtitle: "Major review requests and post-release minor-publication notices use SMTP or mailto:.",
    body: `
      <div class="grid-2">
        <section class="card">
          <div class="row gap-2 mb"><h3 class="card-title">SMTP relay</h3>${badge("credential configured", "ok")}</div>
          ${kv([
            ["Host", "smtp.videoclinic.de"],
            ["Port", "587 (STARTTLS)"],
            ["SMTP login user", "dms-relay@videoclinic.de"],
            ["From address", "&quot;Doc Mgmt&quot; &lt;dms@videoclinic.de&gt;"],
            ["Microsoft 365 app password", "*** · write-only · stored in OS credential store"],
            ["Test target", "dms@videoclinic.de (parsed From mailbox)"],
            ["Minor-publication recipient", "anna@videoclinic.de (effective approver snapshot)"],
          ])}
          <div class="row gap-2" style="margin-top:0.75rem"><button class="btn outline">Send test email to “Doc Mgmt” &lt;dms@videoclinic.de&gt;</button></div>
          <p class="hint">A blank password input retains the existing OS credential. The login user authenticates only; the formatted From mailbox supplies the message identity and fixed test recipient. Changing to <code>mailto:</code> removes the workspace-scoped credential. No password is returned or written to <code>.dms</code>.</p>
        </section>
        <section class="card">
          <div class="row gap-2 mb"><h3 class="card-title">Minor-publication notice</h3>${badge("released V1.4", "ok")}</div>
          ${kv([
            ["Transport", "mailto: via Microsoft Outlook (Windows)"],
            ["Recipient", "anna@videoclinic.de"],
            ["Subject", "[Internal] DMS minor version released — HR Data Privacy Policy — V1.4"],
            ["Body", "A new minor version of your assigned document has been released.<br/><br/>Title: HR Data Privacy Policy<br/>Document: policies/HR/Privacy-Policy.docx<br/>Released by: Lara Becker<br/>Released version: V1.4<br/>Confidentiality: Internal<br/><br/>Open document:<br/><code>dms://open?workspace=ws-9c3b7d1a&amp;document=doc-77a12bce</code>"],
          ])}
          <p class="hint">V1.0 and major candidates send a review request before entering <code>in_review</code>. Minor notices are sent only after committed release; delivery failure never reverses it.</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0011",
    file: "CAP-0011-approval-evidence",
    title: "Workflow chain & evidence",
    nav: "audit",
    activity: "audit-doc-77a12bce",
    subtitle: "Newest event first. Major approval evidence and direct minor-release publication notices are chained together.",
    actions: ["Verify workflow", "Export chain"],
    body: `
      <section class="card">
        <h3 class="card-title">Chain (newest first) — HR Data Privacy Policy</h3>
        <div class="stack">
          ${event("minor_publication_notified", "2025-08-02 11:30 UTC", "—", "Minor V1.4 publication notice delivered to Anna Berg after committed export.", "5b3a…ffe5", "5b3a…ffe4")}
          ${event("release", "2025-08-02 11:29 UTC", "Lukas Roth", "Direct minor release: atomic export committed V1.4; approval not required.", "5b3a…ffe4", "5b3a…ffe3")}
          ${event("review_decision_approved", "2025-08-01 09:42 UTC", "Anna Berg", "Approved major target V2.0. Decision comment optional and omitted.", "5b3a…ffe3", "5b3a…ffe2")}
          ${event("review_requested", "2025-08-01 09:14 UTC", "Lukas Roth", "Changelog: restructured control scope. Target: V2.0 (major version change).", "5b3a…ffe2", "5b3a…ffe1")}
          ${event("review_decision_rejected", "2025-07-29 09:42 UTC", "Anna Berg", "Why was approval not granted? Optional comment: clarify the retention exception in §3.2.", "5b3a…ffe1", "—")}
        </div>
        <div class="callout warn">Rejected and changes-requested major decisions prompt for a reason but allow no comment. Major review requests and direct minor publications remain chain evidence.</div>
        <div class="callout ok">${badge("chain valid", "ok")} Verify workflow recomputed each event hash from its canonical body.</div>
      </section>`,
  },
  {
    id: "CAP-0012",
    file: "CAP-0012-audit-export",
    title: "Audit export",
    nav: "audit",
    subtitle: "Operator-triggered PDF/CSV reports include major review attempts and direct minor publications.",
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
        ${growingTable({
          headers: ["Report", "Generated", "Filter", "SHA-256", "Verify", "Size"],
          rows: [
            ["Audit-2025-08.pdf", "2025-08-05 08:30 UTC", "All", "f1a0…d223", badge("valid", "ok"), "412 KB"],
            ["Audit-2025-07.pdf", "2025-08-01 08:15 UTC", "Confidential only", "b73e…9c44", badge("valid", "ok"), "1.2 MB"],
            ["Audit-2025-07.csv", "2025-08-01 08:15 UTC", "All", "1199…aa01", badge("valid", "ok"), "84 KB"],
            ["Audit-2025-06.pdf", "2025-07-01 08:00 UTC", "Approver: Anna Berg", "—", badge("missing file", "warn"), "—"],
          ],
          filterLabel: "Report history",
          filterAriaLabel: "Filter report history",
          filterPlaceholder: "e.g. Audit-2025 or missing",
          matchingLabel: "reports",
        })}
        <p class="hint">Reports never embed draft or PDF bytes — metadata, digests, and the event chain only. They include every major review attempt and each direct minor release with its approver-notification delivery attempt.</p>
      </section>`,
  },
  {
    id: "CAP-0013",
    file: "CAP-0013-library-maintenance",
    title: "Library maintenance",
    nav: "maintenance",
    subtitle: "Rename/move with preserved ID, missing handling, rescan for recovery or batch work, catalogues, withdraw. Microsoft Entra owns workflow people.",
    body: `
      <div class="grid-explorer">
        <aside class="card">
          <h3 class="card-title">Actions</h3>
          <div class="stack-btns">
            <button class="btn">Rename / move draft (in-root)</button>
            <button class="btn outline">Mark missing</button>
            <button class="btn outline">Rescan library</button>
            <button class="btn outline">Workflow identity source</button>
            <button class="btn outline">Confidentiality catalogue</button>
            <button class="btn outline">Document-type catalogue</button>
            <button class="btn danger">Withdraw release</button>
            <button class="btn danger">Reject / request changes</button>
          </div>
        </aside>
        <section class="card">
          <h3 class="card-title">Drafts requiring attention</h3>
          ${growingTable({
            headers: ["Title", "Old path", "Status", "Suggestion"],
            rows: [
              ["Acceptable Use", "policies/IT/AUP.docx", badge("renamed", "warn"), "Match: policies/IT/AUP-v2.docx"],
              ["Backup Config", "policies/IT/Backup.docx", badge("missing", "danger"), "No candidate — restore from backup"],
              ["Vendor Onboarding", "procedures/Onboarding.docx", badge("candidate", "info"), "Match by last digest"],
              ["Office lock ignored", "~$AUP.docx", badge("ignored", "muted"), "Lock/temp sidecar — never a candidate"],
            ],
            filterLabel: "Finding",
            filterAriaLabel: "Filter rescan findings",
            filterPlaceholder: "e.g. Backup or missing",
            matchingLabel: "findings",
          })}
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
          ${growingTable({
            headers: ["Backup", "Created", "Files", "Manifest SHA-256"],
            rows: [
              ["dms-backup-2025-08-05.zip", "2025-08-05 08:00 UTC", "1,284", "9f2c…b1e0"],
              ["dms-backup-2025-07-29.zip", "2025-07-29 08:00 UTC", "1,279", "3a91…77ee"],
              ["dms-backup-2025-07-22.zip", "2025-07-22 08:00 UTC", "1,272", "5b3a…ffe2"],
            ],
            filterLabel: "Backup",
            filterAriaLabel: "Filter backup archives",
            filterPlaceholder: "e.g. 2025-08 or 9f2c",
            matchingLabel: "backup archives",
          })}
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
    subtitle: "Source file facts come from the filesystem. Exhaustive DMS-managed control data scrolls in the right pane without moving Library or application navigation.",
    actions: [],
    body: `
      ${viewportScrollStyles("CAP-0015", `
        .app[data-cap="CAP-0015"] .content { overflow: hidden; }
        .app[data-cap="CAP-0015"] .document-control-panes { min-height: 0; flex: 1; align-items: stretch; }
        .app[data-cap="CAP-0015"] .document-control-panes > .card { min-height: 0; overflow-y: auto; overscroll-behavior: contain; scrollbar-gutter: stable; }
        .app[data-cap="CAP-0015"] .control-boundary-summary { display: grid; gap: 0.4rem; margin: 0.75rem 0; }
        .app[data-cap="CAP-0015"] .control-boundary-summary > div { padding: 0.45rem 0.55rem; border-left: 3px solid var(--border); background: var(--muted); font-size: 0.75rem; }
        .app[data-cap="CAP-0015"] .control-boundary-summary strong { display: block; color: var(--foreground); }
        .app[data-cap="CAP-0015"] .list-card table { table-layout: fixed; }
        .app[data-cap="CAP-0015"] .list-card th:nth-child(1) { width: 1.5rem; }
        .app[data-cap="CAP-0015"] .list-card th:nth-child(2) { width: 5.9rem; }
        .app[data-cap="CAP-0015"] .list-card th:nth-child(3) { width: 6.6rem; }
        .app[data-cap="CAP-0015"] .list-card th:nth-child(4) { width: 4.5rem; }
        .app[data-cap="CAP-0015"] .list-card th:nth-child(5) { width: 5.3rem; }
      `)}
      <div class="grid-explorer-detail document-control-panes" style="grid-template-columns:17.5rem minmax(22.5rem,1fr) 0.45rem 22rem">
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
            <span class="muted grow">Current-folder results — selection pane in focus</span>
          </div>
          ${tableFilter({
            label: "Search",
            ariaLabel: "Search current library folder",
            placeholder: "Name or Title",
            summary: "2 matching rows",
          })}
          <div class="table-wrap"><table>
            <thead><tr><th></th><th>Name</th><th>Title</th><th>State</th><th>Released</th></tr></thead>
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
          ${tablePagination({ ariaLabel: "Document rows per page", count: 2 })}
        </section>
        <div role="separator" aria-label="Resize document details" aria-orientation="vertical" style="cursor:col-resize;background:var(--border);border-radius:999px" title="Session-only width · 280–640 px"></div>
        ${documentControlDataSelectionPane()}
      </div>`,
  },
  {
    id: "CAP-0016",
    file: "CAP-0016-publish-tree-maintenance",
    title: "Publish tree maintenance",
    nav: "releases",
    subtitle: "Filter releases by DMS-managed Title, set rows per page, open recorded PDFs, verify-all, reveal in host file manager, archive orphans.",
    actions: ["Verify all releases", "Reveal publish folder", "Archive orphans"],
    body: `
      <section class="card">
        <h3 class="card-title">Publish-tree</h3>
        ${growingTable({
          headers: ["Title", "Version", "Publish path", "Released", "SHA-256", "State", "Verify", "Action"],
          rows: [
            ["HR Data Privacy Policy", "V2.0", "policies/HR/Handbook_V2.0_restricted.pdf", "2025-08-01 09:44", "9f2c…b1e0", badge("current", "ok"), badge("match", "ok"), "<button class=\"btn outline\">Open PDF</button>"],
            ["Acceptable Use", "V2.0", "policies/IT/AUP_V2.0_internal.pdf", "2025-07-29 14:12", "3a91…77ee", badge("current", "ok"), badge("match", "ok"), "<button class=\"btn outline\">Open PDF</button>"],
            ["Incident Response", "V3.1", "procedures/IRP_V3.1_restricted.pdf", "2025-06-30 11:20", "1199…aa01", badge("current", "ok"), badge("match", "ok"), "<button class=\"btn outline\">Open PDF</button>"],
            ["Vendor Onboarding", "V1.0", "procedures/Onboarding_V1.0_internal.pdf", "2024-11-04 09:00", "—", badge("orphaned", "warn"), badge("missing file", "danger"), "<button class=\"btn outline\" disabled>Open PDF</button>"],
            ["Backup Config", "V1.4", "policies/IT/Backup_V1.4_internal.pdf", "2025-05-12 16:45", "5b3a…ffe2", badge("withdrawn", "muted"), badge("match", "ok"), "<button class=\"btn outline\">Open PDF</button>"],
          ],
          filterLabel: "Title filter",
          filterAriaLabel: "Filter releases by title",
          filterPlaceholder: "e.g. Doc",
          matchingLabel: "releases",
          pageAriaLabel: "Release rows per page",
        })}
        <p class="hint">Release records are immutable. Correction = withdraw + Begin revision + new approval + new version.</p>
      </section>`,
  },
  {
    id: "CAP-0017",
    file: "CAP-0017-periodic-document-review",
    title: "Periodic review",
    nav: "audit",
    subtitle: "Review schedule is separate from profile and release evidence: stored effective date plus resolved interval, with explicit exemptions.",
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
              ["Due-date basis", "Current release effective date + resolved document interval"],
            ])}
          </section>
          <section class="card">
            <h3 class="card-title">Routing</h3>
            ${kv([
              ["Reviewer", "Effective approver · object ID 41c2… (CAP-0019)"],
              ["Per-document override", "Backup Config · 6 months"],
            ])}
            <p class="hint">Periodic review reuses the document's object-ID-anchored effective approver. A document with <code>&lt;owner&gt;</code> or <code>&lt;editor&gt;</code> cannot start review. Exemption requires a reason comment in the workflow chain.</p>
          </section>
        </div>
        <section class="card">
          <h3 class="card-title">Due &amp; overdue</h3>
          ${growingTable({
            headers: ["Title", "Current release", "Next due", "Status", "Action"],
            rows: [
              ["Acceptable Use", "V2.0", "2025-07-15", badge("overdue", "danger"), "Start review / Remind"],
              ["Backup Config", "V1.4", "2025-08-22", badge("due ≤30d", "warn"), "Start review / Remind"],
              ["Incident Response", "V3.1", "2025-08-30", badge("due ≤30d", "warn"), "Start review / Remind"],
              ["Vendor Onboarding", "V1.0", "2025-09-14", badge("due", "muted"), "Start review / Remind"],
              ["Code of Conduct", "V1.2", "2025-12-01", badge("ok", "ok"), "Start review / Remind"],
            ],
            filterLabel: "Title or status",
            filterAriaLabel: "Filter due and overdue documents",
            filterPlaceholder: "e.g. overdue or Acceptable",
            matchingLabel: "documents",
          })}
          <p class="hint">Each Next due value is calculated from the immutable current-release effective date and the mutable review schedule; month-end dates clamp to the last valid day.</p>
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
          <p><strong>Suggested target mode:</strong> minor version change.</p>
          <p><strong>Suggested changelog:</strong> Replaced retention table with 24-month rule; clarified §5.1 approver role.</p>
          <p class="hint">AI output is untrusted. Operator edits before acceptance. Workflow records <code>assistance_used: true</code>, provider <code>Claude Desktop</code>.</p>
        </section>
      </div>`,
  },
  {
    id: "CAP-0019",
    file: "CAP-0019-inherited-workflow-role-routing",
    title: "Microsoft Entra workflow roles",
    nav: "config",
    configSection: "workflow",
    subtitle: "Set editor and approver defaults at the root, then add only the folder exceptions that need different routing.",
    body: `
      ${defaultsFirstStyles()}
      <style>.app[data-cap="CAP-0019"] .defaults-grid { grid-template-columns: 28rem minmax(0, 1fr); }</style>
      <section class="config-summary">
        <div class="summary-copy"><strong>People source ${badge("connected", "ok")}</strong><span>VC DMS Workflow Users · 3 eligible direct members · refreshed just now</span></div>
        <button class="btn outline">Refresh people</button>
        <button class="btn outline">Manage identity source…</button>
      </section>
      <div class="defaults-grid">
        <section class="card tree">
          <h3 class="card-title">Choose default or exception</h3>
          <ul class="tree-root" role="tree" aria-label="Workflow folders">
            <li role="treeitem" aria-level="1" aria-expanded="true">
              <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">📂 edit-root <span class="badge info">Editor · Lukas</span> <span class="badge info">Approver · Anna</span></span></div>
              <ul role="group">
                <li role="treeitem" aria-level="2" aria-expanded="true">
                  <div class="tree-node"><span class="tree-twisty">▾</span><span class="tree-label">📂 policies</span></div>
                  <ul role="group">
                    <li role="treeitem" aria-level="3" aria-selected="true"><div class="tree-node active"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 HR</span></div></li>
                    <li role="treeitem" aria-level="3"><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 IT <span class="badge warn">Editor · Lara</span> <span class="badge warn">Approver · Anna</span></span></div></li>
                  </ul>
                </li>
                <li role="treeitem" aria-level="2"><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 records <span class="badge danger">Editor · unresolved</span> <span class="badge warn">Approver · Anna</span></span></div></li>
                <li role="treeitem" aria-level="2"><div class="tree-node"><span class="tree-twisty empty">▸</span><span class="tree-label">📁 Archive (empty)</span></div></li>
              </ul>
            </li>
          </ul>
          <p class="hint">Select edit-root to change both routing defaults. <code>.dms</code> is hidden; Library navigation does not set this selection.</p>
        </section>
        <section class="card">
          <div class="row between mb"><h3 class="card-title" style="margin:0">Inherited for policies/HR/</h3>${badge("inherited", "info")}</div>
          <div class="default-state"><strong>Effective roles: Lukas Roth / Anna Berg from edit-root</strong>Save below creates a direct assignment for this folder and inheriting descendants only. Each role remains independent; document overrides remain unchanged.</div>
          <div class="grid-2" style="margin-top:0.75rem">
            <div><div class="label" style="margin-bottom:0.35rem">Responsible editor</div><button class="btn outline" style="width:100%;text-align:left">Lukas Roth · Change…</button></div>
            <div><div class="label" style="margin-bottom:0.35rem">Approver</div><button class="btn outline" style="width:100%;text-align:left">Anna Berg · Change…</button></div>
          </div>
          <div class="row gap-2" style="margin-top:0.75rem"><button class="btn">Save folder exception</button></div>
          <div class="callout warn" style="margin-top:0.75rem">An unresolved role blocks a new review. DMS Desktop never chooses a replacement; an operator reroutes the policy from the current Entra group.</div>
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
  {
    id: "CAP-0021",
    file: "CAP-0021-microsoft-entra-workflow-identity",
    title: "Microsoft Entra workflow identity",
    nav: "config",
    configSection: "workflow",
    configSecondary: "Identity source",
    subtitle: "Configure the Entra public client and tenant once for this OS user; each library retains only its bound group and display cache.",
    body: `${defaultsFirstStyles()}
      <section class="card">
        <div class="row between"><h3 class="card-title" style="margin:0">Current Microsoft Entra identity source</h3>${badge("configured", "ok")}</div>
        ${kv([
          ["Public client ID", "3e7f6750-4052-4db4-8638-234f9a85c2a1"],
          ["Tenant ID", "8d29cb1d-1d51-43ad-a225-3721792d0bf3"],
          ["Library group", "DMS Workflow Users"],
          ["Group ID", '<button class="btn outline" aria-label="Open Microsoft 365 group page for Group ID 9c14e7bf-87a4-409f-83ba-f8761b72bf72"><code>9c14e7bf-87a4-409f-83ba-f8761b72bf72</code> · Open group page</button>'],
          ["Last refresh", "2026-08-14 10:42 UTC"],
        ])}
        <p class="hint">The effective app-global IDs are environment-managed and read only. The library retains only its group binding and display cache; the Group ID control opens Microsoft My Account in the host browser.</p>
      </section>
      <div class="grid-2">
        <section class="card">
          <div class="row between"><h3 class="card-title" style="margin:0">Preview identity source</h3>${badge("first setup", "info")}</div>
          ${kv([
            ["Tenant", "Example Healthcare GmbH"],
            ["Group", "DMS Workflow Users · 9c14…bf72"],
            ["Eligible people", "3 direct enabled users"],
          ])}
          <fieldset style="margin:0.8rem 0;border:1px solid var(--border);border-radius:calc(var(--radius) - 2px);padding:0.75rem">
            <legend style="padding:0 0.3rem;font-weight:600">Initial edit-root workflow roles</legend>
            <p class="hint" style="margin-top:0">Both defaults are required and are saved atomically with this identity source.</p>
            <div class="grid-2">
              <label><span class="label">Editor</span><select aria-label="Initial editor" style="width:100%;height:2rem;margin-top:0.35rem;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.5rem;background:var(--background);color:var(--foreground)"><option selected>Lukas Roth — lukas@example.test</option><option>Anna Berg — anna@example.test</option><option>Mira Klein — mira@example.test</option></select></label>
              <label><span class="label">Approver</span><select aria-label="Initial approver" style="width:100%;height:2rem;margin-top:0.35rem;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.5rem;background:var(--background);color:var(--foreground)"><option>Lukas Roth — lukas@example.test</option><option selected>Anna Berg — anna@example.test</option><option>Mira Klein — mira@example.test</option></select></label>
            </div>
          </fieldset>
          <label class="row gap-2" style="align-items:flex-start"><input type="checkbox" checked/> <span>I confirm this group is the workspace’s people source.</span></label>
          <div class="row gap-2" style="margin-top:0.75rem"><button class="btn">Apply identity source</button><button class="btn outline">Cancel preview</button></div>
          <p class="hint">No intermediate binding-only state is saved. If persistence fails, this preview remains available for retry.</p>
        </section>
        <section class="card">
          <h3 class="card-title">Eligible people — read only</h3>
          ${growingTable({
            headers: ["Person", "Email", "Object ID", "State"],
            rows: [
              ["Lukas Roth", "lukas@example.test", "a714…51bf", badge("active", "ok")],
              ["Anna Berg", "anna@example.test", "b023…882a", badge("active", "ok")],
              ["Mira Klein", "mira@example.test", "c144…0d91", badge("active", "ok")],
            ],
            filterLabel: "Person or email",
            filterAriaLabel: "Filter eligible people",
            filterPlaceholder: "e.g. Anna or example.test",
            matchingLabel: "eligible people",
          })}
          <p class="hint">Use this read-only list only to select editor/approver routing. Add, remove, disable, and profile changes happen in Microsoft Entra.</p>
        </section>
      </div>
      <section class="config-summary"><div class="summary-copy"><strong>Authority boundary</strong><span>DMS does not manage Entra users or file permissions and never sends document content to Microsoft Graph. Replacing this source preserves role references as unresolved; it does not map them to the new group.</span></div></section>`,
  },
];

function badge(text, kind = "muted") {
  return `<span class="badge ${kind}">${text}</span>`;
}

function folderCounter(text, label) {
  return `<span aria-label="${label}" title="${label}" style="display:inline-flex;min-width:1rem;justify-content:center;padding:0.05rem 0.14rem;border-radius:999px;background:var(--muted);font-size:0.58rem;font-weight:700">${text}</span>`;
}

function wireframeIcon(name) {
  const paths = {
    folder: '<path d="M3 6h7l2 2h9v11H3z"/><path d="M3 6V4h7l2 2"/>',
    file: '<path d="M6 3h8l4 4v14H6z"/><path d="M14 3v5h4"/>',
    back: '<path d="m15 18-6-6 6-6"/>',
    forward: '<path d="m9 18 6-6-6-6"/>',
    up: '<path d="m6 15 6-6 6 6"/>',
    refresh: '<path d="M20 11a8 8 0 1 0-2 5.3"/><path d="M20 4v7h-7"/>',
  };
  return `<svg aria-hidden="true" viewBox="0 0 24 24" style="width:1rem;height:1rem;display:inline-block;vertical-align:-0.18rem;fill:none;stroke:currentColor;stroke-width:1.8;stroke-linecap:round;stroke-linejoin:round">${paths[name]}</svg>`;
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
function tableFilter({ label, ariaLabel, placeholder, summary }) {
  return `<div class="row between mb" style="flex-wrap:wrap;gap:0.75rem">
    <label class="muted" style="display:flex;align-items:center;gap:0.45rem">${label} <input aria-label="${ariaLabel}" placeholder="${placeholder}" style="height:2rem;width:15rem;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.65rem;font:inherit;color:var(--foreground);background:var(--background)"/></label>
    <span class="muted">${summary}</span>
  </div>`;
}
function tablePagination({ ariaLabel, count }) {
  return `<div class="row between" style="margin-top:0.75rem;flex-wrap:wrap;gap:0.75rem">
    <label class="muted" style="display:flex;align-items:center;gap:0.45rem">Rows per page <select aria-label="${ariaLabel}" style="height:2rem;border:1px solid var(--input);border-radius:calc(var(--radius) - 2px);padding:0 0.45rem;font:inherit;color:var(--foreground);background:var(--background)"><option>10</option><option selected>25</option><option>50</option><option>100</option></select></label>
    <div class="row gap-2"><span class="muted">1–${count} of ${count}</span><button class="btn outline" disabled>Previous</button><button class="btn outline" disabled>Next</button></div>
  </div>`;
}
function growingTable({
  headers,
  rows,
  filterLabel,
  filterAriaLabel,
  filterPlaceholder,
  matchingLabel,
  pageAriaLabel = "Table rows per page",
}) {
  return `${tableFilter({
    label: filterLabel,
    ariaLabel: filterAriaLabel,
    placeholder: filterPlaceholder,
    summary: `${rows.length} matching ${matchingLabel}`,
  })}${table(headers, rows)}${tablePagination({ ariaLabel: pageAriaLabel, count: rows.length })}`;
}
function note(head, body) {
  return `<article class="note"><header>${head}</header><p>${body}</p></article>`;
}
function layer(name, desc) {
  return `<div class="layer"><strong>${name}</strong><p class="muted">${desc}</p></div>`;
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
const CONFIG_ROUTES = [
  { id: "workspace", label: "Workspace", detail: "Roots and local metadata" },
  { id: "document-defaults", label: "Document defaults", detail: "Classification and catalogues" },
  { id: "workflow", label: "Workflow", detail: "People and role routing" },
  { id: "notifications", label: "Notifications", detail: "Review and release email" },
];
function configurationNavigation(cap) {
  if (!cap.configSection) return "";
  const current = CONFIG_ROUTES.find((route) => route.id === cap.configSection);
  const secondary = cap.configSecondary
    ? `<div class="config-secondary"><span>${current.label}</span><span aria-hidden="true">›</span><strong>${cap.configSecondary}</strong><button class="btn outline config-back">← Back to ${current.label}</button></div>`
    : `<span class="badge info">Current: ${current.label}</span>`;
  const routes = CONFIG_ROUTES.map((route) => {
    const active = route.id === cap.configSection;
    return `<a class="config-tab${active ? " active" : ""}" href="#"${active ? ' aria-current="page"' : ""}><strong>${route.label}</strong><span>${route.detail}</span></a>`;
  }).join("");
  return `<section class="configuration-nav" aria-label="Configuration navigation">
    <div class="config-nav-head"><div><h3>Configuration</h3><p>Set up the workspace once, then choose the task that matches the setting you need.</p></div>${secondary}</div>
    <nav class="config-tabs" aria-label="Configuration sections">${routes}</nav>
  </section>`;
}
function documentControlDataSelectionPane() {
  // CAP-0015 owns this shared selection-pane content; CAP-0006 owns its placement.
  return `<aside class="card detail-pane" aria-label="Scrollable document selection details">
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
    <div class="control-boundary-summary" aria-label="Document control data boundaries">
      <div><strong>Document profile · mutable</strong>Owner Lukas Roth · title HR Data Privacy Policy</div>
      <div><strong>Current release · immutable snapshot</strong>V1.3 · effective 2025-08-01 · captured owner 8a1f…</div>
      <div><strong>Review schedule · mutable</strong>12 months · next due 2026-08-01</div>
    </div>
    <details class="selection-section" open>
      <summary>Document profile <span>Mutable · managed in DMS Desktop</span></summary>
      <div class="selection-section-body">${kv([
        ["Title", "HR Data Privacy Policy"],
        ["Document number", "DOC-014 (unique in workspace)"],
        ["Document type", "policy"],
        ["Owner", "Lukas Roth · lukas@vc.de · object ID 8a1f…"],
        ["Eligible-person assignment", "Select submits object ID only; name/email are refreshable display data"],
        ["Legacy owner label", "— <span class=\"muted\">(pre-v12 text would be shown unresolved here)</span>"],
        ["Effective editor", "Lukas Roth · object ID 8a1f…"],
        ["Effective approver", "Anna Berg · object ID 41c2…"],
        ["Confidentiality", "Internal (inherited from policies/HR)"],
      ])}<p class="hint">Stored under <code>.dms</code>. Effective date is not editable profile data. Renaming the source or refreshing display names does not change identity authority.</p></div>
    </details>
    <details class="selection-section" open>
      <summary>Current release <span>Immutable snapshot</span></summary>
      <div class="selection-section-body">${kv([
        ["Released", "V1.3 (current)"],
        ["Effective date", "2025-08-01"],
        ["Captured title", "HR Data Privacy Policy"],
        ["Captured owner", "Lukas Roth · object ID 8a1f…"],
        ["Draft", badge("newer than last release", "warn")],
      ])}<button class="btn outline">Open latest released PDF</button><p class="hint">Pre-v12 missing profile or effective-date evidence is labelled <strong>unrecorded</strong>; the mutable profile is never substituted.</p></div>
    </details>
    <details class="selection-section" open>
      <summary>Review schedule <span>Mutable</span></summary>
      <div class="selection-section-body">${kv([
        ["Resolved interval", "12 months (workspace default)"],
        ["Next review due", "2026-08-01"],
        ["Exemption", "None"],
      ])}<p class="hint">Due date = current release effective date + resolved interval, clamped to the last valid calendar day.</p></div>
    </details>
    <details class="selection-section" open>
      <summary>Actions <span>17 available</span></summary>
      <div class="selection-section-body stack-btns">
        <button class="btn outline">Open draft</button>
        <button class="btn outline">Edit document control data</button>
        <button class="btn outline">Override confidentiality…</button>
        <button class="btn outline">Submit release candidate</button>
        <button class="btn outline">Apply real Owner / Editor with successful release</button>
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
      <summary>Submit release candidate <span>Required inputs</span></summary>
      <div class="selection-section-body stack">
        <label class="label">Effective date * <input type="date" value="2025-08-15" required></label>
        <label class="label">Target * <select><option value="next_minor" selected>Next minor · V1.4</option><option value="next_major">Next major · V2.0</option><option value="manual">Manual</option></select></label>
        <label class="label">Requesting editor * <select><option value="8a1f">Lukas Roth · lukas@vc.de</option></select></label>
        <p class="hint">For placeholder documents, select real Owner and Editor to stage them. They apply atomically only after successful release; failed export leaves the placeholders unchanged.</p>
        <button class="btn">Submit candidate</button>
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
        ${ver("V1.3", "effective 2025-08-01 · released 2025-08-01 09:44 UTC · owner 8a1f…", "Substantive / major", "current", "ok")}
        ${ver("V1.2", "effective 2025-07-12 · released 2025-07-12 12:01 UTC", "Cosmetic / minor", "superseded", "muted")}
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
function defaultsFirstStyles() {
  return `<style>
    .config-summary { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 0.75rem; padding: 0.75rem 0.9rem; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
    .config-summary .summary-copy { min-width: 14rem; flex: 1; }
    .config-summary .summary-copy strong { display: block; font-size: 0.85rem; }
    .config-summary .summary-copy span { color: var(--muted-foreground); font-size: 0.78rem; }
    .defaults-grid { display: grid; grid-template-columns: 13rem minmax(0, 1fr); gap: 1rem; align-items: start; }
    .defaults-editor { min-width: 0; }
    .default-state { padding: 0.65rem 0.75rem; border-radius: calc(var(--radius) - 2px); background: var(--muted); font-size: 0.8rem; }
    .default-state strong { display: block; margin-bottom: 0.2rem; font-size: 0.82rem; }
  </style>`;
}

function viewportScrollStyles(capId, extra = "") {
  const extraRules = extra.trim().split("\n").map((line) => line.trim()).join("\n    ");
  return `<style>
    html, body { height: 100%; }
    body { overflow: hidden; }
    .app[data-cap="${capId}"] { height: 100vh; min-height: 0; overflow: hidden; }
    .app[data-cap="${capId}"] .sidebar { min-height: 0; overflow-y: auto; overscroll-behavior: contain; }
    .app[data-cap="${capId}"] .main { min-height: 0; overflow: hidden; }
    .app[data-cap="${capId}"] .header { flex: 0 0 auto; }
    .app[data-cap="${capId}"] .content { min-height: 0; overflow: auto; overscroll-behavior: contain; }
${extraRules ? `    ${extraRules}` : ""}
  </style>`;
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
.subtitle { margin: 0; color: var(--muted-foreground); font-size: 0.875rem; max-width: 60rem; }${cap.configSection ? `
.configuration-nav { display: flex; flex-direction: column; gap: 0.75rem; padding: 0.85rem 1rem; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); }
.config-nav-head { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; }
.config-nav-head > div { min-width: 0; }
.config-nav-head h3 { margin: 0; font-size: 0.95rem; }
.config-nav-head p { margin: 0.2rem 0 0; color: var(--muted-foreground); font-size: 0.78rem; }
.config-tabs { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 0.45rem; }
.config-tab { display: flex; flex-direction: column; gap: 0.15rem; min-width: 0; padding: 0.55rem 0.65rem; border: 1px solid var(--border); border-radius: calc(var(--radius) - 2px); background: var(--background); }
.config-tab strong { font-size: 0.8rem; }
.config-tab span { color: var(--muted-foreground); font-size: 0.68rem; line-height: 1.3; }
.config-tab.active { border-color: color-mix(in oklch, var(--info) 38%, var(--border)); background: color-mix(in oklch, var(--info) 10%, white); color: var(--info); }
.config-tab.active span { color: color-mix(in oklch, var(--info) 75%, var(--muted-foreground)); }
.config-secondary { display: flex; align-items: center; gap: 0.4rem; flex-wrap: wrap; justify-content: flex-end; color: var(--muted-foreground); font-size: 0.78rem; }
.config-secondary strong { color: var(--foreground); }
.config-back { height: 1.75rem; margin-left: 0.25rem; font-size: 0.72rem; }` : ""}
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
.composer { margin: 0 0 0.9rem; border: 1px dashed var(--border); border-radius: var(--radius); padding: 0.85rem; display: flex; flex-direction: column; gap: 0.5rem; }
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
        ${badge(`Status: ${cap.status ?? "not implemented"}`, "muted")}
        <span class="wire-meta">Wireframe · shadcn-admin 2.2.0 visual base</span>
      </div>
      <p class="subtitle">${cap.subtitle}</p>
      ${cap.configSection ? `${configurationNavigation(cap)}
      ` : ""}${cap.body}
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
    configSection: r.configSection || null,
  })),
};
fs.writeFileSync(path.join(__dirname, "manifest.json"), JSON.stringify(manifest, null, 2));
console.log("index + manifest written");
console.log(JSON.stringify(manifest.screens.map((s) => s.id)));
