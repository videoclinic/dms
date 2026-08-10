function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function createAssistanceState() {
  return {
    availability: null,
    payload: null,
    response: "",
    accepted_changelog: "",
    loading: false,
    error: "",
    launched: false,
  };
}

export function assistanceDocumentState(states, documentId) {
  return states[documentId] ?? createAssistanceState();
}

export function updateAssistanceState(states, documentId, patch) {
  return {
    ...states,
    [documentId]: { ...assistanceDocumentState(states, documentId), ...patch },
  };
}

function payloadMarkup(payload, state) {
  if (!payload) {
    return '<button class="button" type="button" data-assistance-preview>Preview exact payload</button>';
  }
  return `<section class="assistance-preview"><h3>Exact payload preview</h3><dl class="selection-details"><dt>Release</dt><dd>${escapeHtml(payload.release_version)}</dd><dt>Source digest</dt><dd><code>${escapeHtml(payload.current_source_digest)}</code></dd><dt>Released PDF digest</dt><dd><code>${escapeHtml(payload.released_pdf_digest)}</code></dd><dt>Payload digest</dt><dd><code>${escapeHtml(payload.payload_digest)}</code></dd></dl><label>Prompt copied to Claude Desktop<textarea readonly rows="16" data-assistance-prompt>${escapeHtml(payload.prompt)}</textarea></label><label class="consent-row"><input type="checkbox" data-assistance-consent> I reviewed this exact payload and consent to sending it for Anthropic model processing.</label><button class="button" type="button" data-assistance-handoff>Copy prompt and open Claude Desktop</button>${state.launched ? '<p class="status success">Prompt copied. Paste it into Claude Desktop, then paste the response below.</p>' : ""}</section>`;
}

export function assistanceMarkup(activity, state) {
  const availability = state.availability;
  if (state.loading && !availability) {
    return '<section class="card"><h2>Claude Desktop assistance</h2><p>Checking workspace policy and local installation…</p></section>';
  }
  if (!availability) {
    return `<section class="card"><h2>Claude Desktop assistance</h2><p class="status" role="alert">${escapeHtml(state.error || "Availability has not been checked.")}</p></section>`;
  }
  if (!availability.available) {
    return `<section class="card assistance-workspace"><span class="badge muted">Unavailable</span><h2>${escapeHtml(activity.label)}</h2><p>${escapeHtml(availability.reason)}</p><p class="privacy-notice">${escapeHtml(availability.privacy_notice)}</p>${state.error ? `<p class="status" role="alert">${escapeHtml(state.error)}</p>` : ""}</section>`;
  }
  return `<section class="card assistance-workspace"><span class="badge">Advisory only</span><h2>${escapeHtml(activity.label)}</h2><p class="privacy-notice">${escapeHtml(availability.privacy_notice)}</p>${state.error ? `<p class="status" role="alert">${escapeHtml(state.error)}</p>` : ""}${payloadMarkup(state.payload, state)}<section class="assistance-response"><h3>Review Claude's response</h3><label>Untrusted response<textarea rows="10" data-assistance-response placeholder="Paste the response from Claude Desktop">${escapeHtml(state.response)}</textarea></label><button class="button secondary" type="button" data-assistance-accept ${state.response.trim() ? "" : "disabled"}>Copy response into editable changelog draft</button><label>Editable changelog draft<textarea rows="6" data-assistance-changelog placeholder="Nothing is written to lifecycle state">${escapeHtml(state.accepted_changelog)}</textarea></label><p class="subtle">This draft remains local to this open activity. It cannot select a version, approve, release, or write workspace lifecycle state.</p></section></section>`;
}

export function assistancePolicyMarkup(state) {
  const policy = state.value;
  if (!policy) {
    return `<section class="card"><h2>Claude Desktop assistance policy</h2><p>${escapeHtml(state.error || "Loading policy…")}</p></section>`;
  }
  const allowed = [...(policy.allowed_confidentiality_type_ids ?? [])].join(", ");
  return `<section class="card assistance-workspace"><span class="badge">Disabled by default</span><h2>Claude Desktop assistance policy</h2><p>Allow only confidentiality type IDs approved for operator-mediated Anthropic processing. This does not configure credentials or call an API.</p>${state.error ? `<p class="status" role="alert">${escapeHtml(state.error)}</p>` : ""}<form id="claude-policy-form" class="assistance-response"><label class="consent-row"><input type="checkbox" name="enabled" ${policy.enabled ? "checked" : ""}> Enable optional Claude Desktop handoff for this workspace</label><label>Permitted confidentiality type IDs<input name="allowedIds" value="${escapeHtml(allowed)}" placeholder="public, internal" autocomplete="off"></label><label>Maximum payload characters<input name="maxPayloadChars" type="number" min="1" required value="${escapeHtml(policy.max_payload_chars)}"></label><button class="button" type="submit">Save assistance policy</button></form></section>`;
}
