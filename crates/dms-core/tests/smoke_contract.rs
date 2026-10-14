//! Public converter/assembler behavior used by the independent smoke verifier.
//! Synthetic temporary inputs only; no workspace or installed Word operations.

use std::{
    cell::Cell,
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use dms_core::{
    assemble_markdown_docx_with_options, ConversionExporter, ConversionOptions, ConversionPlan,
    ConversionStatus, ExportChrome, MarkdownTemplateAsset, SCHEMA_VERSION,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use zip::ZipArchive;

const TEMPLATE: &[u8] = include_bytes!("fixtures/markdown-template.docx");
const WIDE_PNG: &[u8] = include_bytes!("fixtures/markdown-wide.png");

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn zip_parts(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut archive = ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let mut parts = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.is_dir() {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        parts.insert(entry.name().to_owned(), bytes);
    }
    parts
}

/// Concatenated visible `w:t` text of an assembled DOCX body, in document order.
fn visible_text(path: &Path) -> String {
    let document = String::from_utf8(zip_parts(path)["word/document.xml"].clone()).unwrap();
    let mut reader = quick_xml::Reader::from_str(&document);
    let mut in_text = 0usize;
    let mut text = String::new();
    loop {
        match reader.read_event().unwrap() {
            quick_xml::events::Event::Start(element) if element.name().as_ref() == b"w:t" => {
                in_text += 1;
            }
            quick_xml::events::Event::End(element) if element.name().as_ref() == b"w:t" => {
                in_text -= 1;
            }
            quick_xml::events::Event::Text(value) if in_text > 0 => {
                text.push_str(&value.unescape().unwrap());
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    text
}

/// Host-adapter double that fails the test if a dry run ever reaches Word.
#[derive(Default)]
struct NeverCalled {
    preflight: Cell<usize>,
    export: Cell<usize>,
}
impl ConversionExporter for NeverCalled {
    fn preflight(&self) -> Result<(), String> {
        self.preflight.set(self.preflight.get() + 1);
        Err("dry run must not call the host adapter".into())
    }
    fn export(
        &mut self,
        _docx: &Path,
        _pdf: &Path,
        _chrome: &ExportChrome,
    ) -> Result<String, String> {
        self.export.set(self.export.get() + 1);
        Err("dry run must not call the host adapter".into())
    }
}

struct ConversionFixture {
    _temp: tempfile::TempDir,
    options: ConversionOptions,
}

impl ConversionFixture {
    fn new(template_id: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        fs::create_dir_all(root.join("edit/.dms")).unwrap();
        fs::create_dir(root.join("publish")).unwrap();
        fs::create_dir(root.join("input")).unwrap();
        fs::write(root.join("edit/Markdown-template.docx"), TEMPLATE).unwrap();
        // Deliberately not a deserializable Workspace: the projection is read-only
        // and never opens the workspace or the publish tree.
        let config = json!({
            "schema_version": SCHEMA_VERSION,
            "edit_root": root.join("edit"),
            "publish_root": root.join("publish"),
            "markdown_template": {
                "id": template_id,
                "relative_path": "Markdown-template.docx",
                "sha256": digest(TEMPLATE),
                "contract_version": 1
            },
            "confidentiality_types": {
                "internal": {"id": "internal", "label": "Internal label", "enabled": true}
            },
            "documents": "must not open"
        });
        let options = ConversionOptions {
            workspace_config: root.join("edit/.dms/workspace.json"),
            input_root: root.join("input"),
            output_root: root.join("output"),
            recursive: false,
            dry_run: true,
            without_toc: false,
        };
        fs::write(
            &options.workspace_config,
            serde_json::to_vec_pretty(&config).unwrap(),
        )
        .unwrap();
        Self {
            _temp: temp,
            options,
        }
    }

    fn write_input(&self, name: &str, markdown: &str) {
        fs::write(self.options.input_root.join(name), markdown).unwrap();
    }
}

#[test]
fn assembler_enabled_extensions_and_smart_punctuation_render_actual_text() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    let output = temp.path().join("output.docx");
    fs::write(&template, TEMPLATE).unwrap();

    // Pin visible text independently of the Python verifier's parser.
    let markdown = "---\ntitle: Extensions\nversion: 1.0\nconfidentiality: internal\n---\n\
# Documents {#docs .wide}\n\n\
Speed -- and power --- and range... says \"fast\" and don't stop.\n\n\
- [X] completed task\n- [ ] pending task\n\n\
Struck ~~struck out~~ text and [the portal](https://example.test) link.\n\n\
Inline math $x^2$ and a footnote [^n].\n\n\
[^n]: Footnote body.\n";
    fs::write(temp.path().join("source.md"), markdown).unwrap();

    assemble_markdown_docx_with_options(&template, markdown, &output, None, false).unwrap();
    let text = visible_text(&output);
    assert!(text.contains("Speed – and power — and range… says “fast” and don’t stop."));
    assert!(text.contains("Struck struck out text and the portal (https://example.test) link."));
    // Smart punctuation: ASCII `--`, `---`, `...` and straight quotes are
    // replaced during parsing, so the literal ASCII forms must be gone.
    assert!(
        !text.contains("--"),
        "en/em dash replacement inactive: {text:?}"
    );
    assert!(
        !text.contains("..."),
        "ellipsis replacement inactive: {text:?}"
    );
    assert!(
        !text.contains('"'),
        "curly quote replacement inactive: {text:?}"
    );
    assert!(text.contains('\u{2026}'), "missing ellipsis: {text:?}"); // …
    assert!(
        text.contains('\u{201c}') && text.contains('\u{201d}'),
        "missing curly quotes: {text:?}"
    );
    assert!(
        text.contains('\u{2019}'),
        "missing curly apostrophe: {text:?}"
    ); // don’t
    assert!(
        text.contains('\u{2013}') && text.contains('\u{2014}'),
        "missing smart dash: {text:?}"
    );

    // Heading attributes are parsed, not leaked as text.
    assert!(text.contains("Documents"), "heading text lost: {text:?}");
    assert!(
        !text.contains("#docs") && !text.contains(".wide"),
        "heading attributes leaked: {text:?}"
    );

    // Task lists, strikethrough, links, math and footnotes keep visible text.
    assert!(text.contains("[x] completed task"), "{text:?}");
    assert!(text.contains("[ ] pending task"), "{text:?}");
    assert!(
        text.contains("struck out"),
        "strikethrough text lost: {text:?}"
    );
    assert!(
        text.contains("https://example.test"),
        "link target lost: {text:?}"
    );
    assert!(text.contains("$x^2$"), "math lost: {text:?}");
    assert!(text.contains("[^n]"), "footnote ref lost: {text:?}");
    assert!(
        text.contains("Footnote body."),
        "footnote body lost: {text:?}"
    );
}

#[test]
fn conversion_document_number_absent_or_blank_is_not_a_rejection() {
    let fixture = ConversionFixture::new("00000000-0000-0000-0000-000000000001");
    fixture.write_input(
        "a-missing.md",
        "---\ntitle: Alpha\nversion: 1.0\nconfidentiality: internal\n---\n# Alpha\n",
    );
    // Quoted so the scalar survives frontmatter parsing as whitespace only.
    fixture.write_input(
        "b-blank.md",
        "---\ntitle: Bravo\ndocument_number: \"   \"\nversion: 1.0\nconfidentiality: internal\n---\n# Bravo\n",
    );
    fixture.write_input(
        "c-present.md",
        "---\ntitle: Charlie\ndocument_number: DOC-7\nversion: 1.0\nconfidentiality: internal\n---\n# Charlie\n",
    );

    let mut adapter = NeverCalled::default();
    let manifest = ConversionPlan::prepare(&fixture.options)
        .unwrap()
        .run(&mut adapter)
        .unwrap();

    assert!(
        !manifest.has_failures(),
        "missing/blank number was rejected: {:?}",
        manifest
            .results
            .iter()
            .map(|r| r.error.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(manifest.results.len(), 3);
    for result in &manifest.results {
        assert!(
            matches!(result.status, ConversionStatus::Validated),
            "{:?}",
            result.status
        );
    }
    // a-missing.md and b-blank.md: field is absent (None); c-present.md: verbatim.
    assert_eq!(manifest.results[0].input, PathBuf::from("a-missing.md"));
    assert!(manifest.results[0]
        .metadata
        .as_ref()
        .unwrap()
        .document_number
        .is_none());
    assert_eq!(manifest.results[1].input, PathBuf::from("b-blank.md"));
    assert!(
        manifest.results[1]
            .metadata
            .as_ref()
            .unwrap()
            .document_number
            .is_none(),
        "whitespace-only document_number was not filtered to absent"
    );
    assert_eq!(manifest.results[2].input, PathBuf::from("c-present.md"));
    assert_eq!(
        manifest.results[2]
            .metadata
            .as_ref()
            .unwrap()
            .document_number
            .as_deref(),
        Some("DOC-7")
    );
    // A dry run never touches the host adapter.
    assert_eq!(adapter.preflight.get(), 0);
    assert_eq!(adapter.export.get(), 0);
}

#[test]
fn conversion_repeated_images_dedup_and_template_uuid_normalizes() {
    // Upper-case hex spelling in the configuration projection.
    let fixture = ConversionFixture::new("00000000-0000-0000-0000-00000000ABCD");
    fs::create_dir_all(fixture.options.input_root.join("assets")).unwrap();
    fs::write(fixture.options.input_root.join("assets/shot.png"), WIDE_PNG).unwrap();
    fixture.write_input(
        "policy.md",
        "---\ntitle: Policy\nversion: 1.0\nconfidentiality: internal\n---\n\
# Policy\n\n![one](assets/shot.png)\n\n![two](assets/shot.png)\n\n![three](assets/shot.png)\n",
    );

    let mut adapter = NeverCalled::default();
    let manifest = ConversionPlan::prepare(&fixture.options)
        .unwrap()
        .run(&mut adapter)
        .unwrap();

    assert!(!manifest.has_failures());
    let result = &manifest.results[0];
    // Three references to the same canonical asset => exactly one manifest entry.
    assert_eq!(
        result.assets.len(),
        1,
        "repeated references were not deduplicated"
    );
    assert_eq!(result.assets[0].relative_path, Path::new("assets/shot.png"));
    assert_eq!(result.assets[0].sha256, digest(WIDE_PNG));

    // Emitted template fields: the UUID is normalized to canonical lower-case,
    // while the other three contract fields are carried verbatim.
    assert_eq!(
        manifest.template.id.to_string(),
        "00000000-0000-0000-0000-00000000abcd"
    );
    assert_eq!(
        manifest.template.relative_path,
        PathBuf::from("Markdown-template.docx")
    );
    assert_eq!(manifest.template.sha256, digest(TEMPLATE));
    assert_eq!(manifest.template.contract_version, 1);

    // The serialized (emitted) id is the canonical spelling.
    let emitted: Value = serde_json::to_value(&manifest.template).unwrap();
    assert_eq!(emitted["id"], json!("00000000-0000-0000-0000-00000000abcd"));

    // Semantic struct equality accepts the upper-case spelling: the config was
    // never rejected for a "template equality mismatch".
    let canonical: MarkdownTemplateAsset = serde_json::from_value(json!({
        "id": "00000000-0000-0000-0000-00000000abcd",
        "relative_path": "Markdown-template.docx",
        "sha256": digest(TEMPLATE),
        "contract_version": 1
    }))
    .unwrap();
    assert!(manifest.template == canonical);
}

#[test]
fn template_asset_uuid_parses_case_insensitively_and_normalizes_output() {
    let sha = digest(TEMPLATE);
    let upper: MarkdownTemplateAsset = serde_json::from_str(&format!(
        r#"{{"id":"00000000-0000-0000-0000-00000000ABCD","relative_path":"Markdown-template.docx","sha256":"{sha}","contract_version":1}}"#
    ))
    .unwrap();
    let lower: MarkdownTemplateAsset = serde_json::from_str(&format!(
        r#"{{"id":"00000000-0000-0000-0000-00000000abcd","relative_path":"Markdown-template.docx","sha256":"{sha}","contract_version":1}}"#
    ))
    .unwrap();

    // Parsing is case-insensitive; both spellings denote the same identity, so
    // full typed equality does NOT reject the valid upper-case manifest input.
    assert!(upper == lower);
    assert_eq!(upper.id, lower.id);

    // Emission always normalizes to lowercase hyphenated.
    assert_eq!(upper.id.to_string(), "00000000-0000-0000-0000-00000000abcd");
    assert_eq!(
        serde_json::to_value(&upper).unwrap()["id"],
        json!("00000000-0000-0000-0000-00000000abcd")
    );
}
