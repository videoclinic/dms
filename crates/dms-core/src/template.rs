use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use quick_xml::{events::Event as XmlEvent, Reader};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use super::{DmsError, Result, Workspace};

pub const MARKDOWN_TEMPLATE_CONTRACT_VERSION: u32 = 1;

const REQUIRED_PARTS: &[&str] = &[
    "[Content_Types].xml",
    "_rels/.rels",
    "word/document.xml",
    "word/styles.xml",
    "word/_rels/document.xml.rels",
    "docProps/custom.xml",
];
const PARAGRAPH_PROTOTYPES: &[(&str, &str)] = &[
    ("heading1", "{Heading 1}"),
    ("heading2", "{Heading 2}"),
    ("heading3", "{Heading 3}"),
    ("heading4", "{Heading 4}"),
    ("paragraph", "{PARAGRAPH}"),
    ("list", "{BULLET LIST}"),
];
const TABLE_COLUMN_1: &str = "{TABLE COLUMN 1}";
const TABLE_COLUMN_2: &str = "{TABLE COLUMN 2}";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MarkdownTemplateAsset {
    pub id: Uuid,
    #[serde(with = "super::relative_path_serde")]
    pub relative_path: PathBuf,
    pub sha256: String,
    pub contract_version: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownTemplateValidationState {
    Valid,
    Changed,
    Missing,
    Invalid,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkdownTemplateValidation {
    pub state: MarkdownTemplateValidationState,
    pub stored_sha256: String,
    pub current_sha256: Option<String>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownTemplateContract {
    pub contract_version: u32,
    pub package_parts: Vec<String>,
}

#[derive(Clone, Debug)]
struct LoadedTemplate {
    entries: Vec<PackageEntry>,
    document_xml: String,
    prototype_start: usize,
    prototype_end: usize,
    paragraphs: BTreeMap<String, String>,
    table: String,
}

#[derive(Clone, Debug)]
struct PackageEntry {
    name: String,
    bytes: Vec<u8>,
    compression: zip::CompressionMethod,
    is_dir: bool,
}

impl Workspace {
    pub fn markdown_template(&self) -> Option<&MarkdownTemplateAsset> {
        self.markdown_template.as_ref()
    }

    pub fn import_markdown_template(
        &mut self,
        source_path: &Path,
    ) -> Result<MarkdownTemplateAsset> {
        let (absolute_path, relative_path) = self.resolve_template_source(source_path)?;
        if self
            .documents
            .values()
            .any(|document| document.relative_path == relative_path)
        {
            return Err(DmsError::TemplateIsControlledDocument(relative_path));
        }
        validate_markdown_template(&absolute_path)?;
        let sha256 = sha256_file(&absolute_path)?;
        let id = self
            .markdown_template
            .as_ref()
            .map(|template| template.id)
            .unwrap_or_else(Uuid::new_v4);
        let template = MarkdownTemplateAsset {
            id,
            relative_path,
            sha256,
            contract_version: MARKDOWN_TEMPLATE_CONTRACT_VERSION,
        };
        self.markdown_template = Some(template.clone());
        Ok(template)
    }

    pub fn remove_markdown_template(&mut self) -> Option<MarkdownTemplateAsset> {
        self.markdown_template.take()
    }

    pub fn markdown_template_validation(&self) -> Option<MarkdownTemplateValidation> {
        let template = self.markdown_template.as_ref()?;
        let path = self.edit_root.join(&template.relative_path);
        if !path.is_file() {
            return Some(MarkdownTemplateValidation {
                state: MarkdownTemplateValidationState::Missing,
                stored_sha256: template.sha256.clone(),
                current_sha256: None,
                detail: Some("configured template file is missing".to_owned()),
            });
        }
        if let Err(error) = refuse_symlink_path(&self.edit_root, &path) {
            return Some(MarkdownTemplateValidation {
                state: MarkdownTemplateValidationState::Invalid,
                stored_sha256: template.sha256.clone(),
                current_sha256: None,
                detail: Some(error.to_string()),
            });
        }
        let current_sha256 = match sha256_file(&path) {
            Ok(digest) => digest,
            Err(error) => {
                return Some(MarkdownTemplateValidation {
                    state: MarkdownTemplateValidationState::Invalid,
                    stored_sha256: template.sha256.clone(),
                    current_sha256: None,
                    detail: Some(error.to_string()),
                })
            }
        };
        if let Err(error) = validate_markdown_template(&path) {
            return Some(MarkdownTemplateValidation {
                state: MarkdownTemplateValidationState::Invalid,
                stored_sha256: template.sha256.clone(),
                current_sha256: Some(current_sha256),
                detail: Some(error.to_string()),
            });
        }
        Some(MarkdownTemplateValidation {
            state: if current_sha256 == template.sha256 {
                MarkdownTemplateValidationState::Valid
            } else {
                MarkdownTemplateValidationState::Changed
            },
            stored_sha256: template.sha256.clone(),
            current_sha256: Some(current_sha256),
            detail: None,
        })
    }

    pub(crate) fn markdown_template_path_for_export(&self) -> Result<PathBuf> {
        let template = self.markdown_template.as_ref().ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate(
                "no Markdown Word template is configured; select one under Configuration → Document defaults"
                    .to_owned(),
            )
        })?;
        let validation = self
            .markdown_template_validation()
            .expect("configured template has a validation result");
        if validation.state != MarkdownTemplateValidationState::Valid {
            let state = match validation.state {
                MarkdownTemplateValidationState::Valid => unreachable!(),
                MarkdownTemplateValidationState::Changed => {
                    "configured template has changed; replace it under Configuration → Document defaults"
                }
                MarkdownTemplateValidationState::Missing => {
                    "configured template file is missing; select a replacement under Configuration → Document defaults"
                }
                MarkdownTemplateValidationState::Invalid => {
                    "configured template is invalid; select a replacement under Configuration → Document defaults"
                }
            };
            let message = validation
                .detail
                .filter(|detail| !state.contains(detail))
                .map_or_else(|| state.to_owned(), |detail| format!("{state}: {detail}"));
            return Err(DmsError::InvalidMarkdownTemplate(message));
        }
        Ok(self.edit_root.join(&template.relative_path))
    }

    pub(crate) fn is_markdown_template_path(&self, relative_path: &Path) -> bool {
        self.markdown_template
            .as_ref()
            .is_some_and(|template| template.relative_path == relative_path)
    }

    pub(crate) fn validate_markdown_template_record(&self) -> Result<()> {
        let Some(template) = &self.markdown_template else {
            return Ok(());
        };
        super::validate_relative_source_path(&template.relative_path)?;
        if super::is_metadata_path(&template.relative_path) {
            return Err(DmsError::MetadataPath(template.relative_path.clone()));
        }
        if template
            .relative_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("docx"))
        {
            return Err(DmsError::InvalidMarkdownTemplate(
                "configured template path must end in .docx".to_owned(),
            ));
        }
        if template
            .relative_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("~$"))
        {
            return Err(DmsError::OfficeTemporaryFile(
                template.relative_path.clone(),
            ));
        }
        if template.contract_version != MARKDOWN_TEMPLATE_CONTRACT_VERSION {
            return Err(DmsError::InvalidMarkdownTemplate(format!(
                "template contract version {} is unsupported; expected {}",
                template.contract_version, MARKDOWN_TEMPLATE_CONTRACT_VERSION
            )));
        }
        if template.sha256.len() != 64
            || !template
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(DmsError::InvalidMarkdownTemplate(
                "configured template SHA-256 is invalid".to_owned(),
            ));
        }
        if self
            .documents
            .values()
            .any(|document| document.relative_path == template.relative_path)
        {
            return Err(DmsError::TemplateIsControlledDocument(
                template.relative_path.clone(),
            ));
        }
        Ok(())
    }

    fn resolve_template_source(&self, source_path: &Path) -> Result<(PathBuf, PathBuf)> {
        let requested = if source_path.is_absolute() {
            source_path.to_path_buf()
        } else {
            self.edit_root.join(source_path)
        };
        refuse_symlink_path(&self.edit_root, &requested)?;
        let (absolute_path, relative_path) = self.resolve_source_path(&requested)?;
        if absolute_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("docx"))
        {
            return Err(DmsError::InvalidMarkdownTemplate(
                "template source must be a .docx file".to_owned(),
            ));
        }
        Ok((absolute_path, relative_path))
    }
}

pub fn validate_markdown_template(path: &Path) -> Result<MarkdownTemplateContract> {
    let loaded = load_template(path)?;
    let mut package_parts = loaded
        .entries
        .iter()
        .filter(|entry| !entry.is_dir)
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    package_parts.sort();
    Ok(MarkdownTemplateContract {
        contract_version: MARKDOWN_TEMPLATE_CONTRACT_VERSION,
        package_parts,
    })
}

pub fn assemble_markdown_docx(template: &Path, markdown: &str, destination: &Path) -> Result<()> {
    let loaded = load_template(template)?;
    let (_, body) = super::parse_markdown_frontmatter(markdown)?;
    let blocks = parse_markdown_blocks(body)?;
    let rendered = render_blocks(&blocks, &loaded)?;
    let mut document_xml = String::with_capacity(loaded.document_xml.len() + rendered.len());
    document_xml.push_str(&loaded.document_xml[..loaded.prototype_start]);
    document_xml.push_str(&rendered);
    document_xml.push_str(&loaded.document_xml[loaded.prototype_end..]);

    let output = fs::File::create(destination).map_err(|source| DmsError::Io {
        path: destination.to_path_buf(),
        source,
    })?;
    let mut writer = ZipWriter::new(output);
    for entry in loaded.entries {
        let options = SimpleFileOptions::default().compression_method(entry.compression);
        if entry.is_dir {
            writer
                .add_directory(&entry.name, options)
                .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
            continue;
        }
        writer
            .start_file(&entry.name, options)
            .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
        if entry.name == "word/document.xml" {
            writer
                .write_all(document_xml.as_bytes())
                .map_err(|source| DmsError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
        } else {
            writer
                .write_all(&entry.bytes)
                .map_err(|source| DmsError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
        }
    }
    writer
        .finish()
        .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
    validate_docx_package(destination)
}

fn load_template(path: &Path) -> Result<LoadedTemplate> {
    validate_docx_package(path)
        .map_err(|error| DmsError::InvalidMarkdownTemplate(error.to_string()))?;
    let file = fs::File::open(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| DmsError::InvalidMarkdownTemplate(error.to_string()))?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| DmsError::InvalidMarkdownTemplate(error.to_string()))?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| DmsError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        entries.push(PackageEntry {
            name: entry.name().to_owned(),
            bytes,
            compression: entry.compression(),
            is_dir: entry.is_dir(),
        });
    }
    for required in REQUIRED_PARTS {
        if !entries.iter().any(|entry| entry.name == *required) {
            return Err(DmsError::InvalidMarkdownTemplate(format!(
                "required package part {required} is missing"
            )));
        }
    }
    let document_bytes = &entries
        .iter()
        .find(|entry| entry.name == "word/document.xml")
        .expect("required part checked")
        .bytes;
    let document_xml = String::from_utf8(document_bytes.clone()).map_err(|error| {
        DmsError::InvalidMarkdownTemplate(format!("word/document.xml is not UTF-8: {error}"))
    })?;
    let custom_xml = String::from_utf8(
        entries
            .iter()
            .find(|entry| entry.name == "docProps/custom.xml")
            .expect("required part checked")
            .bytes
            .clone(),
    )
    .map_err(|error| {
        DmsError::InvalidMarkdownTemplate(format!("docProps/custom.xml is not UTF-8: {error}"))
    })?;
    for (property, token) in [
        ("DMS_TITLE", "{TITLE}"),
        ("DMS_DOCUMENT_NUMBER", "{DOCUMENT_NUMBER}"),
        ("DMS_VERSION", "{VERSION}"),
        ("DMS_CONFIDENTIALITY", "{CONFIDENTIALITY}"),
    ] {
        let marker = format!("name=\"{property}\"");
        let markers = custom_xml.match_indices(&marker).collect::<Vec<_>>();
        if markers.is_empty() {
            return Err(DmsError::InvalidMarkdownTemplate(format!(
                "custom property {property} is missing"
            )));
        }
        if markers.len() != 1 {
            return Err(DmsError::InvalidMarkdownTemplate(format!(
                "custom property {property} must occur exactly once; found {}",
                markers.len()
            )));
        }
        let marker_offset = markers[0].0;
        let opening_offset = custom_xml[..marker_offset].rfind('<').ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate(format!(
                "custom property {property} has no opening element"
            ))
        })?;
        let opening_end = custom_xml[marker_offset..]
            .find('>')
            .map(|offset| marker_offset + offset + 1)
            .ok_or_else(|| {
                DmsError::InvalidMarkdownTemplate(format!(
                    "custom property {property} has an incomplete opening element"
                ))
            })?;
        let element_name = custom_xml[opening_offset + 1..]
            .split(|character: char| character.is_whitespace() || character == '>')
            .next()
            .unwrap_or_default();
        let closing = format!("</{element_name}>");
        let property_end = custom_xml[opening_end..]
            .find(&closing)
            .map(|offset| opening_end + offset)
            .ok_or_else(|| {
                DmsError::InvalidMarkdownTemplate(format!(
                    "custom property {property} has no closing element"
                ))
            })?;
        let count = custom_xml[opening_end..property_end]
            .match_indices(token)
            .count();
        if count != 1 {
            return Err(DmsError::InvalidMarkdownTemplate(format!(
                "custom property {property} must contain placeholder {token} exactly once; found {count}"
            )));
        }
    }

    let mut paragraphs = BTreeMap::new();
    let mut spans = Vec::new();
    for (name, token) in PARAGRAPH_PROTOTYPES {
        require_unique_token(&document_xml, token)?;
        let span = enclosing_element(&document_xml, token, "<w:p", "</w:p>")?;
        paragraphs.insert((*name).to_owned(), document_xml[span.0..span.1].to_owned());
        spans.push(span);
    }
    require_unique_token(&document_xml, TABLE_COLUMN_1)?;
    require_unique_token(&document_xml, TABLE_COLUMN_2)?;
    let table_span = enclosing_element(&document_xml, TABLE_COLUMN_1, "<w:tbl", "</w:tbl>")?;
    let second_table_span = enclosing_element(&document_xml, TABLE_COLUMN_2, "<w:tbl", "</w:tbl>")?;
    if table_span != second_table_span {
        return Err(DmsError::InvalidMarkdownTemplate(
            "table column prototypes must be in the same table".to_owned(),
        ));
    }
    let table = &document_xml[table_span.0..table_span.1];
    let rows = all_element_spans(table, "<w:tr", "</w:tr>")?;
    if rows.len() != 1 {
        return Err(DmsError::InvalidMarkdownTemplate(format!(
            "table prototype must contain exactly one row; found {}",
            rows.len()
        )));
    }
    let row = &table[rows[0].0..rows[0].1];
    let cells = all_element_spans(row, "<w:tc", "</w:tc>")?;
    if cells.len() != 2 {
        return Err(DmsError::InvalidMarkdownTemplate(format!(
            "table prototype must contain exactly two cells; found {}",
            cells.len()
        )));
    }
    spans.push(table_span);
    spans.sort_unstable_by_key(|span| span.0);
    for pair in spans.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(DmsError::InvalidMarkdownTemplate(
                "template body prototypes overlap".to_owned(),
            ));
        }
    }
    let prototype_start = spans.first().expect("prototype spans").0;
    let prototype_end = spans.last().expect("prototype spans").1;

    Ok(LoadedTemplate {
        entries,
        document_xml: document_xml.clone(),
        prototype_start,
        prototype_end,
        paragraphs,
        table: table.to_owned(),
    })
}

fn validate_docx_package(path: &Path) -> Result<()> {
    let file = fs::File::open(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
    for required in REQUIRED_PARTS {
        archive.by_name(required).map_err(|_| {
            DmsError::InvalidDocx(format!("required package part {required} is missing"))
        })?;
    }
    for index in 0..archive.len() {
        let mut part = archive
            .by_index(index)
            .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
        if part.is_dir() || !(part.name().ends_with(".xml") || part.name().ends_with(".rels")) {
            continue;
        }
        let name = part.name().to_owned();
        let mut bytes = Vec::new();
        part.read_to_end(&mut bytes)
            .map_err(|source| DmsError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        let mut reader = Reader::from_reader(bytes.as_slice());
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(XmlEvent::Eof) => break,
                Ok(_) => buffer.clear(),
                Err(error) => {
                    return Err(DmsError::InvalidDocx(format!(
                        "package part {name} is not well-formed XML: {error}"
                    )))
                }
            }
        }
    }
    Ok(())
}

fn require_unique_token(xml: &str, token: &str) -> Result<()> {
    let count = xml.match_indices(token).count();
    if count != 1 {
        return Err(DmsError::InvalidMarkdownTemplate(format!(
            "body prototype {token} must occur exactly once; found {count}"
        )));
    }
    Ok(())
}

fn enclosing_element(
    xml: &str,
    token: &str,
    opening: &str,
    closing: &str,
) -> Result<(usize, usize)> {
    let position = xml.find(token).expect("token checked");
    let start = find_last_opening_element(&xml[..position], opening).ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate(format!(
            "body prototype {token} has no {opening} ancestor"
        ))
    })?;
    let end = xml[position..]
        .find(closing)
        .map(|offset| position + offset + closing.len())
        .ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate(format!(
                "body prototype {token} has no closing {closing} element"
            ))
        })?;
    Ok((start, end))
}

fn refuse_symlink_path(edit_root: &Path, requested: &Path) -> Result<()> {
    let lexical_root = requested
        .ancestors()
        .find(|ancestor| fs::canonicalize(ancestor).is_ok_and(|canonical| canonical == edit_root))
        .ok_or_else(|| DmsError::OutsideEditRoot(requested.to_path_buf()))?;
    let relative = requested
        .strip_prefix(lexical_root)
        .expect("lexical root is an ancestor");
    let mut current = lexical_root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(DmsError::InvalidRelativePath(relative.to_path_buf()));
        }
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|source| DmsError::Io {
            path: current.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() {
            return Err(DmsError::MarkdownTemplateSymlink(current));
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Inline {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
    link: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Block {
    Heading(u8, Vec<Inline>),
    Paragraph(Vec<Inline>),
    ListItem {
        ordered: bool,
        number: u64,
        inlines: Vec<Inline>,
    },
    Code(Vec<Inline>),
    Table(Vec<Vec<Vec<Inline>>>),
}

fn parse_markdown_blocks(markdown: &str) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut block_kind = None;
    let mut bold_depth = 0usize;
    let mut italic_depth = 0usize;
    let mut code_block = false;
    let mut link_stack = Vec::new();
    let mut list_stack = Vec::new();
    let mut list_number = 0u64;
    let mut table = None::<Vec<Vec<Vec<Inline>>>>;
    let mut row = None::<Vec<Vec<Inline>>>;
    let mut cell = None::<Vec<Inline>>;

    for event in Parser::new_ext(markdown, Options::all()) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current.clear();
                block_kind = Some(BlockKind::Heading(heading_number(level)));
            }
            Event::End(TagEnd::Heading(_)) => {
                finish_block(&mut blocks, &mut block_kind, &mut current)
            }
            Event::Start(Tag::Paragraph) if table.is_none() && block_kind.is_none() => {
                current.clear();
                block_kind = Some(BlockKind::Paragraph);
            }
            Event::End(TagEnd::Paragraph) if table.is_none() => {
                if !matches!(block_kind, Some(BlockKind::ListItem { .. })) {
                    finish_block(&mut blocks, &mut block_kind, &mut current);
                }
            }
            Event::Start(Tag::Emphasis) => italic_depth += 1,
            Event::End(TagEnd::Emphasis) => italic_depth = italic_depth.saturating_sub(1),
            Event::Start(Tag::Strong) => bold_depth += 1,
            Event::End(TagEnd::Strong) => bold_depth = bold_depth.saturating_sub(1),
            Event::Start(Tag::Link { dest_url, .. }) => link_stack.push(dest_url.into_string()),
            Event::End(TagEnd::Link) => {
                if let Some(url) = link_stack.pop() {
                    push_inline(
                        &mut current,
                        &mut cell,
                        Inline {
                            text: format!(" ({url})"),
                            link: true,
                            ..Inline::default()
                        },
                    );
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                current.clear();
                if let CodeBlockKind::Fenced(language) = kind {
                    if !language.is_empty() {
                        current.push(Inline {
                            text: format!("{language}\n"),
                            code: true,
                            ..Inline::default()
                        });
                    }
                }
                block_kind = Some(BlockKind::Code);
                code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                code_block = false;
                finish_block(&mut blocks, &mut block_kind, &mut current);
            }
            Event::Start(Tag::List(start)) => {
                list_stack.push(start);
                list_number = start.unwrap_or(1);
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }
            Event::Start(Tag::Item) => {
                current.clear();
                block_kind = Some(BlockKind::ListItem {
                    ordered: list_stack.last().is_some_and(Option::is_some),
                    number: list_number,
                });
            }
            Event::End(TagEnd::Item) => {
                finish_block(&mut blocks, &mut block_kind, &mut current);
                list_number += 1;
            }
            Event::Start(Tag::Table(_)) => table = Some(Vec::new()),
            Event::End(TagEnd::Table) => {
                if let Some(rows) = table.take() {
                    blocks.push(Block::Table(rows));
                }
            }
            Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => row = Some(Vec::new()),
            Event::End(TagEnd::TableHead) | Event::End(TagEnd::TableRow) => {
                if let (Some(rows), Some(completed)) = (table.as_mut(), row.take()) {
                    rows.push(completed);
                }
            }
            Event::Start(Tag::TableCell) => cell = Some(Vec::new()),
            Event::End(TagEnd::TableCell) => {
                if let (Some(cells), Some(completed)) = (row.as_mut(), cell.take()) {
                    cells.push(completed);
                }
            }
            Event::Text(text) => push_inline(
                &mut current,
                &mut cell,
                Inline {
                    text: text.into_string(),
                    bold: bold_depth > 0,
                    italic: italic_depth > 0,
                    code: code_block,
                    link: !link_stack.is_empty(),
                },
            ),
            Event::Code(text) => push_inline(
                &mut current,
                &mut cell,
                Inline {
                    text: text.into_string(),
                    bold: bold_depth > 0,
                    italic: italic_depth > 0,
                    code: true,
                    link: !link_stack.is_empty(),
                },
            ),
            Event::SoftBreak | Event::HardBreak => push_inline(
                &mut current,
                &mut cell,
                Inline {
                    text: "\n".to_owned(),
                    ..Inline::default()
                },
            ),
            Event::Rule => blocks.push(Block::Paragraph(vec![Inline {
                text: "—".to_owned(),
                ..Inline::default()
            }])),
            _ => {}
        }
    }
    if block_kind.is_some() {
        finish_block(&mut blocks, &mut block_kind, &mut current);
    }
    Ok(blocks)
}

#[derive(Clone, Copy, Debug)]
enum BlockKind {
    Heading(u8),
    Paragraph,
    ListItem { ordered: bool, number: u64 },
    Code,
}

fn finish_block(blocks: &mut Vec<Block>, kind: &mut Option<BlockKind>, inlines: &mut Vec<Inline>) {
    let Some(kind) = kind.take() else {
        return;
    };
    let inlines = std::mem::take(inlines);
    blocks.push(match kind {
        BlockKind::Heading(level) => Block::Heading(level, inlines),
        BlockKind::Paragraph => Block::Paragraph(inlines),
        BlockKind::ListItem { ordered, number } => Block::ListItem {
            ordered,
            number,
            inlines,
        },
        BlockKind::Code => Block::Code(inlines),
    });
}

fn push_inline(current: &mut Vec<Inline>, cell: &mut Option<Vec<Inline>>, value: Inline) {
    if let Some(cell) = cell.as_mut() {
        cell.push(value);
    } else {
        current.push(value);
    }
}

fn heading_number(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 4,
    }
}

fn render_blocks(blocks: &[Block], template: &LoadedTemplate) -> Result<String> {
    let mut output = String::new();
    for block in blocks {
        match block {
            Block::Heading(level, inlines) => output.push_str(&render_paragraph(
                template
                    .paragraphs
                    .get(&format!("heading{level}"))
                    .expect("validated heading prototype"),
                inlines,
                None,
            )?),
            Block::Paragraph(inlines) => output.push_str(&render_paragraph(
                template
                    .paragraphs
                    .get("paragraph")
                    .expect("validated paragraph"),
                inlines,
                None,
            )?),
            Block::ListItem {
                ordered,
                number,
                inlines,
            } => output.push_str(&render_paragraph(
                template.paragraphs.get("list").expect("validated list"),
                inlines,
                ordered.then(|| format!("{number}. ")).as_deref(),
            )?),
            Block::Code(inlines) => {
                let mut inlines = inlines.clone();
                for inline in &mut inlines {
                    inline.code = true;
                }
                output.push_str(&render_paragraph(
                    template
                        .paragraphs
                        .get("paragraph")
                        .expect("validated paragraph"),
                    &inlines,
                    None,
                )?);
            }
            Block::Table(rows) => output.push_str(&render_table(&template.table, rows)?),
        }
    }
    Ok(output)
}

fn render_paragraph(prototype: &str, inlines: &[Inline], prefix: Option<&str>) -> Result<String> {
    let opening_end = prototype.find('>').ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate("paragraph prototype has no opening tag".to_owned())
    })? + 1;
    let closing = prototype.rfind("</w:p>").ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate("paragraph prototype has no closing tag".to_owned())
    })?;
    let properties = prototype[opening_end..closing]
        .find("<w:pPr")
        .and_then(|start| {
            prototype[opening_end + start..closing]
                .find("</w:pPr>")
                .map(|end| {
                    &prototype[opening_end + start..opening_end + start + end + "</w:pPr>".len()]
                })
        })
        .unwrap_or_default();
    let mut output = String::new();
    output.push_str(&prototype[..opening_end]);
    output.push_str(properties);
    if let Some(prefix) = prefix {
        output.push_str(&render_run(&Inline {
            text: prefix.to_owned(),
            ..Inline::default()
        }));
    }
    for inline in inlines {
        output.push_str(&render_run(inline));
    }
    output.push_str("</w:p>");
    Ok(output)
}

fn render_run(inline: &Inline) -> String {
    let mut properties = String::new();
    if inline.bold {
        properties.push_str("<w:b/>");
    }
    if inline.italic {
        properties.push_str("<w:i/>");
    }
    if inline.code {
        properties.push_str("<w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>");
    }
    if inline.link {
        properties.push_str("<w:color w:val=\"0563C1\"/><w:u w:val=\"single\"/>");
    }
    let text = xml_escape(&inline.text).replace('\n', "</w:t><w:br/><w:t xml:space=\"preserve\">");
    if properties.is_empty() {
        format!("<w:r><w:t xml:space=\"preserve\">{text}</w:t></w:r>")
    } else {
        format!("<w:r><w:rPr>{properties}</w:rPr><w:t xml:space=\"preserve\">{text}</w:t></w:r>")
    }
}

fn render_table(prototype: &str, rows: &[Vec<Vec<Inline>>]) -> Result<String> {
    let row_span = element_span(prototype, "<w:tr", "</w:tr>")?;
    let row_prototype = &prototype[row_span.0..row_span.1];
    let cells = all_element_spans(row_prototype, "<w:tc", "</w:tc>")?;
    if cells.len() != 2 {
        return Err(DmsError::InvalidMarkdownTemplate(format!(
            "table prototype must contain exactly two cells; found {}",
            cells.len()
        )));
    }
    let mut rendered_rows = String::new();
    for row in rows {
        if row.len() != 2 {
            return Err(DmsError::InvalidMarkdownTemplate(format!(
                "Markdown tables must contain exactly two columns; found {}",
                row.len()
            )));
        }
        let mut rendered = row_prototype.to_owned();
        rendered = rendered.replace(TABLE_COLUMN_1, &plain_inline_text(&row[0]));
        rendered = rendered.replace(TABLE_COLUMN_2, &plain_inline_text(&row[1]));
        rendered_rows.push_str(&rendered);
    }
    Ok(format!(
        "{}{}{}",
        &prototype[..row_span.0],
        rendered_rows,
        &prototype[row_span.1..]
    ))
}

fn element_span(value: &str, opening: &str, closing: &str) -> Result<(usize, usize)> {
    let start = find_next_opening_element(value, opening, 0).ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate(format!("prototype has no {opening} element"))
    })?;
    let end = value[start..]
        .find(closing)
        .map(|offset| start + offset + closing.len())
        .ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate(format!("prototype has no closing {closing} element"))
        })?;
    Ok((start, end))
}

fn all_element_spans(value: &str, opening: &str, closing: &str) -> Result<Vec<(usize, usize)>> {
    let mut spans = Vec::new();
    let mut offset = 0;
    while let Some(start) = find_next_opening_element(value, opening, offset) {
        let end = value[start..]
            .find(closing)
            .map(|end| start + end + closing.len())
            .ok_or_else(|| {
                DmsError::InvalidMarkdownTemplate(format!(
                    "prototype has no closing {closing} element"
                ))
            })?;
        spans.push((start, end));
        offset = end;
    }
    Ok(spans)
}

fn find_last_opening_element(value: &str, opening: &str) -> Option<usize> {
    value
        .match_indices(opening)
        .map(|(position, _)| position)
        .filter(|position| is_element_boundary(value, *position + opening.len()))
        .last()
}

fn find_next_opening_element(value: &str, opening: &str, offset: usize) -> Option<usize> {
    value[offset..]
        .match_indices(opening)
        .map(|(position, _)| offset + position)
        .find(|position| is_element_boundary(value, *position + opening.len()))
}

fn is_element_boundary(value: &str, position: usize) -> bool {
    value
        .as_bytes()
        .get(position)
        .is_some_and(|byte| *byte == b'>' || *byte == b'/' || byte.is_ascii_whitespace())
}

fn plain_inline_text(inlines: &[Inline]) -> String {
    xml_escape(
        &inlines
            .iter()
            .map(|inline| inline.text.as_str())
            .collect::<String>(),
    )
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
