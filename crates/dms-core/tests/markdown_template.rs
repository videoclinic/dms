use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use dms_core::{
    assemble_markdown_docx, validate_markdown_template, DmsError, MarkdownTemplateValidationState,
    Workspace, MARKDOWN_TEMPLATE_CONTRACT_VERSION, SCHEMA_VERSION,
};
use tempfile::TempDir;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

const FIXTURE: &[u8] = include_bytes!("fixtures/markdown-template.docx");

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
        fixture
            .workspace
            .reassociate_document(other.id, &fixture.template_path),
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
---
# Handbook
## Scope
#### Details

Normal **bold** and *italic* text with [portal](https://example.test) and `inline code`.

- first bullet
- second bullet

1. first number
2. second number

```rust
let value = 1;
```

| Name | Value |
| --- | --- |
| Alpha | One |
| Beta | Two |
"#;

    assemble_markdown_docx(&template, markdown, &first).unwrap();
    assemble_markdown_docx(&template, markdown, &second).unwrap();
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    validate_markdown_template_output(&first);

    let template_parts = zip_entries(&template);
    let output_parts = zip_entries(&first);
    assert_eq!(
        template_parts.keys().collect::<Vec<_>>(),
        output_parts.keys().collect::<Vec<_>>()
    );
    for (name, bytes) in &template_parts {
        if name != "word/document.xml" {
            assert_eq!(
                output_parts.get(name),
                Some(bytes),
                "changed package part {name}"
            );
        }
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
    for text in [
        "Handbook",
        "Scope",
        "Details",
        "Normal ",
        "first bullet",
        "1. ",
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
}

#[test]
fn assembler_rejects_missing_frontmatter_and_tables_outside_the_two_column_contract() {
    let temp = tempfile::tempdir().unwrap();
    let template = temp.path().join("template.docx");
    fs::write(&template, FIXTURE).unwrap();
    assert!(matches!(
        assemble_markdown_docx(
            &template,
            "# No frontmatter\n",
            &temp.path().join("missing.docx")
        ),
        Err(DmsError::InvalidMarkdownFrontmatter(_))
    ));

    let three_columns = "---\nversion: 1.0\nconfidentiality: Internal\n---\n| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n";
    assert!(matches!(
        assemble_markdown_docx(&template, three_columns, &temp.path().join("table.docx")),
        Err(DmsError::InvalidMarkdownTemplate(message)) if message.contains("exactly two columns")
    ));
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
