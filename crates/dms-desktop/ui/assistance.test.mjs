import test from "node:test";
import assert from "node:assert/strict";

import {
  assistanceDocumentState,
  assistanceMarkup,
  assistancePolicyMarkup,
  createAssistanceState,
  updateAssistanceState,
} from "./assistance.mjs";

const activity = {
  label: "Evaluate changes · Handbook",
  document_id: "doc-1",
};

const availability = {
  available: true,
  reason: "Available",
  privacy_notice: "Processing may send the displayed payload to Anthropic.",
};

const payload = {
  release_version: "V1.0",
  current_source_digest: "source",
  released_pdf_digest: "pdf",
  payload_digest: "payload",
  prompt: "Review <only> this exact payload",
};

test("assistance state is isolated by stable document ID", () => {
  const states = updateAssistanceState({}, "doc-1", { response: "Suggestion" });
  assert.equal(assistanceDocumentState(states, "doc-1").response, "Suggestion");
  assert.deepEqual(assistanceDocumentState(states, "doc-2"), createAssistanceState());
});

test("unavailable assistance explains policy or installation without a fallback", () => {
  const markup = assistanceMarkup(activity, {
    ...createAssistanceState(),
    availability: { ...availability, available: false, reason: "Claude Desktop is not installed" },
  });
  assert.match(markup, /Unavailable/);
  assert.match(markup, /not installed/);
  assert.doesNotMatch(markup, /API key|cloud fallback/);
});

test("payload requires explicit consent and escapes prompt content", () => {
  const markup = assistanceMarkup(activity, {
    ...createAssistanceState(),
    availability,
    payload,
  });
  assert.match(markup, /data-assistance-consent/);
  assert.match(markup, /data-assistance-handoff/);
  assert.match(markup, /Review &lt;only&gt; this exact payload/);
  assert.match(markup, /may send the displayed payload to Anthropic/);
});

test("accepted response remains an editable activity draft with no lifecycle action", () => {
  const markup = assistanceMarkup(activity, {
    ...createAssistanceState(),
    availability,
    payload,
    response: "Suggested changelog",
    accepted_changelog: "Editor revised changelog",
  });
  assert.match(markup, /data-assistance-changelog/);
  assert.match(markup, /Editor revised changelog/);
  assert.match(markup, /cannot select a version, approve, release, or write workspace lifecycle state/);
  assert.doesNotMatch(markup, /data-assistance-(approve|release|save)/);
});

test("workspace policy is explicit, default-safe, and uses confidentiality IDs", () => {
  const markup = assistancePolicyMarkup({
    value: {
      enabled: false,
      allowed_confidentiality_type_ids: ["internal", "public"],
      max_payload_chars: 24000,
    },
    error: "",
  });
  assert.match(markup, /Disabled by default/);
  assert.match(markup, /internal, public/);
  assert.match(markup, /Maximum payload characters/);
  assert.doesNotMatch(markup, /API key|password/);
});
