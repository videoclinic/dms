use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use dms_core::{
    assemble_markdown_docx, assemble_markdown_docx_with_options, validate_markdown_template,
    DmsError, MarkdownAssetContext, MarkdownAssetDigest, MarkdownTemplateValidationState,
    Workspace, MARKDOWN_TEMPLATE_CONTRACT_VERSION, SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

const FIXTURE: &[u8] = include_bytes!("fixtures/markdown-template.docx");
// Synthetic solid-color images; independently decoded with FFmpeg.
const WIDE_PNG: &[u8] = include_bytes!("fixtures/markdown-wide.png");
const SMALL_PNG: &[u8] = include_bytes!("fixtures/markdown-small.png");
const PHOTO_JPEG: &[u8] = include_bytes!("fixtures/markdown-photo.jpeg");
const SECOND_JPEG: &[u8] = include_bytes!("fixtures/markdown-second.jpeg");

struct FixtureWorkspace {
    _temp: TempDir,
    workspace: Workspace,
    template_path: PathBuf,
}

impl FixtureWorkspace {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let edit_root = temp.path().join("edit");
        let publish_root = temp.path().join("publish");
        fs::create_dir_all(edit_root.join("Configuration")).unwrap();
        let template_path = edit_root.join("Configuration/Markdown-template.docx");
        fs::write(&template_path, FIXTURE).unwrap();
        let workspace = Workspace::init(&edit_root, &publish_root).unwrap();
        Self {
            _temp: temp,
            workspace,
            template_path,
        }
    }
}

#[test]
fn template_asset_is_stable_persisted_revalidated_and_excluded_from_documents() {
    let mut fixture = FixtureWorkspace::new();
    let imported = fixture
        .workspace
        .import_markdown_template(&fixture.template_path)
        .unwrap();
    assert_eq!(
        imported.relative_path,
        Path::new("Configuration/Markdown-template.docx")
    );
    assert_eq!(
        imported.contract_version,
        MARKDOWN_TEMPLATE_CONTRACT_VERSION
    );
    assert_eq!(imported.sha256.len(), 64);
    assert_eq!(
        fixture
            .workspace
            .markdown_template_validation()
            .unwrap()
            .state,
        MarkdownTemplateValidationState::Valid
    );
    assert!(matches!(
        fixture.workspace.add_document(&fixture.template_path),
        Err(DmsError::TemplateLifecycleExcluded(_))
    ));
    let other_path = fixture.workspace.edit_root.join("Other.md");
    fs::write(&other_path, "uncontrolled body").unwrap();
    let other = fixture.workspace.add_document(&other_path).unwrap();
    fixture.workspace.unregister_document(other.id).unwrap();
    assert!(matches!(
        fixture.workspace.reassociate_document(
            other.id,
            &fixture.template_path,
            &dms_core::MutationPrincipal::local_os_user("test-operator")
        ),
        Err(DmsError::TemplateLifecycleExcluded(_))
    ));
    let listing = fixture
        .workspace
        .library_folder(Path::new("Configuration"))
        .unwrap();
    assert!(listing.entries.is_empty());

    fixture.workspace.save().unwrap();
    let reopened = Workspace::open(&fixture.workspace.edit_root).unwrap();
    assert_eq!(reopened.markdown_template().unwrap(), &imported);

    fs::OpenOptions::new()
        .append(true)
        .open(&fixture.template_path)
        .unwrap()
        .write_all(b"changed")
        .unwrap();
    assert_eq!(
        fixture
            .workspace
            .markdown_template_validation()
            .unwrap()
            .state,
        MarkdownTemplateValidationState::Changed
    );
    let replaced = fixture
        .workspace
        .import_markdown_template(&fixture.template_path)
        .unwrap();
    assert_eq!(replaced.id, imported.id);
    assert_ne!(replaced.sha256, imported.sha256);
    assert_eq!(
        fixture
            .workspace
            .markdown_template_validation()
            .unwrap()
            .state,
        MarkdownTemplateValidationState::Valid
    );
    assert_eq!(
        fixture.workspace.remove_markdown_template().unwrap().id,
        imported.id
    );
    assert!(fixture.workspace.markdown_template().is_none());
}

#[test]
fn template_import_refuses_outside_symlink_and_registered_document_paths() {
    let outside = tempfile::tempdir().unwrap();
    let outside_template = outside.path().join("outside.docx");
    fs::write(&outside_template, FIXTURE).unwrap();
    let mut fixture = FixtureWorkspace::new();
    assert!(matches!(
        fixture
            .workspace
            .import_markdown_template(&outside_template),
        Err(DmsError::OutsideEditRoot(_))
    ));

    let registered = fixture
        .workspace
        .add_document(&fixture.template_path)
        .unwrap();
    assert!(matches!(
        fixture
            .workspace
            .import_markdown_template(&fixture.template_path),
        Err(DmsError::TemplateIsControlledDocument(_))
    ));
    assert_eq!(
        fixture
            .workspace
            .document(registered.id)
            .unwrap()
            .relative_path,
        Path::new("Configuration/Markdown-template.docx")
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let mut fixture = FixtureWorkspace::new();
        let symlink_path = fixture
            .workspace
            .edit_root
            .join("Configuration/symlink.docx");
        symlink(&fixture.template_path, &symlink_path).unwrap();
        assert!(matches!(
            fixture.workspace.import_markdown_template(&symlink_path),
            Err(DmsError::MarkdownTemplateSymlink(_))
        ));

        let mut swapped = FixtureWorkspace::new();
        swapped
            .workspace
            .import_markdown_template(&swapped.template_path)
            .unwrap();
        fs::remove_file(&swapped.template_path).unwrap();
        symlink(&outside_template, &swapped.template_path).unwrap();
        assert_eq!(
            swapped
                .workspace
                .markdown_template_validation()
                .unwrap()
                .state,
            MarkdownTemplateValidationState::Invalid
        );
    }
}

#[test]
fn schema_v13_migrates_to_v14_with_an_empty_template_record_and_backup() {
    let fixture = FixtureWorkspace::new();
    fixture.workspace.save().unwrap();
    let metadata_path = fixture.workspace.edit_root.join(".dms/workspace.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
    metadata["schema_version"] = serde_json::Value::from(13);
    metadata
        .as_object_mut()
        .unwrap()
        .remove("markdown_template");
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();

    let migrated = Workspace::open(&fixture.workspace.edit_root).unwrap();
    assert_eq!(migrated.schema_version, SCHEMA_VERSION);
    assert!(migrated.markdown_template().is_none());
    assert!(fixture
        .workspace
        .edit_root
        .join(".dms/workspace.v13.json.bak")
        .is_file());
}

#[test]
fn validator_rejects_missing_or_duplicate_body_prototypes() {
    let temp = tempfile::tempdir().unwrap();
    let valid = temp.path().join("valid.docx");
    fs::write(&valid, FIXTURE).unwrap();
    let contract = validate_markdown_template(&valid).unwrap();
    assert_eq!(
        contract.contract_version,
        MARKDOWN_TEMPLATE_CONTRACT_VERSION
    );
    assert!(contract
        .package_parts
        .contains(&"word/header1.xml".to_owned()));

    let missing = temp.path().join("missing.docx");
    rewrite_xml_part(&valid, &missing, "word/document.xml", |xml| {
        xml.replace("{Heading 4}", "missing")
    });
    assert!(matches!(
        validate_markdown_template(&missing),
        Err(DmsError::InvalidMarkdownTemplate(message)) if message.contains("{Heading 4}")
    ));

    let duplicate = temp.path().join("duplicate.docx");
    rewrite_xml_part(&valid, &duplicate, "word/document.xml", |xml| {
        xml.replace("{PARAGRAPH}", "{PARAGRAPH}{PARAGRAPH}")
    });
    assert!(matches!(
        validate_markdown_template(&duplicate),
        Err(DmsError::InvalidMarkdownTemplate(message)) if message.contains("exactly once")
    ));

    let extra_table_cell = temp.path().join("extra-table-cell.docx");
    rewrite_xml_part(&valid, &extra_table_cell, "word/document.xml", |xml| {
        let table_start = xml.find("<w:tbl").unwrap();
        let row_end = table_start + xml[table_start..].find("</w:tr>").unwrap();
        let cell_start = table_start + xml[table_start..].find("<w:tc").unwrap();
        let cell_end = cell_start + xml[cell_start..].find("</w:tc>").unwrap() + "</w:tc>".len();
        let extra_cell = xml[cell_start..cell_end].replace("{TABLE COLUMN 1}", "extra");
        format!("{}{}{}", &xml[..row_end], extra_cell, &xml[row_end..])
    });
    assert!(matches!(
        validate_markdown_template(&extra_table_cell),
        Err(DmsError::InvalidMarkdownTemplate(message)) if message.contains("exactly two cells")
    ));

    let missing_property = temp.path().join("missing-property.docx");
    rewrite_xml_part(&valid, &missing_property, "docProps/custom.xml", |xml| {
        xml.replace("DMS_DOCUMENT_NUMBER", "REMOVED_DOCUMENT_NUMBER")
    });
    assert!(matches!(
        validate_markdown_template(&missing_property),
        Err(DmsError::InvalidMarkdownTemplate(message)) if message.contains("DMS_DOCUMENT_NUMBER")
    ));

    let swapped_property = temp.path().join("swapped-property.docx");
    rewrite_xml_part(&valid, &swapped_property, "docProps/custom.xml", |xml| {
        xml.replace("{TITLE}", "{SWAP}")
            .replace("{VERSION}", "{TITLE}")
            .replace("{SWAP}", "{VERSION}")
    });
    assert!(matches!(
        validate_markdown_template(&swapped_property),
        Err(DmsError::InvalidMarkdownTemplate(message))
            if message.contains("DMS_TITLE") && message.contains("{TITLE}")
    ));
}

#[test]
fn assembler_renders_supported_markdown_and_preserves_non_body_package_parts() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    let first = temp.path().join("first.docx");
    let second = temp.path().join("second.docx");
    fs::write(&template, FIXTURE).unwrap();
    let markdown = r#"---
title: Employee handbook
document_number: HB-001
version: 1.0
confidentiality: Internal
author: Ada Lovelace
department: Quality
---
# Handbook
## Scope
#### Details

Normal **bold** and *italic* text with [portal](https://example.test) and `inline code`.

- first bullet
- second bullet

3. first number
4. second number

A separator between ordered lists.

1. restarted number
2. restarted again

```rust
let value = 1;
```

| Name | Value |
| --- | --- |
| Alpha | One |
| Beta | Two |
"#;

    assemble_markdown_docx(&template, markdown, &first, None).unwrap();
    assemble_markdown_docx(&template, markdown, &second, None).unwrap();
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    validate_markdown_template_output(&first);

    let template_parts = zip_entries(&template);
    let output_parts = zip_entries(&first);
    let changed_parts = [
        "[Content_Types].xml",
        "word/document.xml",
        "word/_rels/document.xml.rels",
        "word/numbering.xml",
    ];
    for (name, bytes) in &template_parts {
        if !changed_parts.contains(&name.as_str()) {
            assert_eq!(
                output_parts.get(name),
                Some(bytes),
                "changed package part {name}"
            );
        }
    }
    for name in output_parts.keys() {
        assert!(
            template_parts.contains_key(name) || name == "word/numbering.xml",
            "unexpected output package part {name}"
        );
    }

    let document = String::from_utf8(output_parts["word/document.xml"].clone()).unwrap();
    for prototype in [
        "{Heading 1}",
        "{Heading 2}",
        "{Heading 3}",
        "{Heading 4}",
        "{PARAGRAPH}",
        "{BULLET LIST}",
        "{TABLE COLUMN 1}",
        "{TABLE COLUMN 2}",
    ] {
        assert!(!document.contains(prototype));
    }
    for leaked in [
        "---",
        "title: Employee handbook",
        "document_number: HB-001",
        "version: 1.0",
        "confidentiality: Internal",
        "author: Ada Lovelace",
        "Employee handbook",
        "HB-001",
        "Ada Lovelace",
    ] {
        assert!(
            !document.contains(leaked),
            "frontmatter leaked into body: {leaked:?}"
        );
    }
    for text in [
        "Handbook",
        "Scope",
        "Details",
        "Normal ",
        "first bullet",
        "first number",
        "let value = 1;",
        "Alpha",
        "Beta",
        "https://example.test",
    ] {
        assert!(document.contains(text), "missing rendered text {text:?}");
    }
    for markup in [
        "<w:b/>",
        "<w:i/>",
        "Courier New",
        "<w:u w:val=\"single\"/>",
        "<w:pStyle w:val=\"Heading1\"/>",
        "<w:sectPr>",
    ] {
        assert!(document.contains(markup), "missing OOXML {markup:?}");
    }
    let ordered_start = document[..document.find("first number").unwrap()]
        .rfind("<w:p>")
        .unwrap();
    let ordered_end =
        ordered_start + document[ordered_start..].find("</w:p>").unwrap() + "</w:p>".len();
    assert!(
        document[ordered_start..ordered_end].contains("<w:numPr>"),
        "ordered Markdown must use native Word numbering"
    );
    assert!(
        !["1. ", "3. "]
            .iter()
            .any(|prefix| document[ordered_start..ordered_end].contains(prefix)),
        "the native Word list owns the visible ordinal"
    );
    let numbering = std::str::from_utf8(&output_parts["word/numbering.xml"]).unwrap();
    assert!(numbering.contains("<w:numFmt w:val=\"decimal\"/>"));
    assert!(numbering.contains("<w:lvlText w:val=\"%1.\"/>"));
    assert_eq!(numbering.matches("<w:num ").count(), 3);
    assert!(numbering.contains("<w:startOverride w:val=\"3\"/>"));
    assert!(numbering.contains("<w:startOverride w:val=\"1\"/>"));
    let relationships = std::str::from_utf8(&output_parts["word/_rels/document.xml.rels"]).unwrap();
    assert!(relationships.contains("relationships/numbering"));
    let content_types = std::str::from_utf8(&output_parts["[Content_Types].xml"]).unwrap();
    assert!(content_types.contains("wordprocessingml.numbering+xml"));
}

#[test]
fn assembler_renders_unordered_lists_with_native_bullet_numbering() {
    let temp = tempfile::tempdir().unwrap();
    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n- first **bullet**\n- second *bullet* with `code`\n";
    let source = temp.path().join("source.md");
    fs::write(&source, markdown).unwrap();
    for existing_numbering in [false, true] {
        let template = temp
            .path()
            .join(format!("template-{existing_numbering}.docx"));
        let output = temp
            .path()
            .join(format!("output-{existing_numbering}.docx"));
        let repeat = temp
            .path()
            .join(format!("repeat-{existing_numbering}.docx"));
        let mut parts = zip_entries_from_bytes(FIXTURE);
        let document = String::from_utf8(parts["word/document.xml"].clone()).unwrap();
        parts.insert(
            "word/document.xml".to_owned(),
            document.replace(
                "<w:p><w:r><w:t>{PARAGRAPH}",
                "<w:p><w:pPr><w:pStyle w:val=\"Normal\"/><w:keepNext/></w:pPr><w:r><w:t>{PARAGRAPH}",
            ).into_bytes(),
        );
        let existing_abstract = "<w:abstractNum w:abstractNumId=\"40\"><w:multiLevelType w:val=\"singleLevel\"/><w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/><w:lvlText w:val=\"%1)\"/></w:lvl></w:abstractNum>";
        let existing_num = "<w:num w:numId=\"70\"><w:abstractNumId w:val=\"40\"/></w:num>";
        if existing_numbering {
            parts.insert("word/numbering.xml".to_owned(), format!(
                "<w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">{existing_abstract}{existing_num}<w:numIdMac w:val=\"70\"/></w:numbering>"
            ).into_bytes());
            for (name, closing, addition) in [
                ("word/_rels/document.xml.rels", "</Relationships>", "<Relationship Id=\"rId71\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" Target=\"numbering.xml\"/>"),
                ("[Content_Types].xml", "</Types>", "<Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>"),
            ] {
                let xml = String::from_utf8(parts[name].clone()).unwrap();
                parts.insert(name.to_owned(), xml.replace(closing, &format!("{addition}{closing}")).into_bytes());
            }
        }
        let mut writer = ZipWriter::new(fs::File::create(&template).unwrap());
        for (name, bytes) in &parts {
            writer
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
        let template_bytes = fs::read(&template).unwrap();

        assemble_markdown_docx(&template, markdown, &output, None).unwrap();
        assemble_markdown_docx(&template, markdown, &repeat, None).unwrap();
        assert_eq!(fs::read(&output).unwrap(), fs::read(&repeat).unwrap());
        assert_eq!(fs::read(&template).unwrap(), template_bytes);
        assert_eq!(fs::read_to_string(&source).unwrap(), markdown);
        let generated = zip_entries(&output);
        let document = std::str::from_utf8(&generated["word/document.xml"]).unwrap();
        for text in ["first bullet", "second bullet with code"] {
            let paragraph = xml_elements(document, "w:p")
                .into_iter()
                .find(|paragraph| xml_text(paragraph) == text)
                .unwrap();
            assert_eq!(
                xml_elements(paragraph, "w:numPr").len(),
                1,
                "unordered Markdown must use native Word numbering"
            );
            assert_eq!(
                xml_attribute(xml_elements(paragraph, "w:pStyle")[0], "w:val"),
                "Normal"
            );
            assert!(paragraph.contains("<w:keepNext/>"));
            assert!(
                paragraph.find("<w:keepNext/>").unwrap() < paragraph.find("<w:numPr>").unwrap()
            );
            assert!(!paragraph.contains("ListParagraph"));
        }
        assert!(document.contains("<w:b/>"));
        assert!(document.contains("<w:i/>"));
        assert!(document.contains("Courier New"));
        let numbering = std::str::from_utf8(&generated["word/numbering.xml"]).unwrap();
        let first = resolved_list_numbering(document, numbering, "first bullet");
        let second = resolved_list_numbering(document, numbering, "second bullet with code");
        assert_eq!(first, second);
        assert_eq!(first.1, "bullet");
        assert_eq!(first.2, "•");
        assert_eq!(
            xml_elements(numbering, "w:num").len(),
            if existing_numbering { 2 } else { 1 }
        );
        if existing_numbering {
            assert!(numbering.contains(existing_abstract));
            assert!(numbering.contains(existing_num));
            assert!(first.0.parse::<u32>().unwrap() > 70);
            assert!(
                numbering.rfind("</w:abstractNum>").unwrap() < numbering.find("<w:num ").unwrap()
            );
            assert!(numbering.rfind("</w:num>").unwrap() < numbering.find("<w:numIdMac ").unwrap());
        }
        for (name, bytes) in &parts {
            if ![
                "word/document.xml",
                "word/numbering.xml",
                "word/_rels/document.xml.rels",
                "[Content_Types].xml",
            ]
            .contains(&name.as_str())
            {
                assert_eq!(&generated[name], bytes, "changed package part {name}");
            }
        }
        for (name, marker) in [
            ("word/_rels/document.xml.rels", "relationships/numbering"),
            ("[Content_Types].xml", "wordprocessingml.numbering+xml"),
        ] {
            assert_eq!(
                std::str::from_utf8(&generated[name])
                    .unwrap()
                    .matches(marker)
                    .count(),
                1
            );
        }
    }
}

#[test]
fn assembler_sets_one_native_list_reference_in_paragraph_properties() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    fs::write(&original, FIXTURE).unwrap();
    for properties in [
        "<w:pPr/>",
        "<w:pPr><w:pStyle w:val=\"Normal\"/><w:numPr><w:ilvl w:val=\"2\"/><w:numId w:val=\"99\"/></w:numPr><w:spacing w:after=\"120\"/></w:pPr>",
    ] {
        let template = temp.path().join("template.docx");
        rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
            xml.replace("<w:p><w:r><w:t>{PARAGRAPH}", &format!("<w:p>{properties}<w:r><w:t>{{PARAGRAPH}}"))
        });
        let output = temp.path().join("lists.docx");
        assemble_markdown_docx(&template,
            "---\nversion: 1.0\nconfidentiality: Internal\n---\n- bullet\n\n5. numbered\n",
            &output, None).unwrap();
        let parts = zip_entries(&output);
        let document = std::str::from_utf8(&parts["word/document.xml"]).unwrap();
        let numbering = std::str::from_utf8(&parts["word/numbering.xml"]).unwrap();
        for (text, expected) in [("bullet", "bullet"), ("numbered", "decimal")] {
            assert_eq!(resolved_list_numbering(document, numbering, text).1, expected);
        }
        assert!(!document.contains("w:val=\"99\""));
        if properties.contains("w:spacing") {
            assert_eq!(xml_elements(document, "w:spacing").len(), 2);
        }
    }
}

#[test]
fn assembler_keeps_native_bullet_and_decimal_lists_separate() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    let output = temp.path().join("mixed.docx");
    fs::write(&template, FIXTURE).unwrap();
    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n3. third\n4. fourth\n\n- bullet one\n- bullet two\n\n0. zero\n1. one\n\nSeparator.\n\n1. restarted\n2. next\n\n- other bullet\n";
    assemble_markdown_docx(&template, markdown, &output, None).unwrap();
    let parts = zip_entries(&output);
    let document = std::str::from_utf8(&parts["word/document.xml"]).unwrap();
    let numbering = std::str::from_utf8(&parts["word/numbering.xml"]).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    for (texts, format, marker, start) in [
        (vec!["third", "fourth"], "decimal", "%1.", "3"),
        (vec!["bullet one", "bullet two"], "bullet", "•", "1"),
        (vec!["zero", "one"], "decimal", "%1.", "0"),
        (vec!["restarted", "next"], "decimal", "%1.", "1"),
        (vec!["other bullet"], "bullet", "•", "1"),
    ] {
        let first = resolved_list_numbering(document, numbering, texts[0]);
        assert!(
            ids.insert(first.0.clone()),
            "source lists must have distinct numbering instances"
        );
        assert_eq!((&*first.1, &*first.2, &*first.3), (format, marker, start));
        for text in texts {
            assert_eq!(resolved_list_numbering(document, numbering, text), first);
        }
    }
    assert_eq!(xml_elements(numbering, "w:num").len(), ids.len());
    let separator = xml_elements(document, "w:p")
        .into_iter()
        .find(|paragraph| xml_text(paragraph) == "Separator.")
        .unwrap();
    assert!(xml_elements(separator, "w:numPr").is_empty());
}

#[test]
fn assembler_preserves_multiple_paragraphs_in_each_list_item() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    fs::write(&original, FIXTURE).unwrap();
    for properties in [
        "",
        "<w:pPr/>",
        "<w:pPr><w:pStyle w:val=\"Normal\"/><w:numPr><w:numId w:val=\"99\"/></w:numPr><w:spacing w:after=\"120\"/><w:ind w:left=\"100\" w:hanging=\"50\"/></w:pPr>",
    ] {
        let template = temp.path().join("template.docx");
        rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
            xml.replace("<w:p><w:r><w:t>{PARAGRAPH}", &format!("<w:p>{properties}<w:r><w:t>{{PARAGRAPH}}"))
        });
        for (marker, next_marker, format, start) in [
            ("-", "-", "bullet", "1"),
            ("0.", "1.", "decimal", "0"),
            ("3.", "4.", "decimal", "3"),
        ] {
            let indent = " ".repeat(marker.len() + 1);
            let markdown = format!("---\nversion: 1.0\nconfidentiality: Internal\n---\n{marker} First **paragraph**.\n\n{indent}Second *paragraph* with `code`.\n\n{indent}Third paragraph.\n\n{next_marker} Next item.\n\n{indent}Next continuation.\n\nOutside list.\n");
            let output = temp.path().join("multiple.docx");
            let repeat = temp.path().join("repeat.docx");
            assemble_markdown_docx(&template, &markdown, &output, None).unwrap();
            assemble_markdown_docx(&template, &markdown, &repeat, None).unwrap();
            let parts = zip_entries(&output);
            assert_eq!(parts, zip_entries(&repeat));
            let document = std::str::from_utf8(&parts["word/document.xml"]).unwrap();
            let numbering = std::str::from_utf8(&parts["word/numbering.xml"]).unwrap();
            let paragraphs = xml_elements(document, "w:p");
            let expected = ["First paragraph.", "Second paragraph with code.", "Third paragraph.",
                            "Next item.", "Next continuation.", "Outside list."];
            let source: Vec<_> = paragraphs.into_iter()
                .filter(|p| expected.iter().any(|text| xml_text(p).contains(text)))
                .collect();
            assert_eq!(source.iter().map(|p| xml_text(p)).collect::<Vec<_>>(), expected);
            let first = resolved_list_numbering(document, numbering, expected[0]);
            assert_eq!((&*first.1, &*first.3), (format, start));
            assert_eq!(resolved_list_numbering(document, numbering, expected[3]), first);
            assert_eq!(xml_elements(numbering, "w:num").len(), 1);
            for index in [1, 2, 4] {
                let paragraph = source[index];
                // Explicitly cancel any inherited marker, retaining the list body indent.
                assert_eq!(xml_attribute(xml_elements(paragraph, "w:numId")[0], "w:val"), "0");
                let indents = xml_elements(paragraph, "w:ind");
                assert_eq!(indents.len(), 1);
                assert_eq!(xml_attribute(indents[0], "w:left"), "720");
                assert_eq!(xml_attribute(indents[0], "w:firstLine"), "0");
                assert!(!indents[0].contains("w:hanging"));
                assert!(!paragraph.contains("<w:br"));
                if properties.contains("w:spacing") {
                    assert!(paragraph.contains("<w:spacing w:after=\"120\"/>"));
                    assert!(paragraph.contains("<w:pStyle w:val=\"Normal\"/>"));
                }
            }
            assert!(source[0].contains("<w:b/>"));
            assert!(source[1].contains("<w:i/>"));
            assert!(source[1].contains("Courier New"));
            validate_markdown_template_output(&output);
        }
    }
    assert_eq!(fs::read(original).unwrap(), FIXTURE);
}

#[test]
fn assembler_keeps_list_continuation_after_a_nested_list() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    let output = temp.path().join("nested.docx");
    fs::write(&template, FIXTURE).unwrap();
    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n3. Parent.\n\n   - Child.\n\n     Child continuation.\n\n   Parent continuation.\n\n4. Next parent.\n";
    assemble_markdown_docx(&template, markdown, &output, None).unwrap();
    let parts = zip_entries(&output);
    let document = std::str::from_utf8(&parts["word/document.xml"]).unwrap();
    let numbering = std::str::from_utf8(&parts["word/numbering.xml"]).unwrap();
    let parent = resolved_list_numbering(document, numbering, "Parent.");
    assert_eq!(
        resolved_list_numbering(document, numbering, "Next parent."),
        parent
    );
    let child = resolved_list_numbering(document, numbering, "Child.");
    assert_ne!(parent.0, child.0);
    assert_eq!(
        (&*parent.1, &*parent.3, &*child.1),
        ("decimal", "3", "bullet")
    );
    let paragraphs = xml_elements(document, "w:p");
    for text in ["Child continuation.", "Parent continuation."] {
        let paragraph = paragraphs.iter().find(|p| xml_text(p) == text).unwrap();
        assert_eq!(
            xml_attribute(xml_elements(paragraph, "w:numId")[0], "w:val"),
            "0"
        );
        assert_eq!(
            xml_attribute(xml_elements(paragraph, "w:ind")[0], "w:left"),
            "720"
        );
    }
}

fn resolved_list_numbering(
    document: &str,
    numbering: &str,
    text: &str,
) -> (String, String, String, String) {
    let paragraph = xml_elements(document, "w:p")
        .into_iter()
        .find(|paragraph| xml_text(paragraph) == text)
        .unwrap();
    let properties = xml_elements(paragraph, "w:numPr");
    assert_eq!(
        properties.len(),
        1,
        "{text} must have one native numbering reference"
    );
    assert_eq!(
        xml_attribute(xml_elements(properties[0], "w:ilvl")[0], "w:val"),
        "0"
    );
    let id = xml_attribute(xml_elements(properties[0], "w:numId")[0], "w:val");
    let instance = xml_elements(numbering, "w:num")
        .into_iter()
        .find(|instance| xml_attribute(instance, "w:numId") == id)
        .unwrap();
    let abstract_id = xml_attribute(xml_elements(instance, "w:abstractNumId")[0], "w:val");
    let definition = xml_elements(numbering, "w:abstractNum")
        .into_iter()
        .find(|definition| xml_attribute(definition, "w:abstractNumId") == abstract_id)
        .unwrap();
    (
        id,
        xml_attribute(xml_elements(definition, "w:numFmt")[0], "w:val"),
        xml_attribute(xml_elements(definition, "w:lvlText")[0], "w:val"),
        xml_attribute(xml_elements(instance, "w:startOverride")[0], "w:val"),
    )
}

#[test]
fn assembler_renders_authentication_delivery_steps_as_one_native_decimal_list() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    let output = temp.path().join("authentication-delivery.docx");
    fs::write(&template, FIXTURE).unwrap();
    let markdown = r#"---
title: Verwaltung von Authentisierungsinformationen
version: 3.0
confidentiality: internal
---

1. Initiale, zurückgesetzte oder temporäre Geheimnisse werden über einen geschützten und für die empfangende Person authentisierten Kanal übermittelt. Unverschlüsselte Klartext-E-Mail ist untersagt.
2. Als Übermittlungsarten kommen, soweit sie für den jeweiligen Fall freigegeben und geeignet sind, persönliche Übergabe, ein verschlüsselter Einmal-Link, ein Out-of-Band-Kanal oder eine telefonische Mitteilung nach Identitätsprüfung in Betracht.
3. Der Empfang ist zu bestätigen, etwa durch eine Rückmeldung im Vorgang oder einen erfolgreichen ersten Anmeldevorgang.
4. Temporäre Kennwörter sind bei der ersten Anmeldung zu ändern. Das eingesetzte System soll dies technisch erzwingen, soweit es dies unterstützt.
5. TODO: Freigegebene Übermittlungskanäle, die Form der Empfangsbestätigung und das Vorgehen bei nicht erzwingbarer Erstanmeldungsänderung sind festzulegen.
"#;

    assemble_markdown_docx(&template, markdown, &output, None).unwrap();

    let entries = zip_entries(&output);
    let document = std::str::from_utf8(&entries["word/document.xml"]).unwrap();
    for phrase in [
        "Initiale, zurückgesetzte oder temporäre Geheimnisse",
        "Als Übermittlungsarten kommen",
        "Der Empfang ist zu bestätigen",
        "Temporäre Kennwörter sind bei der ersten Anmeldung zu ändern",
        "TODO: Freigegebene Übermittlungskanäle",
    ] {
        let start = document[..document.find(phrase).unwrap()]
            .rfind("<w:p>")
            .unwrap();
        let end = start + document[start..].find("</w:p>").unwrap() + "</w:p>".len();
        assert!(document[start..end].contains("<w:numPr>"), "{phrase}");
    }
    let numbering = std::str::from_utf8(&entries["word/numbering.xml"]).unwrap();
    assert_eq!(numbering.matches("<w:num ").count(), 1);
    assert!(numbering.contains("<w:startOverride w:val=\"1\"/>"));
}

#[test]
fn assembler_fills_frontmatter_template_variables_and_leaves_controlled_tokens() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    let output = temp.path().join("filled.docx");
    write_template_with_header_token(&template, "{AUTHOR} · {DEPARTMENT} · {VERSION}");
    let markdown = r#"---
title: Policy
document_number: P-9
version: 1.0
confidentiality: Internal
author: Ada & Co
department: Quality
---
# Body
"#;

    assemble_markdown_docx(&template, markdown, &output, None).unwrap();
    let parts = zip_entries(&output);
    let header = String::from_utf8(parts["word/header1.xml"].clone()).unwrap();
    assert!(header.contains("Ada &amp; Co"));
    assert!(header.contains("Quality"));
    assert!(header.contains("{VERSION}"));
    assert!(!header.contains("{AUTHOR}"));
    assert!(!header.contains("{DEPARTMENT}"));
    let custom = String::from_utf8(parts["docProps/custom.xml"].clone()).unwrap();
    assert!(custom.contains("{TITLE}"));
    assert!(custom.contains("{DOCUMENT_NUMBER}"));
    assert!(custom.contains("{VERSION}"));
    assert!(custom.contains("{CONFIDENTIALITY}"));
    let document = String::from_utf8(parts["word/document.xml"].clone()).unwrap();
    assert!(document.contains("Body"));
    assert!(!document.contains("Ada"));
    assert!(!document.contains("---"));
}

fn write_template_with_header_token(path: &Path, header_text: &str) {
    let source = zip_entries_from_bytes(FIXTURE);
    let file = fs::File::create(path).unwrap();
    let mut archive = ZipWriter::new(file);
    for (name, bytes) in source {
        archive
            .start_file(&name, SimpleFileOptions::default())
            .unwrap();
        if name == "word/header1.xml" {
            let xml = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:p><w:r><w:t>{header_text}</w:t></w:r></w:p></w:hdr>"
            );
            archive.write_all(xml.as_bytes()).unwrap();
        } else {
            archive.write_all(&bytes).unwrap();
        }
    }
    archive.finish().unwrap();
}

fn zip_entries_from_bytes(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut entries = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.is_dir() {
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        entries.insert(entry.name().to_owned(), data);
    }
    entries
}

#[test]
fn assembler_rejects_missing_frontmatter() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    assert!(matches!(
        assemble_markdown_docx(
            &template,
            "# No frontmatter\n",
            &temp.path().join("missing.docx"),
            None
        ),
        Err(DmsError::InvalidMarkdownFrontmatter(_))
    ));
}

#[test]
fn assembler_preserves_two_three_four_and_five_column_table_fidelity() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    let template = temp.path().join("styled.docx");
    fs::write(&original, FIXTURE).unwrap();
    rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
        xml.replace("<w:tblPr>", "<w:tblPr><w:tblStyle w:val=\"TableGrid\"/>")
            .replace("<w:tr>", "<w:tblGrid><w:gridCol w:w=\"4500\"/><w:gridCol w:w=\"4500\"/></w:tblGrid><w:tr><w:trPr><w:cantSplit/></w:trPr>")
            .replace("</w:tcPr>", "<w:shd w:fill=\"EEEEEE\"/><w:vAlign w:val=\"center\"/></w:tcPr>")
            .replace("<w:p><w:r><w:t>{TABLE COLUMN 1}", "<w:p><w:pPr><w:pStyle w:val=\"FirstCell\"/></w:pPr><w:r><w:rPr><w:color w:val=\"123456\"/></w:rPr><w:t>{TABLE COLUMN 1}")
            .replace("<w:p><w:r><w:t>{TABLE COLUMN 2}", "<w:p><w:pPr><w:pStyle w:val=\"OtherCell\"/></w:pPr><w:r><w:rPr><w:color w:val=\"654321\"/></w:rPr><w:t>{TABLE COLUMN 2}")
            // Exercise non-divisible widths and a binding gutter.
            .replace("w:w=\"11906\"", "w:w=\"11384\"")
            .replace("w:left=\"1440\"", "w:left=\"1440\" w:gutter = \"501\"")
            .replace("</w:tc><w:tc>", "</w:tc><!-- retained row annotation --><w:tc>")
            .replace("<w:body>", "<w:body><w:p><w:pPr><w:sectPr><w:pgSz w:w=\"8000\" w:h=\"16000\"/><w:pgMar w:left=\"1000\" w:right=\"1000\"/></w:sectPr></w:pPr><w:r><w:t>Cover</w:t></w:r></w:p><w:p><w:fldSimple w:instr=\"TOC\"><w:r><w:t>Contents</w:t></w:r></w:fldSimple></w:p>")
    });
    let template_before = fs::read(&template).unwrap();
    let headers = ["Version", "Datum", "Änderung", "Verfasser", "Freigabe"];
    let long_text =
        "Eine ausführliche Beschreibung mit Umlauten äöü, 漢字 und unverändertem Inhalt. "
            .repeat(12);
    let raw = [
        "**1.0**",
        "*13.10.2026*",
        "A \\| B & \\*Stern\\*",
        "`Müller`",
        long_text.as_str(),
    ];
    let expected = [
        "1.0",
        "13.10.2026",
        "A | B & *Stern*",
        "Müller",
        long_text.trim(),
    ];
    let mut markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n".to_owned();
    for columns in 2..=5 {
        markdown.push_str(&format!(
            "\n## Versionsverlauf {columns}\n\n| {} |\n| {} |\n| {} |\n| {} |\n",
            headers[..columns].join(" | "),
            vec!["---"; columns].join(" | "),
            raw[..columns].join(" | "),
            vec![""; columns].join(" | ")
        ));
    }
    let source = temp.path().join("policy.md");
    fs::write(&source, &markdown).unwrap();
    let output = temp.path().join("tables.docx");
    assemble_markdown_docx(
        &template,
        &fs::read_to_string(&source).unwrap(),
        &output,
        None,
    )
    .unwrap();
    let second = temp.path().join("repeat.docx");
    assemble_markdown_docx(&template, &markdown, &second, None).unwrap();
    assert_eq!(fs::read(&output).unwrap(), fs::read(&second).unwrap());
    assert_eq!(fs::read(&source).unwrap(), markdown.as_bytes());
    assert_eq!(fs::read(&template).unwrap(), template_before);

    let before = zip_entries(&template);
    let after = zip_entries(&output);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    for (name, bytes) in &after {
        if name.ends_with(".xml") || name.ends_with(".rels") {
            // Parsing every part also checks matching element names and attributes.
            xml_elements(std::str::from_utf8(bytes).unwrap(), "unused");
        }
        if name != "word/document.xml" {
            assert_eq!(bytes, &before[name], "changed package part {name}");
        }
    }
    let document = std::str::from_utf8(&after["word/document.xml"]).unwrap();
    let original_document = std::str::from_utf8(&before["word/document.xml"]).unwrap();
    assert_eq!(
        xml_elements(document, "w:sectPr"),
        xml_elements(original_document, "w:sectPr")
    );
    let tables = xml_elements(document, "w:tbl");
    assert_eq!(
        &xml_elements(document, "w:p")[..2],
        &xml_elements(original_document, "w:p")[..2]
    );
    assert_eq!(tables.len(), 4);
    for (table, columns) in tables.iter().zip(2..=5) {
        assert!(table.contains("<w:tblStyle w:val=\"TableGrid\"/>"));
        let grid = xml_elements(table, "w:gridCol");
        assert_eq!(grid.len(), columns);
        let widths: Vec<u32> = grid
            .iter()
            .map(|col| xml_attribute(col, "w:w").parse().unwrap())
            .collect();
        // Every table, including two columns, now uses the fixed printable width.
        assert_eq!(widths.iter().sum::<u32>(), 11384 - 1440 - 1440 - 501);
        assert!(widths.iter().all(|width| *width > 0));
        assert!(widths.iter().max().unwrap() - widths.iter().min().unwrap() <= 1);
        assert_eq!(
            xml_attribute(xml_elements(table, "w:tblLayout")[0], "w:type"),
            "fixed"
        );
        assert_eq!(
            xml_attribute(xml_elements(table, "w:tblW")[0], "w:w"),
            widths.iter().sum::<u32>().to_string()
        );
        let rows = xml_elements(table, "w:tr");
        assert_eq!(rows.len(), 3);
        for (index, row) in rows.iter().enumerate() {
            assert!(row.contains("<w:cantSplit/>"));
            assert_eq!(row.matches("<!-- retained row annotation -->").count(), 1);
            let cells = xml_elements(row, "w:tc");
            assert_eq!(cells.len(), columns);
            for (column, cell) in cells.iter().enumerate() {
                let text = xml_text(cell);
                assert_eq!(
                    text,
                    match index {
                        0 => headers[column],
                        1 => expected[column],
                        _ => "",
                    }
                );
                assert_eq!(
                    xml_attribute(xml_elements(cell, "w:tcW")[0], "w:w"),
                    widths[column].to_string()
                );
                assert!(cell.contains("<w:shd w:fill=\"EEEEEE\"/>"));
                assert!(cell.contains("<w:vAlign w:val=\"center\"/>"));
                assert_eq!(
                    xml_attribute(xml_elements(cell, "w:pStyle")[0], "w:val"),
                    if column == 0 {
                        "FirstCell"
                    } else {
                        "OtherCell"
                    }
                );
                if index == 1 {
                    assert!(cell.contains(if column == 0 { "123456" } else { "654321" }));
                    match column {
                        0 => assert!(cell.contains("<w:b/>")),
                        1 => assert!(cell.contains("<w:i/>")),
                        3 => assert!(cell.contains("Courier New")),
                        _ => {}
                    }
                }
            }
        }
    }
}

#[test]
fn assembler_separates_consecutive_tables_for_word_save() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    let source = "---\ntitle: Tables\nversion: 1.0\nconfidentiality: internal\n---\n\n| A | B | C |\n| --- | --- | --- |\n| one | two | three |\n\n| D | E | F |\n| --- | --- | --- |\n| four | five | six |\n";
    let output = temp.path().join("tables.docx");

    assemble_markdown_docx(&template, source, &output, None).unwrap();

    let entries = zip_entries(&output);
    let document = std::str::from_utf8(&entries["word/document.xml"]).unwrap();
    let first_end = document.find("</w:tbl>").unwrap() + "</w:tbl>".len();
    let second_start = first_end + document[first_end..].find("<w:tbl").unwrap();
    assert_eq!(xml_elements(document, "w:tbl").len(), 2);
    assert!(
        document[first_end..second_start].contains("<w:p"),
        "Word coalesces adjacent tables without a paragraph separator"
    );
}

#[test]
fn assembler_sizes_wide_tables_without_grid_or_cell_widths() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    let template = temp.path().join("no-widths.docx");
    fs::write(&original, FIXTURE).unwrap();
    rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
        xml.replace(
            "<w:tcPr><w:tcW w:w=\"4500\" w:type=\"dxa\"/></w:tcPr>",
            "<w:tcPr/>",
        )
        .replace(
            "<w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>",
            "<w:tblPr/>",
        )
    });
    let output = temp.path().join("wide.docx");
    assemble_markdown_docx(
        &template,
        "---\nversion: 1.0\nconfidentiality: Internal\n---\n| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n",
        &output,
        None,
    )
    .unwrap();
    let parts = zip_entries(&output);
    let document = std::str::from_utf8(&parts["word/document.xml"]).unwrap();
    let widths: Vec<u32> = xml_elements(document, "w:gridCol")
        .iter()
        .map(|col| xml_attribute(col, "w:w").parse().unwrap())
        .collect();
    assert_eq!(widths.len(), 3);
    assert_eq!(widths.iter().sum::<u32>(), 11906 - 2880);
    assert_eq!(xml_elements(document, "w:tcW").len(), 6);
}

#[test]
fn assembler_preserves_mixed_inline_styles_and_surrounding_cell_content() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    let template = temp.path().join("mixed.docx");
    fs::write(&original, FIXTURE).unwrap();
    rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
        xml.replace("<w:r><w:t>{TABLE COLUMN 1}</w:t></w:r>", "<w:r><w:rPr><w:color w:val=\"123456\"/><w:b w:val=\"0\"/></w:rPr><w:t xml:space=\"preserve\">Before: {TABLE COLUMN 1} :after</w:t></w:r>")
    });
    let output = temp.path().join("mixed-output.docx");
    assemble_markdown_docx(
        &template,
        "---\nversion: 1.0\nconfidentiality: Internal\n---\n| A | B |\n|---|---|\n| plain **bold** then *italic* and ***both*** <https://example.test> | {TABLE COLUMN 1} |\n",
        &output,
        None,
    )
    .unwrap();
    let parts = zip_entries(&output);
    let document = std::str::from_utf8(&parts["word/document.xml"]).unwrap();
    let rows = xml_elements(document, "w:tr");
    let cells = xml_elements(rows[1], "w:tc");
    assert_eq!(xml_text(cells[0]), "Before: plain bold then italic and both https://example.test (https://example.test) :after");
    assert_eq!(xml_text(cells[1]), "{TABLE COLUMN 1}");
    let runs = xml_elements(cells[0], "w:r");
    for (text, bold, italic) in [
        ("plain ", false, false),
        ("bold", true, false),
        ("italic", false, true),
        ("both", true, true),
    ] {
        let run = runs.iter().find(|run| xml_text(run) == text).unwrap();
        assert_eq!(run.contains("<w:b/>"), bold);
        assert_eq!(run.contains("<w:i/>"), italic);
        assert!(run.contains("123456"));
        assert_eq!(xml_elements(run, "w:b").len(), 1);
    }
}

#[test]
fn assembler_rejects_unusable_page_dimensions_before_creating_wide_table_output() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    fs::write(&original, FIXTURE).unwrap();
    for bad_width in ["0", "100", "not-a-width"] {
        let template = temp.path().join(format!("{bad_width}.docx"));
        rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
            xml.replace("w:w=\"11906\"", &format!("w:w=\"{bad_width}\""))
        });
        let output = temp.path().join("must-not-exist.docx");
        assert!(matches!(
            assemble_markdown_docx(
                &template,
                "---\nversion: 1.0\nconfidentiality: Internal\n---\n| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n",
                &output,
                None
            ),
            Err(DmsError::InvalidMarkdownTemplate(_))
        ));
        assert!(!output.exists());
    }
}

fn xml_elements<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
    use quick_xml::{events::Event, Reader};
    let mut reader = Reader::from_str(xml);
    let mut stack = Vec::new();
    let mut result = Vec::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader.read_event().unwrap() {
            Event::Start(element) => {
                for attribute in element.attributes() {
                    attribute.unwrap();
                }
                stack.push((element.name().as_ref().to_vec(), start));
            }
            Event::Empty(element) => {
                for attribute in element.attributes() {
                    attribute.unwrap();
                }
                if element.name().as_ref() == name.as_bytes() {
                    result.push(&xml[start..reader.buffer_position() as usize]);
                }
            }
            Event::End(element) => {
                let (opening, start) = stack.pop().unwrap();
                assert_eq!(opening, element.name().as_ref());
                if opening == name.as_bytes() {
                    result.push(&xml[start..reader.buffer_position() as usize]);
                }
            }
            Event::Eof => {
                assert!(stack.is_empty());
                break;
            }
            _ => {}
        }
    }
    result
}

fn xml_attribute(xml: &str, name: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(xml);
    match reader.read_event().unwrap() {
        quick_xml::events::Event::Start(element) | quick_xml::events::Event::Empty(element) => {
            element
                .attributes()
                .map(Result::unwrap)
                .find(|attribute| attribute.key.as_ref() == name.as_bytes())
                .unwrap()
                .unescape_value()
                .unwrap()
                .into_owned()
        }
        _ => panic!("expected XML element"),
    }
}

fn xml_text(xml: &str) -> String {
    xml_elements(xml, "w:t")
        .iter()
        .map(|text| {
            let mut reader = quick_xml::Reader::from_str(text);
            let mut output = String::new();
            loop {
                match reader.read_event().unwrap() {
                    quick_xml::events::Event::Text(text) => {
                        output.push_str(&text.unescape().unwrap())
                    }
                    quick_xml::events::Event::Eof => break,
                    _ => {}
                }
            }
            output
        })
        .collect()
}

fn validate_markdown_template_output(path: &Path) {
    let parts = zip_entries(path);
    for required in [
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/styles.xml",
        "word/_rels/document.xml.rels",
        "docProps/custom.xml",
        "word/header1.xml",
        "word/footer1.xml",
        "word/media/fixture.png",
    ] {
        assert!(
            parts.contains_key(required),
            "missing package part {required}"
        );
    }
}

fn zip_entries(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let file = fs::File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut entries = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.is_dir() {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        entries.insert(entry.name().to_owned(), bytes);
    }
    entries
}

fn rewrite_xml_part(
    source: &Path,
    destination: &Path,
    part_name: &str,
    rewrite: impl FnOnce(String) -> String,
) {
    let file = fs::File::open(source).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        entries.push((
            entry.name().to_owned(),
            entry.is_dir(),
            entry.compression(),
            bytes,
        ));
    }
    let output = fs::File::create(destination).unwrap();
    let mut writer = ZipWriter::new(output);
    let mut rewrite = Some(rewrite);
    for (name, is_dir, compression, mut bytes) in entries {
        let options = SimpleFileOptions::default().compression_method(compression);
        if is_dir {
            writer.add_directory(name, options).unwrap();
            continue;
        }
        if name == part_name {
            let xml = String::from_utf8(bytes).unwrap();
            bytes = rewrite.take().unwrap()(xml).into_bytes();
        }
        writer.start_file(name, options).unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();
}

fn fixture_png() -> Vec<u8> {
    zip_entries_from_bytes(FIXTURE)["word/media/fixture.png"].clone()
}

fn png_pixel_size(bytes: &[u8]) -> (u32, u32) {
    (
        u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
        u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
    )
}

/// Structural PNG fixture with a correct signature, IHDR and IEND chunk.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut header = Vec::from(*b"IHDR");
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&crc32(&header).to_be_bytes());
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(b"IEND");
    bytes.extend_from_slice(&crc32(b"IEND").to_be_bytes());
    bytes
}

/// Structural JPEG fixture: SOI, JFIF APP0, SOF0 frame header, SOS and EOI.
fn jpeg_stub(width: u16, height: u16) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xD8];
    let mut app0 = Vec::from(*b"JFIF\0");
    app0.extend_from_slice(&[0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00]);
    bytes.extend_from_slice(&[0xFF, 0xE0]);
    bytes.extend_from_slice(&((app0.len() + 2) as u16).to_be_bytes());
    bytes.extend_from_slice(&app0);
    let mut frame = vec![8];
    frame.extend_from_slice(&height.to_be_bytes());
    frame.extend_from_slice(&width.to_be_bytes());
    frame.push(3);
    frame.extend_from_slice(&[1, 0x22, 0x00, 2, 0x11, 0x01, 3, 0x11, 0x01]);
    bytes.extend_from_slice(&[0xFF, 0xC0]);
    bytes.extend_from_slice(&((frame.len() + 2) as u16).to_be_bytes());
    bytes.extend_from_slice(&frame);
    let scan = [3u8, 1, 0x00, 2, 0x11, 3, 0x11, 0, 63, 0];
    bytes.extend_from_slice(&[0xFF, 0xDA]);
    bytes.extend_from_slice(&((scan.len() + 2) as u16).to_be_bytes());
    bytes.extend_from_slice(&scan);
    bytes.extend_from_slice(&[0xFF, 0xD9]);
    bytes
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[test]
fn assembler_embeds_local_source_images_as_drawings_and_package_media() {
    let temp = tempfile::tempdir().unwrap();
    let input_root = temp.path().join("input");
    fs::create_dir_all(input_root.join("assets")).unwrap();
    fs::create_dir_all(input_root.join("bilder")).unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();

    let logo = fixture_png();
    let (logo_width, logo_height) = png_pixel_size(&logo);
    let wide = WIDE_PNG.to_vec();
    let small = SMALL_PNG.to_vec();
    let photo = PHOTO_JPEG.to_vec();
    fs::write(input_root.join("assets/shot.png"), &logo).unwrap();
    fs::write(input_root.join("assets/shot copy.png"), &small).unwrap();
    fs::write(input_root.join("bilder/diagramm mit Ümlaut.png"), &wide).unwrap();
    fs::write(input_root.join("assets/photo.jpeg"), &photo).unwrap();

    let source = input_root.join("policy.md");
    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n# Policy\n\n\
![Screenshot](assets/shot.png)\n\n\
Inline ![inline shot](assets/shot.png) reference.\n\n\
![](<bilder/diagramm mit Ümlaut.png>)\n\n\
- ![List picture](assets/photo.jpeg)\n\n\
| A | B |\n| --- | --- |\n| ![Cell picture](assets/shot%20copy.png) | plain |\n";
    fs::write(&source, markdown).unwrap();
    let template_before = fs::read(&template).unwrap();
    let source_before = fs::read(&source).unwrap();

    let context = MarkdownAssetContext::new(&input_root, &source);
    let output = temp.path().join("policy.docx");
    assemble_markdown_docx(&template, markdown, &output, Some(&context)).unwrap();
    let repeated = temp.path().join("policy-repeat.docx");
    assemble_markdown_docx(&template, markdown, &repeated, Some(&context)).unwrap();
    assert_eq!(fs::read(&output).unwrap(), fs::read(&repeated).unwrap());
    assert_eq!(fs::read(&template).unwrap(), template_before);
    assert_eq!(fs::read(&source).unwrap(), source_before);
    for (path, bytes) in [
        ("assets/shot.png", &logo),
        ("assets/shot copy.png", &small),
        ("bilder/diagramm mit Ümlaut.png", &wide),
        ("assets/photo.jpeg", &photo),
    ] {
        assert_eq!(fs::read(input_root.join(path)).unwrap(), *bytes);
    }

    let before = zip_entries(&template);
    let after = zip_entries(&output);
    assert_eq!(after["word/media/fixture.png"], logo);
    for (name, bytes) in &before {
        if !matches!(
            name.as_str(),
            "word/document.xml" | "word/_rels/document.xml.rels" | "[Content_Types].xml"
        ) {
            assert_eq!(after.get(name), Some(bytes), "changed package part {name}");
        }
    }
    // One media part per distinct asset, in first-reference order.
    assert_eq!(after["word/media/dms-image-1.png"], logo);
    assert_eq!(after["word/media/dms-image-2.png"], wide);
    assert_eq!(after["word/media/dms-image-3.jpeg"], photo);
    assert_eq!(after["word/media/dms-image-4.png"], small);
    let mut expected_parts = before.keys().cloned().collect::<Vec<_>>();
    expected_parts.extend(
        [
            "word/media/dms-image-1.png",
            "word/media/dms-image-2.png",
            "word/media/dms-image-3.jpeg",
            "word/media/dms-image-4.png",
            "word/numbering.xml",
        ]
        .map(str::to_owned),
    );
    expected_parts.sort();
    assert_eq!(after.keys().cloned().collect::<Vec<_>>(), expected_parts);

    let relationships = String::from_utf8(after["word/_rels/document.xml.rels"].clone()).unwrap();
    for existing in ["rId1", "rId2", "rId3", "rId4"] {
        assert!(
            relationships.contains(&format!("Id=\"{existing}\"")),
            "lost template relationship {existing}"
        );
    }
    assert_eq!(relationships.matches("relationships/image").count(), 5);
    for (id, target) in [
        ("rId5", "media/dms-image-1.png"),
        ("rId6", "media/dms-image-2.png"),
        ("rId7", "media/dms-image-3.jpeg"),
        ("rId8", "media/dms-image-4.png"),
    ] {
        assert!(
            relationships.contains(&format!("Id=\"{id}\" ")) && relationships.contains(target),
            "missing media relationship {id} -> {target}"
        );
    }

    let content_types = String::from_utf8(after["[Content_Types].xml"].clone()).unwrap();
    assert_eq!(content_types.matches("Extension=\"png\"").count(), 1);
    assert_eq!(
        content_types.matches("Extension=\"jpeg\"").count(),
        1,
        "jpeg default must be added exactly once"
    );
    assert!(content_types.contains("<Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/>"));

    let document = String::from_utf8(after["word/document.xml"].clone()).unwrap();
    assert_eq!(xml_elements(&document, "w:drawing").len(), 5);
    assert_eq!(xml_elements(&document, "wp:extent").len(), 5);
    // A repeated source image reuses one media part and one relationship.
    assert_eq!(document.matches("r:embed=\"rId5\"").count(), 2);
    assert_eq!(document.matches("r:embed=\"rId6\"").count(), 1);
    for description in [
        "Screenshot",
        "inline shot",
        "bilder/diagramm mit Ümlaut.png",
        "List picture",
        "Cell picture",
    ] {
        assert!(
            document.contains(&format!("descr=\"{description}\"")),
            "missing alternative text {description:?}"
        );
    }
    // Printability limit: the fixture page is 11906 twips wide with 1440 twip
    // margins, so drawings may not exceed 9026 twips (5_731_510 EMU).
    let limit = (11906 - 1440 - 1440) * 635;
    let extents = xml_elements(&document, "wp:extent")
        .iter()
        .map(|extent| {
            (
                xml_attribute(extent, "cx").parse::<u64>().unwrap(),
                xml_attribute(extent, "cy").parse::<u64>().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        extents[0],
        (u64::from(logo_width) * 9525, u64::from(logo_height) * 9525),
        "a small image keeps its native 96 DPI size"
    );
    assert_eq!(extents[1], extents[0], "a repeated image keeps its size");
    assert_eq!(extents[2], (limit, limit / 2), "aspect ratio is preserved");
    assert_eq!(
        extents[3],
        (limit, limit * 360 / 640),
        "a wide JPEG is scaled to the printable width"
    );
    assert_eq!(extents[4], (40 * 9525, 20 * 9525));

    let image_only = xml_elements(&document, "w:p")
        .into_iter()
        .filter(|paragraph| paragraph.contains("<w:drawing>") && xml_text(paragraph).is_empty())
        .count();
    assert!(
        image_only >= 2,
        "image-only paragraphs must remain drawing paragraphs"
    );
}

#[test]
fn assembler_rejects_unusable_or_uncontained_source_images() {
    let temp = tempfile::tempdir().unwrap();
    let outside = temp.path().join("outside.png");
    let input_root = temp.path().join("input");
    fs::create_dir_all(input_root.join("assets")).unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    let logo = fixture_png();
    fs::write(input_root.join("assets/shot.png"), &logo).unwrap();
    fs::write(
        input_root.join("assets/vector.svg"),
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    )
    .unwrap();
    let mut broken = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    broken.extend_from_slice(b"truncated payload without a header chunk");
    fs::write(input_root.join("assets/broken.png"), &broken).unwrap();
    fs::write(&outside, &logo).unwrap();
    assert_eq!(
        fs::canonicalize(input_root.join("../outside.png")).unwrap(),
        fs::canonicalize(&outside).unwrap()
    );
    assert!(!fs::canonicalize(&outside)
        .unwrap()
        .starts_with(fs::canonicalize(&input_root).unwrap()));

    let source = input_root.join("policy.md");
    let context = MarkdownAssetContext::new(&input_root, &source);
    let absolute = if cfg!(windows) {
        ("C:\\outside\\shot.png", "remote or unsupported")
    } else {
        ("/outside/shot.png", "is an absolute path")
    };
    #[cfg(unix)]
    let symlink_escape = {
        use std::os::unix::fs::symlink;
        symlink(&outside, input_root.join("assets/link.png")).unwrap();
        Some(("assets/link.png", "resolves outside the input root"))
    };
    #[cfg(not(unix))]
    let symlink_escape: Option<(&str, &str)> = None;

    let cases = [
        (
            "assets/shot.png",
            false,
            "no source/asset resolution context",
        ),
        (
            "https://example.test/shot.png",
            true,
            "remote or unsupported",
        ),
        (absolute.0, true, absolute.1),
        ("../outside.png", true, "resolves outside the input root"),
        (
            "%2e%2e/outside.png",
            true,
            "resolves outside the input root",
        ),
        ("assets/missing.png", true, "does not exist"),
        ("assets/broken.png", true, "is corrupt"),
        ("assets/vector.svg", true, "unsupported image format"),
        ("assets/shot.png#fragment", true, "fragment or query"),
        ("assets/shot.png?query=1", true, "fragment or query"),
        ("file:assets/shot.png", true, "remote or unsupported"),
        ("data:image/png;base64,AA==", true, "remote or unsupported"),
        ("assets", true, "is not a regular file"),
        (
            "<img src=\"assets/shot.png\">",
            true,
            "raw HTML image markup",
        ),
    ];
    for (reference, use_context, expected) in cases {
        let markdown = format!(
            "---\nversion: 1.0\nconfidentiality: Internal\n---\n# Policy\n\n![Picture](<{reference}>)\n"
        );
        let output = temp.path().join("rejected.docx");
        let result = assemble_markdown_docx(
            &template,
            &markdown,
            &output,
            use_context.then_some(&context),
        );
        assert!(
            matches!(&result, Err(DmsError::InvalidMarkdownImage(message)) if message.contains(expected)),
            "reference {reference:?} was not rejected with {expected:?}: {result:?}"
        );
        assert!(!output.exists(), "rejected input created {reference:?}");
    }

    if let Some((reference, expected)) = symlink_escape {
        let markdown = format!(
            "---\nversion: 1.0\nconfidentiality: Internal\n---\n# Policy\n\n![Link]({reference})\n"
        );
        let output = temp.path().join("symlink.docx");
        let result = assemble_markdown_docx(&template, &markdown, &output, Some(&context));
        assert!(
            matches!(&result, Err(DmsError::InvalidMarkdownImage(message)) if message.contains(expected)),
            "symlink escape was not rejected: {result:?}"
        );
        assert!(!output.exists());
    }
}

#[test]
fn assembler_resolves_parent_relative_shared_images_inside_input_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("input");
    fs::create_dir_all(root.join("policies")).unwrap();
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::write(root.join("assets/shared.png"), SMALL_PNG).unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n![Shared](../assets/shared.png)\n\n![Again](%2e%2e/assets/shared.png)";
    for name in ["a", "b"] {
        let source = root.join(format!("policies/{name}.md"));
        fs::write(&source, markdown).unwrap();
        let output = temp.path().join(format!("{name}.docx"));
        let context = MarkdownAssetContext::new(&root, &source);
        assemble_markdown_docx(&template, markdown, &output, Some(&context)).unwrap();
        let parts = zip_entries(&output);
        assert_eq!(parts["word/media/dms-image-1.png"], SMALL_PNG);
        assert!(!parts.contains_key("word/media/dms-image-2.png"));
        let document = String::from_utf8(parts["word/document.xml"].clone()).unwrap();
        assert_eq!(document.matches("r:embed=\"rId5\"").count(), 2);
        assert_eq!(fs::read(source).unwrap(), markdown.as_bytes());
    }
    assert_eq!(fs::read(root.join("assets/shared.png")).unwrap(), SMALL_PNG);
    assert_eq!(fs::read(template).unwrap(), FIXTURE);
}

#[test]
fn assembler_rejects_header_only_and_structurally_corrupt_images() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    let context = MarkdownAssetContext::new(temp.path(), temp.path().join("policy.md"));
    let mut bad_crc = SMALL_PNG.to_vec();
    bad_crc[29] ^= 1;
    let mut bad_length = SMALL_PNG.to_vec();
    bad_length[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
    let mut no_scan = PHOTO_JPEG.to_vec();
    let scan = no_scan
        .windows(2)
        .position(|pair| pair == [0xff, 0xda])
        .unwrap();
    no_scan.truncate(scan);
    no_scan.extend_from_slice(&[0xff, 0xd9]);
    let mut empty_scan = PHOTO_JPEG.to_vec();
    let scan_length = u16::from_be_bytes([empty_scan[scan + 2], empty_scan[scan + 3]]);
    empty_scan.truncate(scan + 2 + usize::from(scan_length));
    empty_scan.extend_from_slice(&[0xff, 0xd9]);
    for (name, bytes) in [
        ("header.png", png_header(40, 20)),
        ("header.jpeg", jpeg_stub(640, 360)),
        ("crc.png", bad_crc),
        ("length.png", bad_length),
        ("truncated.png", SMALL_PNG[..SMALL_PNG.len() - 1].to_vec()),
        (
            "truncated.jpeg",
            PHOTO_JPEG[..PHOTO_JPEG.len() - 1].to_vec(),
        ),
        ("no-scan.jpeg", no_scan),
        ("empty-scan.jpeg", empty_scan),
    ] {
        fs::write(temp.path().join(name), bytes).unwrap();
        let markdown = format!("---\nversion: 1.0\nconfidentiality: Internal\n---\n![Bad]({name})");
        let output = temp.path().join("rejected.docx");
        let result = assemble_markdown_docx(&template, &markdown, &output, Some(&context));
        assert!(
            matches!(&result, Err(DmsError::InvalidMarkdownImage(message)) if message.contains("is corrupt")),
            "{name}: {result:?}"
        );
        assert!(!output.exists(), "{name} created output");
    }
}

#[test]
fn assembler_deduplicates_jpeg_content_types_and_rejects_template_conflicts() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    fs::write(&original, FIXTURE).unwrap();
    // Distinct bytes and canonical assets, not references to one cached image.
    assert_ne!(PHOTO_JPEG, SECOND_JPEG);
    fs::write(temp.path().join("one.jpeg"), PHOTO_JPEG).unwrap();
    fs::write(temp.path().join("two.jpeg"), SECOND_JPEG).unwrap();
    let context = MarkdownAssetContext::new(temp.path(), temp.path().join("policy.md"));
    let markdown =
        "---\nversion: 1.0\nconfidentiality: Internal\n---\n![One](one.jpeg)\n\n![Two](two.jpeg)";
    for declaration in [
        "",
        "<Default Extension=\"JpEg\" ContentType=\"image/jpeg\"/>",
    ] {
        let template = temp.path().join("template.docx");
        rewrite_xml_part(&original, &template, "[Content_Types].xml", |xml| {
            xml.replace("</Types>", &format!("{declaration}</Types>"))
        });
        let output = temp.path().join("ok.docx");
        assemble_markdown_docx(&template, markdown, &output, Some(&context)).unwrap();
        let parts = zip_entries(&output);
        assert_eq!(parts["word/media/dms-image-1.jpeg"], PHOTO_JPEG);
        assert_eq!(parts["word/media/dms-image-2.jpeg"], SECOND_JPEG);
        let types = String::from_utf8(parts["[Content_Types].xml"].clone()).unwrap();
        let defaults = xml_elements(&types, "Default");
        assert_eq!(
            defaults
                .iter()
                .filter(|entry| xml_attribute(entry, "Extension").eq_ignore_ascii_case("jpeg"))
                .count(),
            1
        );
        if !declaration.is_empty() {
            assert_eq!(
                parts["[Content_Types].xml"],
                zip_entries(&template)["[Content_Types].xml"]
            );
        }
    }
    for declaration in [
        "<Default Extension=\"jpeg\" ContentType=\"image/png\"/>",
        "<Default Extension=\"JPEG\" ContentType=\"image/png\"/>",
        "<Default Extension=\"JPEG\" ContentType=\"image/png\"/><Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/>",
        "<Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/><Default Extension=\"JPEG\" ContentType=\"image/png\"/>",
    ] {
        let template = temp.path().join("conflict.docx");
        rewrite_xml_part(&original, &template, "[Content_Types].xml", |xml| {
            xml.replace("</Types>", &format!("{declaration}</Types>"))
        });
        let before = fs::read(&template).unwrap();
        let output = temp.path().join("rejected.docx");
        let result = assemble_markdown_docx(&template, markdown, &output, Some(&context));
        assert!(matches!(&result, Err(DmsError::InvalidMarkdownTemplate(message)) if message.contains("[Content_Types].xml")), "{result:?}");
        assert!(!output.exists());
        assert_eq!(fs::read(&template).unwrap(), before);
    }
}

#[test]
fn assembler_avoids_template_media_relationship_and_drawing_id_collisions() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    let mut entries = zip_entries_from_bytes(FIXTURE);
    entries.insert("word/media/dms-image-1.png".to_owned(), SMALL_PNG.to_vec());
    let rels = String::from_utf8(entries["word/_rels/document.xml.rels"].clone()).unwrap();
    let occupied = "<Relationship Id=\"rId900\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/dms-image-1.png\"/>";
    entries.insert(
        "word/_rels/document.xml.rels".to_owned(),
        rels.replace("</Relationships>", &format!("{occupied}</Relationships>"))
            .into_bytes(),
    );
    let document = String::from_utf8(entries["word/document.xml"].clone()).unwrap();
    let drawing = concat!(
        "<w:p><w:r><w:drawing><wp:inline xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\">",
        "<wp:extent cx=\"381000\" cy=\"190500\"/><wp:docPr id=\"7000\" name=\"Existing\"/>",
        "<a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">",
        "<pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:nvPicPr><pic:cNvPr id=\"7000\" name=\"Existing\"/><pic:cNvPicPr/></pic:nvPicPr>",
        "<pic:blipFill><a:blip r:embed=\"rId900\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr/></pic:pic>",
        "</a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"
    );
    entries.insert(
        "word/document.xml".to_owned(),
        document
            .replace("<w:body>", &format!("<w:body>{drawing}"))
            .into_bytes(),
    );
    let mut writer = ZipWriter::new(fs::File::create(&template).unwrap());
    for (name, bytes) in &entries {
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap();
    let before = fs::read(&template).unwrap();
    fs::write(temp.path().join("new.png"), WIDE_PNG).unwrap();
    let context = MarkdownAssetContext::new(temp.path(), temp.path().join("policy.md"));
    let output = temp.path().join("out.docx");
    assemble_markdown_docx(
        &template,
        "---\nversion: 1.0\nconfidentiality: Internal\n---\n![New](new.png)\n\n![Repeat](new.png)",
        &output,
        Some(&context),
    )
    .unwrap();
    let parts = zip_entries(&output);
    assert_eq!(parts["word/media/dms-image-1.png"], SMALL_PNG);
    assert_eq!(parts["word/media/dms-image-2.png"], WIDE_PNG);
    let relationships = String::from_utf8(parts["word/_rels/document.xml.rels"].clone()).unwrap();
    assert!(relationships.contains(occupied));
    let new_relationship = xml_elements(&relationships, "Relationship")
        .into_iter()
        .find(|entry| xml_attribute(entry, "Id") == "rId901")
        .unwrap();
    assert_eq!(
        xml_attribute(new_relationship, "Target"),
        "media/dms-image-2.png"
    );
    let document = String::from_utf8(parts["word/document.xml"].clone()).unwrap();
    assert!(document.contains(drawing));
    let ids = xml_elements(&document, "wp:docPr")
        .iter()
        .map(|entry| xml_attribute(entry, "id"))
        .collect::<Vec<_>>();
    assert_eq!(ids, ["7000", "7001", "7002"]);
    assert_eq!(document.matches("r:embed=\"rId901\"").count(), 2);
    assert_eq!(fs::read(&template).unwrap(), before);
    for (name, bytes) in entries {
        if !matches!(
            name.as_str(),
            "word/document.xml" | "word/_rels/document.xml.rels"
        ) {
            assert_eq!(parts[&name], bytes, "changed template part {name}");
        }
    }
}

#[test]
fn assembler_avoids_drawing_ids_in_preserved_package_parts() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("new.png"), SMALL_PNG).unwrap();
    let context = MarkdownAssetContext::new(temp.path(), temp.path().join("policy.md"));
    for (part, root, contents, relationship_type) in [
        ("word/header1.xml", "w:hdr", "", "header"),
        ("word/footer1.xml", "w:ftr", "", "footer"),
        (
            "word/footnotes.xml",
            "w:footnotes",
            "w:footnote",
            "footnotes",
        ),
        ("word/comments.xml", "w:comments", "w:comment", "comments"),
    ] {
        // Independently exercise both ID kinds as the package maximum.
        for (doc_id, pic_id) in [(1, 0), (9000, 8000), (8000, 9000)] {
            let drawing = format!(concat!(
                "<w:p><w:r><w:drawing><wp:inline>",
                "<wp:extent cx=\"9525\" cy=\"9525\"/><wp:docPr id=\"{}\" name=\"Preserved\"/>",
                "<a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">",
                "<pic:pic><pic:nvPicPr><pic:cNvPr id=\"{}\" name=\"Preserved\"/><pic:cNvPicPr/></pic:nvPicPr>",
                "<pic:blipFill><a:blip r:embed=\"rIdLogo\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr/></pic:pic>",
                "</a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"
            ), doc_id, pic_id);
            let drawing = if contents.is_empty() {
                drawing
            } else {
                format!("<{contents} w:id=\"1\">{drawing}</{contents}>")
            };
            let xml = format!(
                "<{root} xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">{drawing}</{root}>"
            );
            let xml = if part == "word/comments.xml" {
                xml.replace("wp:", "draw:")
                    .replace("xmlns:wp=", "xmlns:draw=")
                    .replace("pic:", "picture:")
                    .replace("xmlns:pic=", "xmlns:picture=")
                    .replace(&format!("id=\"{pic_id}\""), &format!("id = '{pic_id}'"))
            } else {
                xml
            };
            let mut entries = zip_entries_from_bytes(FIXTURE);
            entries.insert(part.to_owned(), xml.into_bytes());
            let filename = part.strip_prefix("word/").unwrap();
            entries.insert(
                format!("word/_rels/{filename}.rels"),
                b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdLogo\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/fixture.png\"/></Relationships>".to_vec(),
            );
            if !contents.is_empty() {
                let types = String::from_utf8(entries["[Content_Types].xml"].clone()).unwrap();
                entries.insert("[Content_Types].xml".to_owned(), types.replace(
                    "</Types>",
                    &format!("<Override PartName=\"/{part}\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.{relationship_type}+xml\"/></Types>"),
                ).into_bytes());
                let rels =
                    String::from_utf8(entries["word/_rels/document.xml.rels"].clone()).unwrap();
                entries.insert("word/_rels/document.xml.rels".to_owned(), rels.replace(
                    "</Relationships>",
                    &format!("<Relationship Id=\"rIdPreserved\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/{relationship_type}\" Target=\"{filename}\"/></Relationships>"),
                ).into_bytes());
            }
            let template = temp.path().join("template.docx");
            let mut writer = ZipWriter::new(fs::File::create(&template).unwrap());
            for (name, bytes) in &entries {
                writer
                    .start_file(name, SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(bytes).unwrap();
            }
            writer.finish().unwrap();
            let before = fs::read(&template).unwrap();
            let output = temp.path().join("out.docx");
            assemble_markdown_docx(
                &template,
                "---\nversion: 1.0\nconfidentiality: Internal\n---\n![New](new.png)\n\n![Repeat](new.png)",
                &output,
                Some(&context),
            ).unwrap();
            let after = zip_entries(&output);
            let document = std::str::from_utf8(&after["word/document.xml"]).unwrap();
            for name in ["wp:docPr", "pic:cNvPr"] {
                let ids = xml_elements(document, name)
                    .iter()
                    .map(|entry| xml_attribute(entry, "id").parse::<u32>().unwrap())
                    .collect::<Vec<_>>();
                let maximum = doc_id.max(pic_id);
                assert_eq!(
                    ids,
                    [maximum + 1, maximum + 2],
                    "{part}: {name}, existing {doc_id}/{pic_id}"
                );
                assert!(ids.iter().all(|id| *id > doc_id && *id > pic_id));
            }
            for (name, bytes) in &after {
                if name.ends_with(".xml") || name.ends_with(".rels") {
                    xml_elements(std::str::from_utf8(bytes).unwrap(), "unused");
                }
            }
            for (name, bytes) in &entries {
                if !matches!(
                    name.as_str(),
                    "word/document.xml" | "word/_rels/document.xml.rels"
                ) {
                    assert_eq!(after.get(name), Some(bytes), "changed package part {name}");
                }
            }
            assert_eq!(fs::read(&template).unwrap(), before);
            assert_eq!(fs::read(temp.path().join("new.png")).unwrap(), SMALL_PNG);
        }
    }
}

#[test]
fn assembler_keeps_image_alt_inline_events_out_of_body_runs() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    fs::write(temp.path().join("image.png"), SMALL_PNG).unwrap();
    let source = temp.path().join("policy.md");
    let context = MarkdownAssetContext::new(temp.path(), &source);
    for (alt, expected) in [
        ("line one\nline two", "line one line two"),
        ("line one  \nline two", "line one line two"),
        ("line one\\\nline two", "line one line two"),
        (
            "**bold** *italic* `code` ~~strike~~",
            "bold italic code strike",
        ),
        ("[portal](https://example.test)", "portal"),
        (
            "<span>markup</span> & \"quoted\"",
            "<span>markup</span> & “quoted”",
        ),
        ("equation $x^2$", "equation $x^2$"),
        ("note\\[^n\\]", "note[^n]"),
        ("outer ![inner](unused.png) end", "outer inner end"),
    ] {
        let markdown = format!(
            "---\nversion: 1.0\nconfidentiality: Internal\n---\n![{alt}](image.png)\n\n*before ![{alt}](image.png) after*\n"
        );
        fs::write(&source, &markdown).unwrap();
        let output = temp.path().join("alt.docx");
        assemble_markdown_docx(&template, &markdown, &output, Some(&context)).unwrap();
        let parts = zip_entries(&output);
        let document = std::str::from_utf8(&parts["word/document.xml"]).unwrap();
        let paragraphs = xml_elements(document, "w:p")
            .into_iter()
            .filter(|paragraph| paragraph.contains("<w:drawing>"))
            .collect::<Vec<_>>();
        assert_eq!(paragraphs.len(), 2, "{alt:?}");
        for paragraph in &paragraphs {
            for name in ["wp:docPr", "pic:cNvPr"] {
                let properties = xml_elements(paragraph, name);
                assert_eq!(properties.len(), 1, "{alt:?}");
                assert_eq!(xml_attribute(properties[0], "descr"), expected, "{alt:?}");
            }
        }
        assert_eq!(xml_elements(paragraphs[0], "w:r").len(), 1, "{alt:?}");
        assert!(xml_elements(paragraphs[0], "w:t").is_empty(), "{alt:?}");
        assert!(xml_elements(paragraphs[0], "w:br").is_empty(), "{alt:?}");
        assert_eq!(xml_text(paragraphs[1]), "before  after", "{alt:?}");
        let text_runs = xml_elements(paragraphs[1], "w:r")
            .into_iter()
            .filter(|run| !xml_elements(run, "w:t").is_empty())
            .collect::<Vec<_>>();
        assert_eq!(text_runs.len(), 2, "{alt:?}");
        assert!(
            text_runs.iter().all(|run| run.contains("<w:i/>")),
            "{alt:?}"
        );
        assert_eq!(fs::read(&source).unwrap(), markdown.as_bytes());
    }
    let output = temp.path().join("rejected.docx");
    assert!(matches!(assemble_markdown_docx(
        &template,
        "---\nversion: 1.0\nconfidentiality: Internal\n---\n![<img src=\"nested.png\">](image.png)",
        &output,
        Some(&context),
    ), Err(DmsError::InvalidMarkdownImage(message)) if message.contains("raw HTML image markup")));
    assert!(!output.exists());
    assert_eq!(fs::read(template).unwrap(), FIXTURE);
    assert_eq!(fs::read(temp.path().join("image.png")).unwrap(), SMALL_PNG);
}

#[test]
fn assembler_preserves_other_markdown_constructs_without_losing_content() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n# Constructs\n\n\
- [x] done task\n- [ ] open task\n\n\
Footnote reference [^note] here.\n\n[^note]: Footnote body text.\n\n\
Line with <br> inline HTML and <span>markup</span> here.\n\n\
Math $x^2$ stays.\n\n~~struck out~~ text.\n\n\
Term\n: Definition text\n\n\
> Quoted line.\n\n1. first\n   1. nested\n\n\
Autolink <https://example.test> and `code span`.\n\n---\n\nx: 1\n";
    let output = temp.path().join("constructs.docx");
    assemble_markdown_docx(&template, markdown, &output, None).unwrap();

    let parts = zip_entries(&output);
    let document = String::from_utf8(parts["word/document.xml"].clone()).unwrap();
    let text = xml_text(&document);
    for visible in [
        "[x] done task",
        "[ ] open task",
        "[^note]",
        "Footnote body text.",
        "markup",
        "$x^2$",
        "struck out",
        "Term",
        "Definition text",
        "Quoted line.",
        "first",
        "nested",
        "https://example.test",
        "x: 1",
        "—",
    ] {
        assert!(
            text.contains(visible),
            "lost visible content {visible:?} in {text:?}"
        );
    }
    for markup in ["&lt;br&gt;", "&lt;span&gt;", "&lt;/span&gt;"] {
        assert!(
            document.contains(markup),
            "raw inline HTML was not preserved as text: {markup:?}"
        );
    }
}

// SHA-256 of `fixtures/markdown-wide.png`, independently computed on disk.
const WIDE_PNG_SHA256: &str = "94c9ba9d18743280297aa28d8727e9c259c330450f05ed5049bafc7e71bda064";

// A synthetic template Table of Contents structured document: the
// `Table of Contents` gallery, an English heading and a real TOC field.
const TEMPLATE_TOC_SDT: &str = concat!(
    "<w:sdt><w:sdtPr><w:docPartObj>",
    "<w:docPartGallery w:val=\"Table of Contents\"/><w:docPartUnique/>",
    "</w:docPartObj></w:sdtPr><w:sdtContent>",
    "<w:p><w:pPr><w:pStyle w:val=\"TOCHeading\"/></w:pPr>",
    "<w:r><w:t>Table of Contents</w:t></w:r></w:p>",
    "<w:p><w:fldSimple w:instr=\" TOC \\o &quot;1-3&quot; \\h \\z \\u \">",
    "<w:r><w:t>1 Scope</w:t></w:r></w:fldSimple></w:p>",
    "</w:sdtContent></w:sdt>"
);
// A second, unrelated structured document that must never be removed.
const TEMPLATE_OTHER_SDT: &str = concat!(
    "<w:sdt><w:sdtPr><w:docPartObj>",
    "<w:docPartGallery w:val=\"Custom Gallery\"/>",
    "</w:docPartObj></w:sdtPr><w:sdtContent>",
    "<w:p><w:r><w:t>Custom block</w:t></w:r></w:p>",
    "</w:sdtContent></w:sdt>"
);
const TEMPLATE_TOC_FIELD: &str = "TOC \\o &quot;1-3&quot; \\h \\z \\u";

fn document_xml_of(path: &Path) -> String {
    String::from_utf8(zip_entries(path)["word/document.xml"].clone()).unwrap()
}

fn gallery_count(xml: &str, gallery: &str) -> usize {
    xml.matches(&format!("<w:docPartGallery w:val=\"{gallery}\"/>"))
        .count()
}

#[test]
fn assembler_retains_template_toc_by_default_and_removes_only_matching_sdt() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    let template = temp.path().join("toc.docx");
    fs::write(&original, FIXTURE).unwrap();
    rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
        xml.replace(
            "<w:body>",
            &format!("<w:body>{TEMPLATE_TOC_SDT}{TEMPLATE_OTHER_SDT}"),
        )
    });
    let template_before = fs::read(&template).unwrap();
    let baseline = document_xml_of(&template);
    let baseline_toc = gallery_count(&baseline, "Table of Contents");
    assert!(baseline_toc >= 1, "fixture must carry the injected TOC sdt");
    assert_eq!(gallery_count(&baseline, "Custom Gallery"), 1);

    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n# Policy\n\n\
## Inhaltsverzeichnis\n\n- Scope\n- Versionsverlauf\n\n## Scope\n\nBody text.\n";

    let default_output = temp.path().join("default.docx");
    assemble_markdown_docx(&template, markdown, &default_output, None).unwrap();
    let retained = document_xml_of(&default_output);
    // The default assembly keeps the template TOC, its field and its heading.
    assert_eq!(gallery_count(&retained, "Table of Contents"), baseline_toc);
    assert_eq!(gallery_count(&retained, "Custom Gallery"), 1);
    assert!(retained.contains(TEMPLATE_TOC_FIELD));
    assert!(retained.contains(">Table of Contents<"));
    assert!(retained.contains("Custom block"));
    // A source-authored static TOC is body content and survives untouched.
    assert!(retained.contains("Inhaltsverzeichnis"));

    let without = temp.path().join("without.docx");
    assemble_markdown_docx_with_options(&template, markdown, &without, None, true).unwrap();
    let removed = document_xml_of(&without);
    // Only the Table-of-Contents structured document is gone.
    assert_eq!(gallery_count(&removed, "Table of Contents"), 0);
    assert_eq!(gallery_count(&removed, "Custom Gallery"), 1);
    assert!(!removed.contains(TEMPLATE_TOC_FIELD));
    assert!(removed.contains("Custom block"));
    assert!(removed.contains("Inhaltsverzeichnis"));
    assert!(removed.contains("Body text."));
    // TOC removal changes only the body; native list numbering is identical.
    assert_eq!(fs::read(&template).unwrap(), template_before);
    let before_parts = zip_entries(&default_output);
    let after_parts = zip_entries(&without);
    assert_eq!(
        before_parts.keys().collect::<Vec<_>>(),
        after_parts.keys().collect::<Vec<_>>()
    );
    for (name, bytes) in &before_parts {
        if name != "word/document.xml" {
            assert_eq!(
                after_parts.get(name),
                Some(bytes),
                "changed package part {name}"
            );
        }
    }
}

#[test]
fn assembler_reports_digests_of_the_asset_bytes_it_actually_embedded() {
    let temp = tempfile::tempdir().unwrap();
    let input_root = temp.path().join("input");
    fs::create_dir_all(input_root.join("assets")).unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    let shot = WIDE_PNG.to_vec();
    fs::write(input_root.join("assets/shot.png"), &shot).unwrap();

    let source = input_root.join("policy.md");
    // The same asset is referenced twice: one media part, one digest.
    let markdown = "---\nversion: 1.0\nconfidentiality: Internal\n---\n# Policy\n\n\
![one](assets/shot.png)\n\n![two](assets/shot.png)\n";
    fs::write(&source, markdown).unwrap();
    let context = MarkdownAssetContext::new(&input_root, &source);

    let output = temp.path().join("policy.docx");
    let digests: Vec<MarkdownAssetDigest> =
        assemble_markdown_docx_with_options(&template, markdown, &output, Some(&context), false)
            .unwrap();
    assert_eq!(digests.len(), 1);
    assert_eq!(digests[0].relative_path, Path::new("assets/shot.png"));
    // The digest equals the hash of the bytes written into the package, which
    // are the source bytes, exactly.
    assert_eq!(digests[0].sha256, WIDE_PNG_SHA256);
    let parts = zip_entries(&output);
    assert_eq!(parts["word/media/dms-image-1.png"], shot);
    assert_eq!(
        format!("{:x}", Sha256::digest(&parts["word/media/dms-image-1.png"])),
        digests[0].sha256
    );

    // An assembly without images reports no assets.
    let empty = assemble_markdown_docx_with_options(
        &template,
        "---\nversion: 1.0\nconfidentiality: Internal\n---\n# T\n\nText.\n",
        &temp.path().join("empty.docx"),
        None,
        false,
    )
    .unwrap();
    assert!(empty.is_empty());

    // The default wrapper keeps the same bytes and still returns unit.
    let plain = temp.path().join("plain.docx");
    assemble_markdown_docx(&template, markdown, &plain, Some(&context)).unwrap();
    assert_eq!(fs::read(&output).unwrap(), fs::read(&plain).unwrap());
}

#[test]
fn toc_removal_matches_word_namespaces_not_prefix_spelling_or_foreign_attributes() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("original.docx");
    let template = temp.path().join("template.docx");
    let output = temp.path().join("output.docx");
    fs::write(&original, FIXTURE).unwrap();
    let alternate = TEMPLATE_TOC_SDT.replace("w:", "q:").replacen(
        "<q:sdt>",
        "<q:sdt xmlns:q='http://schemas.openxmlformats.org/wordprocessingml/2006/main'>",
        1,
    );
    let foreign = TEMPLATE_OTHER_SDT.replace(
        "w:val=\"Custom Gallery\"",
        "x:val='Table of Contents' xmlns:x='urn:not-word'",
    );
    rewrite_xml_part(&original, &template, "word/document.xml", |xml| {
        xml.replace("<w:body>", &format!("<w:body>{alternate}{foreign}"))
    });
    assemble_markdown_docx_with_options(
        &template,
        "---\nversion: 1.0\nconfidentiality: internal\n---\nBody",
        &output,
        None,
        true,
    )
    .unwrap();
    let result = document_xml_of(&output);
    assert!(!result.contains("<q:sdt"));
    assert!(result.contains(&foreign));
}
