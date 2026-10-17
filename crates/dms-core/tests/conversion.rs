use dms_core::{
    ConversionExporter, ConversionOptions, ConversionPlan, ConversionStatus, ExportChrome,
    SCHEMA_VERSION,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

const TEMPLATE: &[u8] = include_bytes!("fixtures/markdown-template.docx");
const IMAGE: &[u8] = include_bytes!("fixtures/markdown-small.png");
const SOURCE: &str = "---\ntitle: Synthetic policy\nversion: 3.0\nconfidentiality: internal\n---\n# Synthetic body\n\n| One | Two |\n| --- | --- |\n| alpha | beta |\n";
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

struct Fixture {
    temp: tempfile::TempDir,
    options: ConversionOptions,
}
impl Fixture {
    fn new(schema: u32) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("edit/.dms")).unwrap();
        fs::create_dir(root.join("publish")).unwrap();
        fs::create_dir(root.join("input")).unwrap();
        fs::write(root.join("edit/template.docx"), TEMPLATE).unwrap();
        let config = json!({"schema_version": schema, "edit_root": root.join("edit"), "publish_root": root.join("publish"),
            "markdown_template": {"id": "00000000-0000-0000-0000-000000000001", "relative_path": "template.docx", "sha256": digest(TEMPLATE), "contract_version": 1},
            "confidentiality_types": {"internal": {"id": "internal", "label": "Internal label", "enabled": true}},
            // Deliberately not deserializable as Workspace: no identity activation,
            // lifecycle validation, lock acquisition or migration is possible.
            "identity_source": "must not deserialize", "identity_cache": "must not read", "documents": "must not open"});
        let options = ConversionOptions {
            workspace_config: root.join("edit/.dms/workspace.json"),
            input_root: root.join("input"),
            output_root: root.join("output"),
            recursive: false,
            dry_run: false,
            without_toc: false,
        };
        fs::write(
            &options.workspace_config,
            serde_json::to_vec_pretty(&config).unwrap(),
        )
        .unwrap();
        fs::write(options.input_root.join("policy.md"), SOURCE).unwrap();
        Self { temp, options }
    }
    fn config(&self, f: impl FnOnce(&mut Value)) {
        let mut value: Value =
            serde_json::from_slice(&fs::read(&self.options.workspace_config).unwrap()).unwrap();
        f(&mut value);
        fs::write(
            &self.options.workspace_config,
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
    }
    fn dry(&mut self) {
        self.options.dry_run = true;
    }

    fn date_template(&self, stories: &[&str]) {
        let mut zip = ZipArchive::new(std::io::Cursor::new(TEMPLATE)).unwrap();
        let mut out = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index).unwrap();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            if stories.contains(&entry.name()) {
                let xml = String::from_utf8(bytes).unwrap();
                // Before the body prototypes, or before the story's first paragraph.
                bytes = xml
                    .replacen("<w:p>", "<w:p><w:r><w:t>{DATE}</w:t></w:r></w:p><w:p>", 1)
                    .into_bytes();
            }
            out.start_file(entry.name(), SimpleFileOptions::default())
                .unwrap();
            out.write_all(&bytes).unwrap();
        }
        let bytes = out.finish().unwrap().into_inner();
        fs::write(self.temp.path().join("edit/template.docx"), &bytes).unwrap();
        self.config(|v| v["markdown_template"]["sha256"] = json!(digest(&bytes)));
    }
}

#[derive(Default)]
struct FakeWord {
    preflight_calls: Cell<usize>,
    paths: Vec<PathBuf>,
    failure: Option<&'static str>,
    changed_source: Option<PathBuf>,
    changed_asset: Option<PathBuf>,
    expected_date: Option<String>,
    pdf_from_docx: bool,
}
impl ConversionExporter for FakeWord {
    fn preflight(&self) -> Result<(), String> {
        self.preflight_calls.set(self.preflight_calls.get() + 1);
        if self.failure == Some("unavailable") {
            Err("installed Word is unavailable".into())
        } else {
            Ok(())
        }
    }
    fn export(&mut self, docx: &Path, pdf: &Path, chrome: &ExportChrome) -> Result<String, String> {
        assert_eq!(chrome.title, "Synthetic policy");
        assert_eq!(chrome.version_label, "3.0");
        assert_eq!(chrome.confidentiality.type_id, "internal");
        assert_eq!(chrome.confidentiality.label, "Internal label");
        self.paths.push(docx.to_path_buf());
        assert!(docx.exists());
        if let Some(date) = &self.expected_date {
            // DATE is already filled before the controlled-field/Word adapter.
            for story in ["word/document.xml", "word/header1.xml", "word/footer1.xml"] {
                let text = package_text(docx, story);
                assert!(text.contains(date), "{story}");
                assert!(!text.contains("{DATE}"), "{story}");
                assert!(!text.contains("WrongDate"), "{story}");
            }
            let custom = package_text(docx, "docProps/custom.xml");
            for token in [
                "{TITLE}",
                "{DOCUMENT_NUMBER}",
                "{VERSION}",
                "{CONFIDENTIALITY}",
            ] {
                assert!(custom.contains(token), "reserved token {token}");
            }
            assert!(!custom.contains(date) && !custom.contains("DMS_DATE"));
        }
        fill_and_save(docx, chrome);
        if let Some(path) = &self.changed_source {
            fs::write(path, "changed while Word ran").unwrap();
        }
        if let Some(path) = &self.changed_asset {
            fs::write(path, "changed while Word ran").unwrap();
        }
        if self.failure == Some("word") {
            fs::write(pdf, b"partial").unwrap();
            return Err("Word failed to save/export".into());
        }
        if self.failure == Some("docx") {
            fs::write(docx, "not DOCX").unwrap();
        }
        if self.failure != Some("missing") {
            fs::write(
                pdf,
                if self.failure == Some("pdf") {
                    b"%PDF-1.7\nnot a PDF".to_vec()
                } else if self.pdf_from_docx {
                    // Synthetic text transport only, not Word layout/structure proof.
                    synthetic_pdf_text(&xml_text(&package_text(docx, "word/document.xml")))
                } else {
                    synthetic_pdf()
                },
            )
            .unwrap();
        }
        Ok("test-double, not installed Word".into())
    }
}
fn fill_and_save(path: &Path, chrome: &ExportChrome) {
    let mut zip = ZipArchive::new(std::io::Cursor::new(fs::read(path).unwrap())).unwrap();
    let mut out = ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name().ends_with(".xml") {
            let mut text = String::from_utf8(bytes).unwrap();
            for (token, value) in [
                ("{TITLE}", chrome.title.as_str()),
                ("{VERSION}", &chrome.version_label),
                (
                    "{DOCUMENT_NUMBER}",
                    chrome.document_number.as_deref().unwrap_or(""),
                ),
                ("{CONFIDENTIALITY}", &chrome.confidentiality.label),
            ] {
                text = text.replace(token, value);
            }
            bytes = text.into_bytes();
        }
        out.start_file(entry.name(), SimpleFileOptions::default())
            .unwrap();
        out.write_all(&bytes).unwrap();
    }
    // Simulate Word saving its final updated fields into the retained document.
    out.start_file(
        "docProps/synthetic-word-save.xml",
        SimpleFileOptions::default(),
    )
    .unwrap();
    out.write_all(b"<saved>updated fields and pagination test double</saved>")
        .unwrap();
    fs::write(path, out.finish().unwrap().into_inner()).unwrap();
}
fn package_text(path: &Path, part: &str) -> String {
    let mut zip = ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let mut text = String::new();
    zip.by_name(part)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

fn synthetic_pdf() -> Vec<u8> {
    synthetic_pdf_text("Synthetic policy")
}

fn xml_text(xml: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut text = String::new();
    loop {
        match reader.read_event().unwrap() {
            quick_xml::events::Event::Text(value) => text.push_str(&value.unescape().unwrap()),
            quick_xml::events::Event::End(tag) if tag.name().as_ref() == b"w:p" => text.push('\n'),
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    text
}

fn synthetic_pdf_text(text: &str) -> Vec<u8> {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('\u{a0}', "\\240")
        .replace('\n', ") Tj 0 -14 Td (");
    let stream = format!("BT /F1 12 Tf 72 720 Td ({escaped}) Tj ET");
    let objects = ["<< /Type /Catalog /Pages 2 0 R >>".to_owned(), "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_owned(), format!("<< /Length {} >>\nstream\n{stream}\nendstream",stream.len())];
    let mut pdf = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        write!(pdf, "{} 0 obj\n{}\nendobj\n", i + 1, object).unwrap();
    }
    let xref = pdf.len();
    write!(pdf, "xref\n0 6\n0000000000 65535 f \n").unwrap();
    for offset in offsets {
        writeln!(pdf, "{offset:010} 00000 n ").unwrap();
    }
    write!(
        pdf,
        "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
    )
    .unwrap();
    pdf
}
fn tree(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, p: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(p).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(root, &p, out);
            } else {
                out.insert(
                    p.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(path, path, &mut out);
    out
}

fn todo_source(body: &str, override_value: Option<&str>) -> String {
    let setting = override_value
        .map(|value| format!("todo_projection: {value}\n"))
        .unwrap_or_default();
    format!("---\ntitle: Synthetic policy\nversion: 3.0\nconfidentiality: internal\n{setting}---\n{body}")
}

#[test]
fn todo_projection_preserves_docx_structure_and_transports_only_retained_text_to_pdf() {
    let f = Fixture::new(SCHEMA_VERSION);
    f.config(|v| v["conversion_policy"] = json!({"todo_projection": "enabled"}));
    let source = todo_source(
        "TODO: PRIVATE_PARAGRAPH\n\n# tOdO: PRIVATE_HEADING\n\nKept after heading.\n\n**TO**DO: PRIVATE_SPLIT\n\n[TODO: PRIVATE_LINK](https://example.test)\n\n```rust\nTODO: PRIVATE_CODE\n```\n\n    todo: PRIVATE_INDENTED_CODE\n\n- **TODO:** PRIVATE_ITEM\n\n  PRIVATE_CONTINUATION\n\n  - PRIVATE_CHILD\n\n- Kept bullet\n\n> TODO: PRIVATE_QUOTE\n\n# Kept heading\n\nKept **bold** and *italic* and `code` and [link](https://example.test).\n\n![TODO: alt is not body](image.png)\n\nTODO: PRIVATE_IMAGE ![unused](missing.png)\n\n| TODO: PRIVATE_HEADER | Kept header | Third |\n| --- | --- | --- |\n| **TODO:** PRIVATE_CELL | Kept cell | Last |\n| Ordinary | `todo: PRIVATE_CODE_CELL` | End |\n",
        None,
    );
    fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
    fs::write(f.options.input_root.join("image.png"), IMAGE).unwrap();
    let before_input = tree(&f.options.input_root);
    let before_edit = tree(&f.temp.path().join("edit"));
    let mut word = FakeWord {
        pdf_from_docx: true,
        ..Default::default()
    };
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(!manifest.has_failures(), "{manifest:?}");
    let result = &manifest.results[0];
    assert_eq!(
        result.todo_projection,
        Some(dms_core::TodoProjection::Enabled)
    );
    assert_eq!(
        result.input_sha256.as_deref(),
        Some(digest(source.as_bytes()).as_str())
    );
    assert_eq!(result.assets.len(), 1);
    assert_eq!(result.assets[0].sha256, digest(IMAGE));
    let xml = package_text(&word.paths[0], "word/document.xml");
    let text = xml_text(&xml);
    assert!(!text.contains("PRIVATE_"), "{text}");
    for retained in [
        "Kept after heading.",
        "Kept bullet",
        "Kept heading",
        "Kept bold and italic and code and link (https://example.test).",
        "Kept cell",
        "Ordinary",
        "Last",
        "End",
    ] {
        assert!(text.contains(retained), "missing {retained}");
    }
    for markup in [
        "<w:b/>",
        "<w:i/>",
        "<w:drawing>",
        "<w:pStyle w:val=\"Heading1\"/>",
        "<w:tblLayout w:type=\"fixed\"/>",
    ] {
        assert!(xml.contains(markup), "missing {markup}");
    }
    assert_eq!(xml.matches("<w:tr>").count(), 3);
    assert_eq!(xml.matches("<w:tc>").count(), 9);
    assert_eq!(xml.matches("<w:gridCol ").count(), 3);
    assert_eq!(text.matches('\u{a0}').count(), 3);
    for cell in xml
        .split("<w:tc>")
        .skip(1)
        .map(|v| v.split("</w:tc>").next().unwrap())
    {
        if cell.contains('\u{a0}') {
            assert_eq!(xml_text(cell), "\u{a0}\n");
        }
    }
    let pdf = fs::read(f.options.output_root.join("policy.pdf")).unwrap();
    let pdf_text = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    assert!(!pdf_text.contains("PRIVATE_"));
    for retained in [
        "Kept after heading.",
        "Kept bullet",
        "Kept heading",
        "Kept cell",
        "Ordinary",
    ] {
        assert!(
            pdf_text.contains(retained),
            "PDF missing {retained}: {pdf_text}"
        );
    }
    assert_eq!(tree(&f.options.input_root), before_input);
    assert_eq!(tree(&f.temp.path().join("edit")), before_edit);
    assert!(tree(&f.temp.path().join("publish")).is_empty());
    let receipt = fs::read_to_string(f.options.output_root.join("manifest.json")).unwrap();
    assert!(!receipt.contains("PRIVATE_") && !receipt.contains("Kept cell"));
    assert_eq!(
        serde_json::from_str::<Value>(&receipt).unwrap(),
        serde_json::to_value(&manifest).unwrap()
    );
}

#[test]
fn todo_projection_matches_retained_source_assembly_across_nested_and_inline_content() {
    let f = Fixture::new(SCHEMA_VERSION);
    let body = "# Kept heading\n\n**TO**DO: drop formatted marker\n\nKept **bold**, *italic*, `code`, ~~strike~~, $math$, <span>raw</span> and [link](https://example.test).\n\n> TODO: drop quoted paragraph\n>\n> Kept quote\n\n4. TODO: drop first\n5. Kept parent\n\n   8. TODO: drop nested first\n   9. Kept nested\n\n   Kept continuation\n\n6. TODO: drop last\n\n- Kept table parent\n\n  | **TODO:** drop header | Kept header |\n  | --- | --- |\n  | TODO: drop cell | Kept *cell* |\n\n- [x] Kept task\n\n```text\n  TODO: drop code\n```\n\n```TODO:language-is-not-body\nKept code\n```\n\n![TODO: alt-only](image.png)\n\n&#84;ODO: drop entity marker\n\nTODO\\: drop escaped marker\n\nKept final paragraph.\n";
    let retained = "# Kept heading\n\nKept **bold**, *italic*, `code`, ~~strike~~, $math$, <span>raw</span> and [link](https://example.test).\n\n> Kept quote\n\n4. Kept parent\n\n   8. Kept nested\n\n   Kept continuation\n\n- Kept table parent\n\n  | \u{a0} | Kept header |\n  | --- | --- |\n  | \u{a0} | Kept *cell* |\n\n- [x] Kept task\n\n```TODO:language-is-not-body\nKept code\n```\n\n![TODO: alt-only](image.png)\n\nKept final paragraph.\n";
    // An unmarked empty parent survives removal of its marked child; raw HTML
    // without a marker remains literal text, just as in the ordinary assembler.
    let body = format!("{body}\n-\n  - TODO: drop child\n\n<p>Kept raw block</p>\n");
    let retained = format!("{retained}\n-\n\n<p>Kept raw block</p>\n");
    let source = todo_source(&body, Some("enabled"));
    let source_path = f.options.input_root.join("policy.md");
    fs::write(&source_path, &source).unwrap();
    fs::write(f.options.input_root.join("image.png"), IMAGE).unwrap();
    let before = tree(&f.options.input_root);
    let mut word = FakeWord::default();
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(!manifest.has_failures(), "{manifest:?}");
    let expected = f.temp.path().join("expected.docx");
    dms_core::assemble_markdown_docx(
        &f.temp.path().join("edit/template.docx"),
        &todo_source(&retained, None),
        &expected,
        Some(&dms_core::MarkdownAssetContext::new(
            &f.options.input_root,
            &source_path,
        )),
    )
    .unwrap();
    fill_and_save(
        &expected,
        &ExportChrome {
            title: "Synthetic policy".into(),
            document_number: None,
            version_label: "3.0".into(),
            confidentiality: dms_core::ConfidentialitySnapshot {
                type_id: "internal".into(),
                label: "Internal label".into(),
            },
        },
    );
    let actual_parts = package_parts(&word.paths[0]);
    assert_eq!(actual_parts, package_parts(&expected));
    assert_eq!(tree(&f.options.input_root), before);
}

fn package_parts(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut archive = ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut entry = archive.by_index(index).unwrap();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            (entry.name().to_owned(), bytes)
        })
        .collect()
}

#[test]
fn todo_projection_compacts_ordered_items_and_preserves_starts_restarts_and_continuations() {
    let f = Fixture::new(SCHEMA_VERSION);
    let source = todo_source(
        "4. TODO: dropped first\n5. First survivor\n6. TODO: dropped middle\n7. Second survivor\n\n   Continuation\n\n   8. TODO: dropped nested\n   9. Nested survivor\n\n8. Third survivor\n\nSeparate paragraph\n\n20. Restart survivor\n21. TODO: dropped final\n\nAnother paragraph\n\n1. TODO: entire list\n2. TODO: entire list too\n",
        Some("enabled"),
    );
    fs::write(f.options.input_root.join("policy.md"), source).unwrap();
    let mut word = FakeWord::default();
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(!manifest.has_failures(), "{manifest:?}");
    let xml = package_text(&word.paths[0], "word/document.xml");
    let numbering = package_text(&word.paths[0], "word/numbering.xml");
    assert!(!xml.contains("TODO:") && !xml.contains("dropped"));
    assert_eq!(numbering.matches("<w:num ").count(), 3);
    for start in [4, 8, 20] {
        assert!(numbering.contains(&format!("<w:startOverride w:val=\"{start}\"/>")));
    }
    let paragraph = |label: &str| {
        let position = xml.find(label).unwrap();
        let start = xml[..position].rfind("<w:p>").unwrap();
        let end = position + xml[position..].find("</w:p>").unwrap();
        &xml[start..end]
    };
    let num_id = |p: &str| {
        p.split("<w:numId w:val=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_owned()
    };
    let outer_id = num_id(paragraph("First survivor"));
    for label in ["Second survivor", "Third survivor"] {
        assert_eq!(num_id(paragraph(label)), outer_id);
    }
    assert_ne!(num_id(paragraph("Nested survivor")), outer_id);
    assert_ne!(num_id(paragraph("Restart survivor")), outer_id);
    assert_eq!(num_id(paragraph("Continuation")), "0");
    // One shared native instance, three numbered paragraphs, no per-item reset:
    // Word owns 4,5,6 despite removed source items.
    assert_eq!(
        xml.matches(&format!("<w:numId w:val=\"{outer_id}\"/>"))
            .count(),
        3
    );
}

#[test]
fn todo_projection_removes_renderer_visible_entries_only_when_enabled() {
    for body in [
        "---\nToDo: PRIVATE_DROP\n---\n\nKept body\n",
        "+++\nToDo: PRIVATE_DROP\n+++\n\nKept body\n",
        "**ToDo:** PRIVATE_DROP\n: Kept body\n",
        "Kept body\n: **ToDo:** PRIVATE_DROP\n",
        "[**TODO:** PRIVATE_DROP](TODO:PRIVATE_DESTINATION)\n\nKept body\n",
        "| A | B |\n| --- | --- |\n| [TODO: PRIVATE_DROP](TODO:PRIVATE_DESTINATION) | Kept body |\n",
    ] {
        for policy in ["enabled", "disabled"] {
            let f = Fixture::new(SCHEMA_VERSION);
            let source = todo_source(body, Some(policy));
            fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
            let mut word = FakeWord {
                pdf_from_docx: true,
                ..Default::default()
            };
            let manifest = ConversionPlan::prepare(&f.options)
                .unwrap()
                .run(&mut word)
                .unwrap();
            assert!(!manifest.has_failures());
            assert_eq!(word.paths.len(), 1);
            let xml = package_text(&word.paths[0], "word/document.xml");
            let pdf = pdf_extract::extract_text_from_mem(
                &fs::read(f.options.output_root.join("policy.pdf")).unwrap(),
            )
            .unwrap();
            for text in [&xml, &pdf] {
                assert!(text.contains("Kept body"));
                assert_eq!(text.contains("PRIVATE_DROP"), policy == "disabled");
            }
            let release = f.temp.path().join("release.docx");
            dms_core::assemble_markdown_docx(
                &f.temp.path().join("edit/template.docx"),
                &source,
                &release,
                None,
            )
            .unwrap();
            let release_xml = package_text(&release, "word/document.xml");
            assert!(release_xml.contains("PRIVATE_DROP"));
            if policy == "disabled" {
                assert_eq!(xml, release_xml);
            }
            assert_eq!(
                fs::read_to_string(f.options.input_root.join("policy.md")).unwrap(),
                source
            );
            assert_eq!(
                manifest.results[0].input_sha256.as_deref(),
                Some(digest(source.as_bytes()).as_str())
            );
            assert!(!serde_json::to_string(&manifest)
                .unwrap()
                .contains("PRIVATE_"));
        }
    }
}

#[test]
fn todo_projection_matches_renderer_visible_retained_source_without_widening_entries() {
    for (body, retained) in [
        ("Kept term\n: **ToDo:** PRIVATE_DROP\n", "Kept term\n"),
        ("Kept term\n: TODO: PRIVATE_DROP\n  > PRIVATE_DESCENDANT\n\nKept final\n", "Kept term\n\nKept final\n"),
        ("Kept term\n: TODO: PRIVATE_DROP\n\nKept final\n", "Kept term\n\nKept final\n"),
        ("Kept term\n: TODO: PRIVATE_DROP\n  - PRIVATE_DESCENDANT\n\nKept final\n", "Kept term\n\nKept final\n"),
        ("Kept term\n: Kept definition\n  - Kept child\n\nKept final\n", "Kept term\n: Kept definition\n  - Kept child\n\nKept final\n"),
        ("Kept term\n: TODO: PRIVATE_DROP\n: Kept definition\n\nKept final\n", "Kept term\n: Kept definition\n\nKept final\n"),
        ("TODO: PRIVATE_DROP\n: TODO: PRIVATE_BODY\n\nKept final\n", "Kept final\n"),
        ("Kept term\n\n: TODO: PRIVATE_DROP\n\nKept final\n", "Kept term\n\nKept final\n"),
        ("- Kept parent\n  - TODO: PRIVATE_DROP\n\nKept final\n", "- Kept parent\n\nKept final\n"),
        ("> TODO: PRIVATE_DROP\n\nKept final\n", "Kept final\n"),
        ("Kept term\n: TODO: PRIVATE_DROP\n  > PRIVATE_DESCENDANT\n\nKept second\n: Kept definition\n", "Kept term\n\nKept second\n: Kept definition\n"),
        ("Kept term\n: Kept definition\n  > Kept quote\n", "Kept term\n: Kept definition\n  > Kept quote\n"),
        ("NOTODO: kept\n\nmytodo: kept\n\n**TO**DO: PRIVATE_DROP\n", "NOTODO: kept\n\nmytodo: kept\n"),
        // Loose definitions contain independent paragraphs, not one removable entry.
        ("Kept term\n\n: TODO: PRIVATE_DROP\n\n  Kept paragraph\n", "Kept term\n\n: Kept paragraph\n"),
        ("Kept term\n\n: Kept paragraph\n\n  TODO: PRIVATE_DROP\n", "Kept term\n\n: Kept paragraph\n"),
        (
            "+++\nKept metadata\n+++\n\nKept term\n: Kept definition\n\n<div>\nKept HTML\n</div>\n\n[Kept link](https://example.test)\n\n| A | B |\n| --- | --- |\n| [Kept cell](https://example.test) | Other |\n",
            "+++\nKept metadata\n+++\n\nKept term\n: Kept definition\n\n<div>\nKept HTML\n</div>\n\n[Kept link](https://example.test)\n\n| A | B |\n| --- | --- |\n| [Kept cell](https://example.test) | Other |\n",
        ),
        (
            "NOTODO: kept identifier\n\n# NOTODO: kept heading\n\n- _TODO: kept underscore\n\n```text\n1TODO: kept digit\n```\n\n| A | B |\n| --- | --- |\n| éTODO: kept Unicode | [Kept link](https://example.test/NOTODO:topic) |\n",
            "NOTODO: kept identifier\n\n# NOTODO: kept heading\n\n- _TODO: kept underscore\n\n```text\n1TODO: kept digit\n```\n\n| A | B |\n| --- | --- |\n| éTODO: kept Unicode | [Kept link](https://example.test/NOTODO:topic) |\n",
        ),
    ] {
        for policy in ["enabled", "disabled"] {
            let f = Fixture::new(SCHEMA_VERSION);
            let source = todo_source(body, Some(policy));
            fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
            let before = tree(f.temp.path());
            let mut word = FakeWord::default();
            let manifest = ConversionPlan::prepare(&f.options).unwrap().run(&mut word).unwrap();
            assert!(!manifest.has_failures(), "{body}: {manifest:?}");
            let expected = f.temp.path().join("expected.docx");
            dms_core::assemble_markdown_docx(
                &f.temp.path().join("edit/template.docx"),
                &todo_source(if policy == "enabled" { retained } else { body }, Some(policy)),
                &expected,
                None,
            ).unwrap();
            fill_and_save(&expected, &ExportChrome {
                title: "Synthetic policy".into(),
                document_number: None,
                version_label: "3.0".into(),
                confidentiality: dms_core::ConfidentialitySnapshot {
                    type_id: "internal".into(),
                    label: "Internal label".into(),
                },
            });
            if policy == "enabled" && retained == "Kept term\n\nKept final\n" {
                let document = package_text(&word.paths[0], "word/document.xml");
                let text = xml_text(&document);
                let retained_paragraphs: Vec<_> = text
                    .lines()
                    .filter(|text| text.contains("Kept term") || text.contains("Kept final"))
                    .collect();
                assert_eq!(retained_paragraphs, ["Kept term", "Kept final"]);
                assert!(!document.contains("PRIVATE_DROP"));
                assert!(!document.contains("PRIVATE_DESCENDANT"));
            }
            assert!(package_parts(&word.paths[0]) == package_parts(&expected), "whole-package mismatch: {body}");
            assert_eq!(manifest.results[0].input_sha256.as_deref(), Some(digest(source.as_bytes()).as_str()));
            for (path, bytes) in before {
                assert_eq!(fs::read(f.temp.path().join(path)).unwrap(), bytes);
            }
        }
    }
}

#[test]
fn todo_projection_rejects_mixed_entries_and_cells_without_pairs_or_source_leaks() {
    for (body, kind, line, column) in [
        ("**bold**TODO: PRIVATE_DROP\n", "paragraph", 7, 1),
        ("*PRIVATE_KEEP*TODO: PRIVATE_DROP\n", "paragraph", 7, 1),
        ("`PRIVATE_KEEP`TODO: PRIVATE_DROP\n", "paragraph", 7, 1),
        ("~~PRIVATE_KEEP~~TODO: PRIVATE_DROP\n", "paragraph", 7, 1),
        (
            "Kept term\n: PRIVATE_KEEP\n  > TODO: PRIVATE_DROP\n",
            "definition body",
            8,
            1,
        ),
        (
            "Kept term\n: PRIVATE_KEEP TODO: PRIVATE_DROP\n  > Kept quote\n",
            "definition body",
            8,
            1,
        ),
        (
            "- PRIVATE_KEEP\n  # TODO: PRIVATE_DROP\n",
            "list item",
            7,
            1,
        ),
        (
            "- PRIVATE_KEEP\n  ```text\n  TODO: PRIVATE_DROP\n  ```\n",
            "list item",
            7,
            1,
        ),
        (
            "- PRIVATE_KEEP\n  : TODO: PRIVATE_DROP\n",
            "list item",
            7,
            1,
        ),
        (
            "NOTODO: PRIVATE_KEEP; TODO: PRIVATE_DROP\n",
            "paragraph",
            7,
            1,
        ),
        ("(TODO: PRIVATE_DROP)\n", "paragraph", 7, 1),
        (
            "Kept term\n\n: PRIVATE_KEEP TODO: PRIVATE_DROP\n",
            "paragraph",
            9,
            3,
        ),
        (
            "Kept term\n: PRIVATE_KEEP TODO: PRIVATE_DEFINITION\n",
            "definition body",
            8,
            1,
        ),
        (
            "[PRIVATE_KEEP](TODO:PRIVATE_DESTINATION)\n",
            "paragraph",
            7,
            1,
        ),
        (
            "| A | B |\n| --- | --- |\n| [PRIVATE_KEEP](TODO:PRIVATE_DESTINATION) | kept |\n",
            "table cell",
            9,
            2,
        ),
        (
            "| [PRIVATE_KEEP](TODO:PRIVATE_DESTINATION) | B |\n| --- | --- |\n",
            "table cell",
            7,
            2,
        ),
        ("<div>\nTODO: PRIVATE_HTML\n</div>\n", "paragraph", 7, 1),
        ("PRIVATE_KEEP TODO: PRIVATE_DROP\n", "paragraph", 7, 1),
        (
            "---\nPRIVATE_KEEP TODO: PRIVATE_DROP\n---\n",
            "paragraph",
            7,
            1,
        ),
        (
            "+++\nPRIVATE_KEEP TODO: PRIVATE_DROP\n+++\n",
            "paragraph",
            7,
            1,
        ),
        (
            "PRIVATE_KEEP **TODO:** PRIVATE_DROP\n: Kept definition\n",
            "paragraph",
            7,
            1,
        ),
        (
            "<p>PRIVATE_KEEP TODO: PRIVATE_DROP</p>\n",
            "paragraph",
            7,
            1,
        ),
        (
            "<!-- PRIVATE_KEEP TODO: PRIVATE_DROP -->\n",
            "paragraph",
            7,
            1,
        ),
        ("## PRIVATE_KEEP **TODO:** PRIVATE_DROP\n", "heading", 7, 1),
        ("- PRIVATE_KEEP TODO: PRIVATE_DROP\n", "list item", 7, 1),
        ("4. PRIVATE_KEEP TODO: PRIVATE_DROP\n", "list item", 7, 1),
        (
            "```rust\nPRIVATE_KEEP\nTODO: PRIVATE_DROP\n```\n",
            "code block",
            7,
            1,
        ),
        ("    PRIVATE_KEEP TODO: PRIVATE_DROP\n", "code block", 7, 5),
        (
            "| A | B |\n| --- | --- |\n| ok | PRIVATE_KEEP **TODO:** PRIVATE_DROP |\n",
            "table cell",
            9,
            7,
        ),
        (
            "- PRIVATE_KEEP\n\n  TODO: PRIVATE_DROP\n",
            "list item",
            7,
            1,
        ),
        (
            "![PRIVATE_ALT](image.png) TODO: PRIVATE_DROP\n",
            "paragraph",
            7,
            1,
        ),
        ("PRIVATE_KEEP\nTODO: PRIVATE_DROP\n", "paragraph", 7, 1),
        ("- [ ] TODO: PRIVATE_DROP\n", "list item", 7, 1),
        (
            "- PRIVATE_KEEP\n  > TODO: PRIVATE_DROP\n",
            "list item",
            7,
            1,
        ),
        (
            "See [notes](https://example.test/todo:topics).\n",
            "paragraph",
            7,
            1,
        ),
    ] {
        for dry_run in [false, true] {
            let mut f = Fixture::new(SCHEMA_VERSION);
            f.options.dry_run = dry_run;
            let source = todo_source(body, Some("enabled"));
            fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
            fs::write(f.options.input_root.join("z-good.md"), SOURCE).unwrap();
            let before_input = tree(&f.options.input_root);
            let before_edit = tree(&f.temp.path().join("edit"));
            let before_publish = tree(&f.temp.path().join("publish"));
            let mut word = FakeWord::default();
            let manifest = ConversionPlan::prepare(&f.options)
                .unwrap()
                .run(&mut word)
                .unwrap();
            assert!(manifest.has_failures(), "{body}");
            let failed = &manifest.results[0];
            assert_eq!(
                failed.todo_projection,
                Some(dms_core::TodoProjection::Enabled)
            );
            assert_eq!(
                failed.input_sha256.as_deref(),
                Some(digest(source.as_bytes()).as_str())
            );
            assert_eq!(failed.error.as_deref(), Some(format!("mixed TODO {kind} at line {line}, column {column}: TODO: follows substantive visible content; split retained content and TODO into separate entries").as_str()));
            assert!(failed.docx.is_none() && failed.pdf.is_none());
            assert!(failed.retained_diagnostics.is_empty());
            assert!(!f.options.output_root.join("policy.docx").exists());
            assert!(!f.options.output_root.join("policy.pdf").exists());
            assert!(manifest.results[1].error.is_none());
            assert_eq!(word.paths.len(), usize::from(!dry_run));
            assert!(!serde_json::to_string(&manifest)
                .unwrap()
                .contains("PRIVATE_"));
            assert_eq!(
                fs::read_to_string(f.options.input_root.join("policy.md")).unwrap(),
                source
            );
            assert_eq!(tree(&f.options.input_root), before_input);
            assert_eq!(tree(&f.temp.path().join("edit")), before_edit);
            assert_eq!(tree(&f.temp.path().join("publish")), before_publish);
            if dry_run {
                assert!(!f.options.output_root.exists());
            }
        }
    }
}

#[test]
fn todo_projection_disabled_is_identical_to_release_assembly_and_override_wins() {
    for (library, override_value) in [
        (None, None),
        (Some("disabled"), None),
        (Some("enabled"), Some("disabled")),
    ] {
        let f = Fixture::new(SCHEMA_VERSION);
        if let Some(value) = library {
            f.config(|v| v["conversion_policy"] = json!({"todo_projection": value}));
        }
        let source = todo_source("TODO: retained paragraph\n\n# TODO: retained heading\n\n4. first\n5. TODO: retained item\n6. last\n\nRetained TODO: mixed\n\n```rust\nTODO: retained code\n```\n\n| A | B |\n| --- | --- |\n| TODO: cell | Mixed TODO: cell |\n", override_value);
        fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
        let mut word = FakeWord {
            pdf_from_docx: true,
            ..Default::default()
        };
        let manifest = ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut word)
            .unwrap();
        assert!(!manifest.has_failures());
        assert_eq!(
            manifest.results[0].todo_projection,
            Some(dms_core::TodoProjection::Disabled)
        );
        let release = f.temp.path().join("release.docx");
        dms_core::assemble_markdown_docx(
            &f.temp.path().join("edit/template.docx"),
            &source,
            &release,
            None,
        )
        .unwrap();
        for part in ["word/document.xml", "word/numbering.xml"] {
            assert_eq!(
                package_text(&word.paths[0], part),
                package_text(&release, part)
            );
        }
        let text = pdf_extract::extract_text_from_mem(
            &fs::read(f.options.output_root.join("policy.pdf")).unwrap(),
        )
        .unwrap();
        assert!(text.contains("TODO: retained paragraph") && text.contains("Mixed TODO: cell"));
    }
}

#[test]
fn todo_projection_enabled_without_markers_preserves_assembly_and_release_ignores_enabled_override()
{
    let f = Fixture::new(SCHEMA_VERSION);
    let source = todo_source("# Kept heading\n\nText with **bold**, *italic* and [link](https://example.test).\n\n4. First\n5. Second\n\n   Continuation\n\n   - Nested\n\n| A | B |\n| --- | --- |\n| First cell | Second cell |\n", Some("enabled"));
    fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
    let mut word = FakeWord::default();
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(!manifest.has_failures());
    let release = f.temp.path().join("release.docx");
    let template = f.temp.path().join("edit/template.docx");
    dms_core::assemble_markdown_docx(&template, &source, &release, None).unwrap();
    for part in [
        "word/document.xml",
        "word/numbering.xml",
        "word/_rels/document.xml.rels",
    ] {
        assert_eq!(
            package_text(&word.paths[0], part),
            package_text(&release, part)
        );
    }
    let source =
        source + "\nTODO: release keeps enabled content\n\nRetained TODO: mixed release content\n";
    dms_core::assemble_markdown_docx(&template, &source, &release, None).unwrap();
    let xml = package_text(&release, "word/document.xml");
    assert!(xml.contains("TODO: release keeps enabled content"));
    assert!(xml.contains("Retained TODO: mixed release content"));
}

#[test]
fn todo_projection_invalid_overrides_have_no_effective_policy_or_artifacts() {
    for value in [
        "true",
        "ENABLED",
        "PRIVATE_VALUE",
        "enabled\ntodo_projection: disabled",
    ] {
        let f = Fixture::new(SCHEMA_VERSION);
        let source = todo_source("PRIVATE_BODY", Some(value));
        fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
        let mut word = FakeWord::default();
        let manifest = ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut word)
            .unwrap();
        assert!(manifest.has_failures());
        let result = &manifest.results[0];
        assert!(result.todo_projection.is_none());
        assert!(result.docx.is_none() && result.pdf.is_none());
        assert!(word.paths.is_empty());
        assert_eq!(
            result.input_sha256.as_deref(),
            Some(digest(source.as_bytes()).as_str())
        );
        assert_eq!(fs::read_dir(&f.options.output_root).unwrap().count(), 1);
        assert!(!serde_json::to_string(&manifest)
            .unwrap()
            .contains("PRIVATE_"));
    }
}

#[test]
fn configuration_projection_is_read_only_for_supported_schemas_and_dry_run_never_calls_word() {
    for schema in [18, 19, 21, SCHEMA_VERSION] {
        let mut f = Fixture::new(schema);
        f.dry();
        fs::write(f.temp.path().join("edit/.dms/lock"), "existing lock").unwrap();
        fs::write(
            f.temp.path().join("edit/.dms/AGENTS.md"),
            "manual instructions",
        )
        .unwrap();
        let before = tree(f.temp.path());
        let mut word = FakeWord::default();
        let result = ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut word)
            .unwrap();
        assert!(!result.has_failures());
        assert!(result.conversion_only && result.dry_run);
        assert!(matches!(
            result.results[0].status,
            ConversionStatus::Validated
        ));
        assert_eq!(word.preflight_calls.get(), 0);
        assert!(word.paths.is_empty());
        assert_eq!(tree(f.temp.path()), before);
    }
}
#[test]
fn todo_policy_resolution_is_read_only_and_manifest_keeps_policy_separate_from_metadata() {
    use dms_core::TodoProjection;
    for library in [None, Some("disabled"), Some("enabled")] {
        let f = Fixture::new(SCHEMA_VERSION);
        if let Some(setting) = library {
            f.config(|v| v["conversion_policy"] = json!({"todo_projection": setting}));
        }
        let source = SOURCE.replacen(
            "title:",
            "todo_projection: enabled\nprivate_extra: PRIVATE_EXTRA\ntitle:",
            1,
        ) + "\nTODO: PRIVATE_BODY\n";
        fs::write(f.options.input_root.join("policy.md"), &source).unwrap();
        fs::write(f.temp.path().join("edit/.dms/lock"), "existing lock").unwrap();
        let before = tree(f.temp.path());
        let plan = ConversionPlan::prepare(&f.options).unwrap();
        let inherited = if library == Some("enabled") {
            TodoProjection::Enabled
        } else {
            TodoProjection::Disabled
        };
        assert_eq!(plan.resolve_todo_projection(SOURCE).unwrap(), inherited);
        assert_eq!(
            plan.resolve_todo_projection(&source).unwrap(),
            TodoProjection::Enabled
        );
        assert_eq!(
            plan.resolve_todo_projection(
                &source.replace("todo_projection: enabled", "todo_projection: disabled")
            )
            .unwrap(),
            TodoProjection::Disabled
        );
        let invalid = source.replace(
            "todo_projection: enabled",
            "todo_projection: PRIVATE_INVALID",
        );
        let error = plan
            .resolve_todo_projection(&invalid)
            .unwrap_err()
            .to_string();
        assert_eq!(error, "todo_projection must be enabled or disabled");
        assert_eq!(tree(f.temp.path()), before);

        let mut word = FakeWord::default();
        let manifest = plan.run(&mut word).unwrap();
        assert!(!manifest.has_failures());
        assert_eq!(
            manifest.results[0].input_sha256.as_deref(),
            Some(digest(source.as_bytes()).as_str())
        );
        assert!(!package_text(&word.paths[0], "word/document.xml").contains("TODO: PRIVATE_BODY"));
        let serialized = serde_json::to_value(&manifest).unwrap();
        assert_eq!(serialized["results"][0]["todo_projection"], "enabled");
        let metadata = serialized["results"][0]["metadata"].as_object().unwrap();
        assert_eq!(
            metadata.keys().map(String::as_str).collect::<Vec<_>>(),
            ["confidentiality", "document_number", "title", "version"]
        );
        let text = serde_json::to_string(&manifest).unwrap();
        for excluded in [
            "PRIVATE_EXTRA",
            "PRIVATE_BODY",
            "identity_source",
            "documents",
            "workflow",
        ] {
            assert!(!text.contains(excluded), "manifest leaked {excluded}");
        }
        for (path, bytes) in before {
            assert_eq!(fs::read(f.temp.path().join(path)).unwrap(), bytes);
        }
        let disk: Value =
            serde_json::from_slice(&fs::read(f.options.output_root.join("manifest.json")).unwrap())
                .unwrap();
        assert_eq!(disk, serialized);
    }
}

#[test]
fn todo_policy_projection_rejects_invalid_configuration_and_unsupported_schemas_without_writes() {
    for schema in [18, 19, 21, SCHEMA_VERSION] {
        for policy in [
            json!(null),
            json!({}),
            json!({"todo_projection": true}),
            json!({"todo_projection": "Enabled"}),
            json!({"todo_projection": "PRIVATE_INVALID"}),
            json!({"todo_projection": "enabled", "extra": true}),
        ] {
            let f = Fixture::new(schema);
            f.config(|v| v["conversion_policy"] = policy);
            let before = tree(f.temp.path());
            let error = ConversionPlan::prepare(&f.options)
                .err()
                .unwrap()
                .to_string();
            assert_eq!(error, "invalid conversion configuration projection");
            assert_eq!(tree(f.temp.path()), before);
        }
    }
    for schema in [0, 17, 20, SCHEMA_VERSION + 1] {
        let f = Fixture::new(schema);
        let before = tree(f.temp.path());
        assert!(ConversionPlan::prepare(&f.options).is_err());
        assert_eq!(tree(f.temp.path()), before);
    }
}

#[test]
fn conversion_date_fills_all_stories_before_export_and_preserves_non_release_inputs() {
    for date in ["2026-10-14", "2024-02-29", "2000-02-29"] {
        let f = Fixture::new(SCHEMA_VERSION);
        f.date_template(&["word/document.xml", "word/header1.xml", "word/footer1.xml"]);
        let source = SOURCE.replacen("title:", &format!("last_modified: {date}\ndate: WrongDate\nTITLE: WrongTitle\nVERSION: 99.99\nCONFIDENTIALITY: WrongLabel\nDOCUMENT_NUMBER: invented\ntitle:"), 1);
        let source_path = f.options.input_root.join("policy.md");
        fs::write(&source_path, &source).unwrap();
        fs::write(f.temp.path().join("edit/.dms/lock"), "existing lock").unwrap();
        let before_edit = tree(&f.temp.path().join("edit"));
        let before_input = tree(&f.options.input_root);
        let before_publish = tree(&f.temp.path().join("publish"));
        let source_mtime = fs::metadata(&source_path).unwrap().modified().unwrap();
        let mut word = FakeWord {
            expected_date: Some(date.to_owned()),
            ..Default::default()
        };
        let manifest = ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut word)
            .unwrap();
        assert!(!manifest.has_failures());
        assert!(manifest.conversion_only);
        assert_eq!(word.paths.len(), 1);
        assert_eq!(
            manifest.results[0].input_sha256.as_deref(),
            Some(digest(source.as_bytes()).as_str())
        );
        for story in ["word/document.xml", "word/header1.xml", "word/footer1.xml"] {
            let xml = package_text(&word.paths[0], story);
            assert!(xml.contains(date) && !xml.contains("{DATE}"));
        }
        assert_eq!(tree(&f.temp.path().join("edit")), before_edit);
        assert_eq!(tree(&f.options.input_root), before_input);
        assert_eq!(tree(&f.temp.path().join("publish")), before_publish);
        assert_eq!(
            fs::metadata(&source_path).unwrap().modified().unwrap(),
            source_mtime
        );
    }
}

#[test]
fn conversion_date_is_required_per_document_even_when_only_a_story_contains_it() {
    for story in ["word/document.xml", "word/header1.xml", "word/footer1.xml"] {
        for dry_run in [false, true] {
            let mut f = Fixture::new(SCHEMA_VERSION);
            f.options.dry_run = dry_run;
            f.date_template(&[story]);
            let invalid = [
                None,
                Some("2023-02-29"),
                Some("1900-02-29"),
                Some("2024-04-31"),
                Some("2024-00-10"),
                Some("2024-01-00"),
                Some("2024-2-09"),
                Some("2024-02-9"),
                Some("+2024-02-09"),
                Some("2024-02-09T00:00:00Z"),
                Some("not-a-date"),
                Some(""),
                Some("\"\""),
                Some("' '"),
            ];
            fs::remove_file(f.options.input_root.join("policy.md")).unwrap();
            for (i, date) in invalid.iter().enumerate() {
                let source = date.map_or_else(
                    || SOURCE.to_owned(),
                    |date| SOURCE.replacen("title:", &format!("last_modified: {date}\ntitle:"), 1),
                );
                fs::write(f.options.input_root.join(format!("bad-{i:02}.md")), source).unwrap();
            }
            fs::write(
                f.options.input_root.join("valid.md"),
                SOURCE.replacen("title:", "last_modified: 2024-02-29\ntitle:", 1),
            )
            .unwrap();
            let before_edit = tree(&f.temp.path().join("edit"));
            let before_input = tree(&f.options.input_root);
            let mut word = FakeWord::default();
            let manifest = ConversionPlan::prepare(&f.options)
                .unwrap()
                .run(&mut word)
                .unwrap();
            assert!(manifest.has_failures());
            assert_eq!(manifest.results.len(), invalid.len() + 1);
            for result in &manifest.results[..invalid.len()] {
                assert!(matches!(result.status, ConversionStatus::Failed));
                assert_eq!(result.error.as_deref(), Some("conversion template {DATE} requires last_modified as a real calendar date in YYYY-MM-DD format"));
                assert!(
                    result.docx.is_none() && result.pdf.is_none() && result.word_version.is_none()
                );
                assert!(!f
                    .options
                    .output_root
                    .join(result.input.with_extension("docx"))
                    .exists());
            }
            assert!(manifest.results.last().unwrap().error.is_none());
            assert_eq!(word.paths.len(), usize::from(!dry_run));
            if dry_run {
                assert!(!f.options.output_root.exists());
            }
            assert_eq!(tree(&f.temp.path().join("edit")), before_edit);
            assert_eq!(tree(&f.options.input_root), before_input);
            assert!(tree(&f.temp.path().join("publish")).is_empty());
        }
    }
}

#[test]
fn conversion_without_date_token_accepts_missing_invalid_and_empty_last_modified() {
    for value in [
        None,
        Some("2023-02-29"),
        Some("not-a-date"),
        Some(""),
        Some("\"\""),
        Some("''"),
    ] {
        let mut f = Fixture::new(SCHEMA_VERSION);
        f.dry();
        let source = value.map_or_else(
            || SOURCE.to_owned(),
            |value| SOURCE.replacen("title:", &format!("last_modified: {value}\ntitle:"), 1),
        );
        fs::write(f.options.input_root.join("policy.md"), source).unwrap();
        let before = tree(f.temp.path());
        let manifest = ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut FakeWord::default())
            .unwrap();
        assert!(!manifest.has_failures(), "{value:?}");
        assert_eq!(tree(f.temp.path()), before);
    }
}

#[test]
fn release_assembler_does_not_alias_or_require_conversion_date() {
    let f = Fixture::new(SCHEMA_VERSION);
    f.date_template(&["word/document.xml", "word/header1.xml", "word/footer1.xml"]);
    let template = f.temp.path().join("edit/template.docx");
    let output = f.temp.path().join("release-assembly.docx");
    for extra in [
        "",
        "last_modified: 2024-02-29\n",
        "last_modified: not-a-date\n",
    ] {
        let source = SOURCE.replacen("title:", &format!("{extra}title:"), 1);
        dms_core::assemble_markdown_docx(&template, &source, &output, None).unwrap();
        assert!(package_text(&output, "word/header1.xml").contains("{DATE}"));
    }
    let source = SOURCE.replacen("title:", "date: Existing optional value\ntitle:", 1);
    dms_core::assemble_markdown_docx(&template, &source, &output, None).unwrap();
    assert!(package_text(&output, "word/header1.xml").contains("Existing optional value"));
    assert!(dms_core::parse_markdown_frontmatter(&SOURCE.replacen(
        "title:",
        "last_modified: \"\"\ntitle:",
        1
    ))
    .is_err());
}

#[test]
fn retained_pair_and_manifest_hash_the_exact_saved_word_input() {
    let f = Fixture::new(SCHEMA_VERSION);
    let before_edit = tree(&f.temp.path().join("edit"));
    let before_source = fs::read(f.options.input_root.join("policy.md")).unwrap();
    let mut word = FakeWord::default();
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(!manifest.has_failures());
    assert_eq!(word.paths, vec![f.options.output_root.join("policy.docx")]);
    let result = &manifest.results[0];
    assert_eq!(
        result.input_sha256.as_deref(),
        Some(digest(&before_source).as_str())
    );
    assert!(result.metadata.as_ref().unwrap().document_number.is_none());
    for artifact in [result.docx.as_ref().unwrap(), result.pdf.as_ref().unwrap()] {
        assert_eq!(
            artifact.sha256,
            digest(&fs::read(f.options.output_root.join(&artifact.relative_path)).unwrap())
        );
    }
    let mut docx = ZipArchive::new(fs::File::open(&word.paths[0]).unwrap()).unwrap();
    assert!(docx.by_name("docProps/synthetic-word-save.xml").is_ok());
    let recorded: Value =
        serde_json::from_slice(&fs::read(f.options.output_root.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(recorded, serde_json::to_value(&manifest).unwrap());
    assert_eq!(tree(&f.temp.path().join("edit")), before_edit);
    assert!(tree(&f.temp.path().join("publish")).is_empty());
    assert_eq!(
        fs::read(f.options.input_root.join("policy.md")).unwrap(),
        before_source
    );
    assert!(ConversionPlan::prepare(&f.options).is_err());
}
#[test]
fn mixed_results_continue_sequentially_and_do_not_leak_source_lines() {
    let f = Fixture::new(19);
    fs::rename(
        f.options.input_root.join("policy.md"),
        f.options.input_root.join("z.md"),
    )
    .unwrap();
    fs::write(f.options.input_root.join("a.md"), SOURCE).unwrap();
    fs::write(
        f.options.input_root.join("bad.md"),
        "---\nSECRET SOURCE LINE\n---\n",
    )
    .unwrap();
    let mut word = FakeWord::default();
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(manifest.has_failures());
    assert_eq!(manifest.results.len(), 3);
    assert_eq!(
        word.paths,
        vec![
            f.options.output_root.join("a.docx"),
            f.options.output_root.join("z.docx")
        ]
    );
    assert!(matches!(
        manifest.results[1].status,
        ConversionStatus::Failed
    ));
    assert!(!f.options.output_root.join("bad.docx").exists());
    assert!(!serde_json::to_string(&manifest)
        .unwrap()
        .contains("SECRET SOURCE LINE"));
}
#[test]
fn failed_word_invalid_pdf_missing_pdf_and_invalid_docx_never_leave_success_pairs() {
    for failure in ["word", "pdf", "missing", "docx"] {
        let f = Fixture::new(18);
        let mut word = FakeWord {
            failure: Some(failure),
            ..Default::default()
        };
        let m = ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut word)
            .unwrap();
        assert!(m.has_failures(), "{failure}");
        assert!(m.results[0].docx.is_none() && m.results[0].pdf.is_none());
        assert!(!f.options.output_root.join("policy.docx").exists());
        assert!(!f.options.output_root.join("policy.pdf").exists());
        assert!(m.results[0].retained_diagnostics.is_empty());
    }
}
#[test]
fn missing_word_stops_before_output_creation() {
    let f = Fixture::new(19);
    let mut word = FakeWord {
        failure: Some("unavailable"),
        ..Default::default()
    };
    assert!(ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .is_err());
    assert!(!f.options.output_root.exists());
    assert!(word.paths.is_empty());
}
#[test]
fn source_asset_and_template_digests_describe_consumed_bytes_even_if_inputs_change() {
    let f = Fixture::new(19);
    let source = f.options.input_root.join("policy.md");
    let asset = f.options.input_root.join("image.png");
    fs::write(&asset, IMAGE).unwrap();
    fs::write(&source, format!("{SOURCE}\n![Synthetic](image.png)\n")).unwrap();
    let source_digest = digest(&fs::read(&source).unwrap());
    let plan = ConversionPlan::prepare(&f.options).unwrap();
    fs::write(
        f.temp.path().join("edit/template.docx"),
        "changed after prepare",
    )
    .unwrap();
    let mut word = FakeWord {
        changed_source: Some(source),
        changed_asset: Some(asset),
        ..Default::default()
    };
    let m = plan.run(&mut word).unwrap();
    assert!(!m.has_failures());
    assert_eq!(m.template.sha256, digest(TEMPLATE));
    assert_eq!(
        m.results[0].input_sha256.as_deref(),
        Some(source_digest.as_str())
    );
    assert_eq!(m.results[0].assets.len(), 1);
    assert_eq!(m.results[0].assets[0].sha256, digest(IMAGE));
    let mut zip =
        ZipArchive::new(fs::File::open(f.options.output_root.join("policy.docx")).unwrap())
            .unwrap();
    let mut embedded = Vec::new();
    zip.by_name("word/media/dms-image-1.png")
        .unwrap()
        .read_to_end(&mut embedded)
        .unwrap();
    assert_eq!(embedded, IMAGE);
}
#[test]
fn strict_metadata_rejects_missing_invalid_disabled_and_display_label_classification() {
    for source in [
        SOURCE.replace("title: Synthetic policy\n", ""),
        SOURCE.replace("3.0", "V3.0"),
        SOURCE.replace("3.0", "+3.0"),
        SOURCE.replace("internal", "Internal label"),
        SOURCE.replace("version: 3.0\n", ""),
        SOURCE.replace("confidentiality: internal\n", ""),
        SOURCE.replace("title: Synthetic policy", "title: ''"),
    ] {
        let mut f = Fixture::new(19);
        f.dry();
        fs::write(f.options.input_root.join("policy.md"), source).unwrap();
        assert!(ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut FakeWord::default())
            .unwrap()
            .has_failures());
    }
    let mut f = Fixture::new(19);
    f.dry();
    f.config(|v| v["confidentiality_types"]["internal"]["enabled"] = json!(false));
    assert!(ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut FakeWord::default())
        .unwrap()
        .has_failures());
}
#[test]
fn preflight_rejects_schema_template_digest_contract_and_root_mismatches() {
    for change in ["schema", "digest", "contract", "root", "locator", "label"] {
        let f = Fixture::new(19);
        f.config(|v| match change {
            "schema" => v["schema_version"] = json!(17),
            "digest" => v["markdown_template"]["sha256"] = json!("0".repeat(64)),
            "contract" => v["markdown_template"]["contract_version"] = json!(2),
            "root" => v["edit_root"] = json!(f.options.input_root),
            "locator" => v["markdown_template"]["relative_path"] = json!("../template.docx"),
            _ => v["confidentiality_types"]["internal"]["id"] = json!("other"),
        });
        assert!(ConversionPlan::prepare(&f.options).is_err(), "{change}");
        assert!(!f.options.output_root.exists());
    }
}
#[test]
fn selection_is_explicit_and_excludes_asset_and_metadata_subtrees() {
    let mut f = Fixture::new(19);
    f.dry();
    for sub in ["nested", "assets", ".dms"] {
        fs::create_dir(f.options.input_root.join(sub)).unwrap();
        fs::write(f.options.input_root.join(sub).join("readme.md"), SOURCE).unwrap();
    }
    assert_eq!(
        ConversionPlan::prepare(&f.options)
            .unwrap()
            .selected_paths(),
        vec![PathBuf::from("policy.md")]
    );
    f.options.recursive = true;
    assert_eq!(
        ConversionPlan::prepare(&f.options)
            .unwrap()
            .selected_paths(),
        vec![
            PathBuf::from("nested/readme.md"),
            PathBuf::from("policy.md")
        ]
    );
}
#[test]
fn empty_input_and_portable_file_directory_collisions_fail_before_output() {
    for name in [
        "POLICY.MD",
        "policy.MD",
        "CON.md",
        "bad:name.md",
        "policy.docx/other.md",
        "manifest.json/other.md",
    ] {
        let mut f = Fixture::new(19);
        f.options.recursive = true;
        let p = f.options.input_root.join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, SOURCE).unwrap();
        assert!(ConversionPlan::prepare(&f.options).is_err(), "{name}");
        assert!(!f.options.output_root.exists());
    }
    let f = Fixture::new(19);
    fs::remove_file(f.options.input_root.join("policy.md")).unwrap();
    assert!(ConversionPlan::prepare(&f.options).is_err());
}
#[test]
fn output_must_be_new_disjoint_and_not_parent_traversing() {
    for target in [
        "input/new",
        "edit/new",
        "publish/new",
        "input",
        "output/../new",
    ] {
        let mut f = Fixture::new(19);
        f.options.output_root = f.temp.path().join(target);
        assert!(ConversionPlan::prepare(&f.options).is_err(), "{target}");
    }
    let f = Fixture::new(19);
    let plan = ConversionPlan::prepare(&f.options).unwrap();
    fs::create_dir(&f.options.output_root).unwrap();
    fs::write(f.options.output_root.join("keep"), "user content").unwrap();
    assert!(plan.run(&mut FakeWord::default()).is_err());
    assert_eq!(
        fs::read(f.options.output_root.join("keep")).unwrap(),
        b"user content"
    );
}
#[cfg(unix)]
#[test]
fn symlink_escapes_and_output_aliases_are_rejected() {
    use std::os::unix::fs::symlink;
    let mut f = Fixture::new(19);
    symlink(&f.options.input_root, f.temp.path().join("alias")).unwrap();
    f.options.output_root = f.temp.path().join("alias/new");
    assert!(ConversionPlan::prepare(&f.options).is_err());
    let f = Fixture::new(19);
    symlink(
        f.temp.path().join("edit/template.docx"),
        f.options.input_root.join("escape.md"),
    )
    .unwrap();
    assert!(ConversionPlan::prepare(&f.options).is_err());
    let f = Fixture::new(19);
    let plan = ConversionPlan::prepare(&f.options).unwrap();
    fs::remove_file(f.options.input_root.join("policy.md")).unwrap();
    symlink(
        f.temp.path().join("edit/template.docx"),
        f.options.input_root.join("policy.md"),
    )
    .unwrap();
    assert!(plan.run(&mut FakeWord::default()).unwrap().has_failures());
}

#[test]
fn optional_variables_cannot_override_reserved_conversion_fields_or_invent_a_number() {
    let f = Fixture::new(19);
    let source = SOURCE.replacen("---\n#", "TITLE: WrongTitle\nVERSION: 99.99\nCONFIDENTIALITY: WrongLabel\nDOCUMENT_NUMBER: invented\nteam: OptionalValue\n---\n#", 1);
    fs::write(
        f.options.input_root.join("policy.md"),
        format!("{source}\n{{TEAM}}\n"),
    )
    .unwrap();
    let mut word = FakeWord::default();
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(!manifest.has_failures());
    assert!(manifest.results[0]
        .metadata
        .as_ref()
        .unwrap()
        .document_number
        .is_none());
    let mut archive = ZipArchive::new(fs::File::open(&word.paths[0]).unwrap()).unwrap();
    let mut document = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut document)
        .unwrap();
    assert!(document.contains("OptionalValue"));
    assert!(!document.contains("WrongTitle") && !document.contains("invented"));
}

#[test]
fn newly_occupied_destination_is_not_overwritten_or_cleaned_up() {
    struct OccupyingWord(FakeWord);
    impl ConversionExporter for OccupyingWord {
        fn preflight(&self) -> Result<(), String> {
            Ok(())
        }
        fn export(
            &mut self,
            docx: &Path,
            pdf: &Path,
            chrome: &ExportChrome,
        ) -> Result<String, String> {
            let result = self.0.export(docx, pdf, chrome)?;
            if docx.file_stem().unwrap() == "a" {
                fs::write(docx.parent().unwrap().join("b.docx"), b"operator file").unwrap();
            }
            Ok(result)
        }
    }
    let f = Fixture::new(19);
    fs::remove_file(f.options.input_root.join("policy.md")).unwrap();
    for name in ["a.md", "b.md", "c.md"] {
        fs::write(f.options.input_root.join(name), SOURCE).unwrap();
    }
    let mut word = OccupyingWord(FakeWord::default());
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .unwrap();
    assert!(manifest.has_failures());
    assert!(matches!(
        manifest.results[1].status,
        ConversionStatus::Failed
    ));
    assert!(matches!(
        manifest.results[2].status,
        ConversionStatus::Succeeded
    ));
    assert_eq!(
        fs::read(f.options.output_root.join("b.docx")).unwrap(),
        b"operator file"
    );
    assert!(!f.options.output_root.join("b.pdf").exists());
}

#[cfg(unix)]
#[test]
fn changed_output_parent_is_rejected_before_any_creation() {
    use std::os::unix::fs::symlink;
    let mut f = Fixture::new(19);
    let parent = f.temp.path().join("parent");
    fs::create_dir(&parent).unwrap();
    f.options.output_root = parent.join("new");
    let plan = ConversionPlan::prepare(&f.options).unwrap();
    fs::remove_dir(&parent).unwrap();
    symlink(&f.options.input_root, &parent).unwrap();
    assert!(plan.run(&mut FakeWord::default()).is_err());
    assert!(!f.options.input_root.join("new").exists());
}

#[test]
fn pair_validation_rejects_incomplete_pdf_wrong_docx_roots_and_split_placeholders() {
    let f = Fixture::new(19);
    let manifest = ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut FakeWord::default())
        .unwrap();
    assert!(!manifest.has_failures());
    let docx = fs::read(f.options.output_root.join("policy.docx")).unwrap();
    let pdf = fs::read(f.options.output_root.join("policy.pdf")).unwrap();
    assert!(dms_core::validate_conversion_pair(&docx, &pdf[..pdf.len() - 7]).is_err());
    for document in [
        "<not-a-word-document/>",
        "<w:document xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main'/>",
        "<w:document xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main'><w:body><w:p><w:r><w:t>{TI</w:t></w:r><w:r><w:t>TLE}</w:t></w:r></w:p></w:body></w:document>",
    ] {
        let mut input = ZipArchive::new(std::io::Cursor::new(&docx)).unwrap();
        let mut out = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for i in 0..input.len() {
            let mut part = input.by_index(i).unwrap();
            let mut bytes = Vec::new(); part.read_to_end(&mut bytes).unwrap();
            out.start_file(part.name(), SimpleFileOptions::default()).unwrap();
            out.write_all(if part.name() == "word/document.xml" { document.as_bytes() } else { &bytes }).unwrap();
        }
        let invalid = out.finish().unwrap().into_inner();
        assert!(dms_core::validate_conversion_pair(&invalid, &pdf).is_err());
    }
}

#[cfg(unix)]
#[test]
fn replacing_a_preflighted_input_root_with_an_alias_never_reads_workspace_sources() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new(19);
    let plan = ConversionPlan::prepare(&f.options).unwrap();
    fs::rename(&f.options.input_root, f.temp.path().join("original-input")).unwrap();
    fs::write(f.temp.path().join("edit/policy.md"), SOURCE).unwrap();
    symlink(f.temp.path().join("edit"), &f.options.input_root).unwrap();
    let mut word = FakeWord::default();
    let manifest = plan.run(&mut word).unwrap();
    assert!(manifest.has_failures());
    assert!(manifest.results[0].input_sha256.is_none());
    assert!(word.paths.is_empty());
}

#[cfg(unix)]
#[test]
fn replacing_output_root_cannot_redirect_artifacts_manifest_or_cleanup() {
    use std::os::unix::fs::symlink;
    struct RedirectingWord {
        output: PathBuf,
        input: PathBuf,
    }
    impl ConversionExporter for RedirectingWord {
        fn preflight(&self) -> Result<(), String> {
            Ok(())
        }
        fn export(&mut self, _: &Path, _: &Path, _: &ExportChrome) -> Result<String, String> {
            fs::rename(&self.output, self.output.with_file_name("moved-output")).unwrap();
            symlink(&self.input, &self.output).unwrap();
            Err("output tree moved".to_owned())
        }
    }
    let f = Fixture::new(19);
    fs::write(f.options.input_root.join("policy.docx"), b"operator DOCX").unwrap();
    fs::write(f.options.input_root.join("policy.pdf"), b"operator PDF").unwrap();
    let before = tree(&f.options.input_root);
    let mut word = RedirectingWord {
        output: f.options.output_root.clone(),
        input: f.options.input_root.clone(),
    };
    assert!(ConversionPlan::prepare(&f.options)
        .unwrap()
        .run(&mut word)
        .is_err());
    assert_eq!(tree(&f.options.input_root), before);
    assert!(!f.options.input_root.join("manifest.json").exists());
}

#[cfg(unix)]
#[test]
fn review_config_symlink_is_rejected_before_resolving_its_target() {
    use std::os::unix::fs::symlink;
    let mut f = Fixture::new(19);
    let alias = f.temp.path().join("workspace-alias.json");
    symlink(&f.options.workspace_config, &alias).unwrap();
    let before = fs::read(&f.options.workspace_config).unwrap();
    f.options.workspace_config = alias;
    assert!(ConversionPlan::prepare(&f.options).is_err());
    assert_eq!(fs::read(&f.options.workspace_config).unwrap(), before);
    assert!(!f.options.output_root.exists());
}

#[test]
fn review_whitespace_only_number_is_absent_but_nonblank_numbers_are_preserved() {
    for (number, expected) in [("  ", None), (" DOC-001 ", Some(" DOC-001 "))] {
        let mut f = Fixture::new(19);
        f.dry();
        let source = SOURCE.replacen(
            "version:",
            &format!("document_number: '{number}'\nversion:"),
            1,
        );
        fs::write(f.options.input_root.join("policy.md"), source).unwrap();
        let manifest = ConversionPlan::prepare(&f.options)
            .unwrap()
            .run(&mut FakeWord::default())
            .unwrap();
        assert!(!manifest.has_failures());
        assert_eq!(
            manifest.results[0]
                .metadata
                .as_ref()
                .unwrap()
                .document_number
                .as_deref(),
            expected
        );
    }
}
