use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use quick_xml::{
    events::Event as XmlEvent,
    name::{Namespace, ResolveResult},
    reader::NsReader,
    Reader,
};
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
const WORDPROCESSINGML_MAIN_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const TABLE_OF_CONTENTS_GALLERY: &str = "Table of Contents";

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

/// One embedded Markdown asset: the asset path relative to the assembly input
/// root plus the SHA-256 digest of the exact bytes written into the DOCX
/// package. Callers use it to prove which asset bytes were actually consumed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkdownAssetDigest {
    #[serde(with = "super::relative_path_serde")]
    pub relative_path: PathBuf,
    pub sha256: String,
}

/// Explicit source and containment context for resolving local Markdown image
/// references. `input_root` is the only directory an asset may live in;
/// `source_path` is the Markdown document the references are relative to.
#[derive(Clone, Debug)]
pub struct MarkdownAssetContext {
    input_root: PathBuf,
    source_path: PathBuf,
}

impl MarkdownAssetContext {
    pub fn new(input_root: impl Into<PathBuf>, source_path: impl Into<PathBuf>) -> Self {
        Self {
            input_root: input_root.into(),
            source_path: source_path.into(),
        }
    }

    pub fn input_root(&self) -> &Path {
        &self.input_root
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    fn canonical_input_root(&self) -> Result<PathBuf> {
        if !self.input_root.is_dir() {
            return Err(DmsError::InvalidMarkdownImage(format!(
                "input root {} is not a directory",
                self.input_root.display()
            )));
        }
        fs::canonicalize(&self.input_root).map_err(|source| DmsError::Io {
            path: self.input_root.clone(),
            source,
        })
    }

    fn canonical_source_directory(&self) -> Result<PathBuf> {
        let directory = self
            .source_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::canonicalize(directory).map_err(|source| DmsError::Io {
            path: directory.to_path_buf(),
            source,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageFormat {
    Png,
    Jpeg,
}

impl ImageFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }
}

#[derive(Clone, Debug)]
struct EmbeddedImage {
    part_name: String,
    relationship_id: String,
    bytes: Vec<u8>,
    format: ImageFormat,
    width: u32,
    height: u32,
}

/// Per-assembly image state: resolved assets, allocated package parts and
/// drawing identity, and the printable-width limit used for sizing.
struct RenderContext<'a> {
    template: &'a LoadedTemplate,
    assets: Option<&'a MarkdownAssetContext>,
    input_root: Option<PathBuf>,
    source_directory: Option<PathBuf>,
    printable_width_emu: Option<u64>,
    media: Vec<EmbeddedImage>,
    media_by_path: BTreeMap<PathBuf, usize>,
    taken_parts: BTreeSet<String>,
    next_media: u32,
    next_relationship: u32,
    next_drawing: u64,
}

impl<'a> RenderContext<'a> {
    fn new(template: &'a LoadedTemplate, assets: Option<&'a MarkdownAssetContext>) -> Result<Self> {
        let mut taken_parts = BTreeSet::new();
        for entry in &template.entries {
            taken_parts.insert(entry.name.clone());
        }
        let relationships = template
            .entries
            .iter()
            .find(|entry| entry.name == "word/_rels/document.xml.rels")
            .and_then(|entry| String::from_utf8(entry.bytes.clone()).ok())
            .unwrap_or_default();
        Ok(Self {
            template,
            assets,
            input_root: None,
            source_directory: None,
            printable_width_emu: None,
            media: Vec::new(),
            media_by_path: BTreeMap::new(),
            taken_parts,
            next_media: 1,
            next_relationship: max_relationship_id(&relationships) + 1,
            next_drawing: u64::from(max_drawing_id(&template.entries)?) + 1,
        })
    }

    fn printable_width(&mut self) -> Result<u64> {
        if let Some(width) = self.printable_width_emu {
            return Ok(width);
        }
        // One twip is 635 EMU; a drawing may not exceed the body text column.
        let width = u64::from(table_printable_width(self.template)?) * 635;
        self.printable_width_emu = Some(width);
        Ok(width)
    }

    /// SHA-256 digests of the asset bytes actually embedded into the package,
    /// keyed by path relative to the canonical input root. Repeated references
    /// share one digest; the result is ordered by relative path.
    fn asset_digests(&self) -> Vec<MarkdownAssetDigest> {
        let root = self.input_root.as_deref();
        let mut digests = self
            .media_by_path
            .iter()
            .map(|(path, index)| {
                let relative_path = root
                    .and_then(|root| path.strip_prefix(root).ok())
                    .map_or_else(|| portable_relative_path(path), portable_relative_path);
                MarkdownAssetDigest {
                    relative_path,
                    sha256: format!("{:x}", Sha256::digest(&self.media[*index].bytes)),
                }
            })
            .collect::<Vec<_>>();
        digests.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        digests
    }

    /// Resolve one Markdown image reference to a canonical file inside the
    /// explicit input root, or fail with the exact rejection reason.
    fn resolve_asset(&mut self, reference: &str) -> Result<PathBuf> {
        if self.assets.is_none() {
            return Err(DmsError::InvalidMarkdownImage(format!(
                "no source/asset resolution context was provided for image reference `{reference}`"
            )));
        }
        let relative = Path::new(&decode_reference(reference)?).to_path_buf();
        if relative.is_absolute() {
            return Err(DmsError::InvalidMarkdownImage(format!(
                "image reference `{reference}` is an absolute path"
            )));
        }
        if relative
            .components()
            .any(|component| matches!(component, Component::RootDir | Component::Prefix(_)))
        {
            return Err(DmsError::InvalidMarkdownImage(format!(
                "image reference `{reference}` is not a relative path"
            )));
        }
        let root = match &self.input_root {
            Some(root) => root.clone(),
            None => {
                let root = self
                    .assets
                    .expect("context checked")
                    .canonical_input_root()?;
                self.input_root = Some(root.clone());
                root
            }
        };
        let directory = match &self.source_directory {
            Some(directory) => directory.clone(),
            None => {
                let directory = self
                    .assets
                    .expect("context checked")
                    .canonical_source_directory()?;
                self.source_directory = Some(directory.clone());
                directory
            }
        };
        let candidate = directory.join(&relative);
        let canonical = fs::canonicalize(&candidate).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                DmsError::InvalidMarkdownImage(format!(
                    "image asset {} does not exist",
                    candidate.display()
                ))
            } else {
                DmsError::Io {
                    path: candidate.clone(),
                    source: error,
                }
            }
        })?;
        if !canonical.starts_with(&root) {
            return Err(DmsError::InvalidMarkdownImage(format!(
                "image asset {} resolves outside the input root {}",
                canonical.display(),
                root.display()
            )));
        }
        if !canonical.is_file() {
            return Err(DmsError::InvalidMarkdownImage(format!(
                "image asset {} is not a regular file",
                canonical.display()
            )));
        }
        Ok(canonical)
    }

    fn embed_asset(&mut self, path: &Path) -> Result<usize> {
        if let Some(index) = self.media_by_path.get(path) {
            return Ok(*index);
        }
        let bytes = fs::read(path).map_err(|source| DmsError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let (format, width, height) = image_dimensions(path, &bytes)?;
        let relationship_id = format!("rId{}", self.next_relationship);
        self.next_relationship += 1;
        let part_name = loop {
            let candidate = format!(
                "word/media/dms-image-{}.{}",
                self.next_media,
                format.extension()
            );
            self.next_media += 1;
            if !self.taken_parts.contains(&candidate) {
                break candidate;
            }
        };
        self.taken_parts.insert(part_name.clone());
        self.media.push(EmbeddedImage {
            part_name,
            relationship_id,
            bytes,
            format,
            width,
            height,
        });
        let index = self.media.len() - 1;
        self.media_by_path.insert(path.to_path_buf(), index);
        Ok(index)
    }

    fn image_run(&mut self, image: &ImageInline) -> Result<String> {
        let path = self.resolve_asset(&image.reference)?;
        let index = self.embed_asset(&path)?;
        let limit = self.printable_width()?;
        let drawing_id = u32::try_from(self.next_drawing).map_err(|_| {
            DmsError::InvalidMarkdownImage(
                "no unused drawing IDs remain in the template package".to_owned(),
            )
        })?;
        self.next_drawing += 1;
        let embedded = &self.media[index];
        // Native size assumes 96 DPI (9525 EMU per pixel) and is never upscaled.
        let native_width = u64::from(embedded.width) * 9525;
        let width = native_width.min(limit);
        let height = width * u64::from(embedded.height) / u64::from(embedded.width);
        let description = if image.alt_text.trim().is_empty() {
            image.reference.clone()
        } else {
            image.alt_text.clone()
        };
        Ok(render_image_run(
            &embedded.relationship_id,
            drawing_id,
            width,
            height,
            &description,
        ))
    }
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

#[derive(Clone, Debug)]
struct OrderedListNumbering {
    xml: String,
    existed_in_template: bool,
    num_ids: BTreeMap<usize, u32>,
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

/// Assemble one Markdown body into the workspace Word template. Local image
/// references are embedded as OOXML drawings; `assets` is the explicit
/// source/containment context and its absence rejects asset-bearing input.
/// The template's Table of Contents is retained.
pub fn assemble_markdown_docx(
    template: &Path,
    markdown: &str,
    destination: &Path,
    assets: Option<&MarkdownAssetContext>,
) -> Result<()> {
    assemble_markdown_docx_with_options(template, markdown, destination, assets, false).map(|_| ())
}

/// Assemble one Markdown body into the workspace Word template, optionally
/// removing the template's Table of Contents structured-document tag
/// (`without_toc`). Returns the digests of the asset bytes actually embedded
/// into the output package so callers can prove which bytes were consumed.
pub fn assemble_markdown_docx_with_options(
    template: &Path,
    markdown: &str,
    destination: &Path,
    assets: Option<&MarkdownAssetContext>,
    without_toc: bool,
) -> Result<Vec<MarkdownAssetDigest>> {
    assemble_markdown(template, markdown, destination, assets, without_toc, false)
}

pub(crate) fn assemble_markdown_conversion_docx(
    template: &Path,
    markdown: &str,
    destination: &Path,
    assets: Option<&MarkdownAssetContext>,
    without_toc: bool,
) -> Result<Vec<MarkdownAssetDigest>> {
    assemble_markdown(template, markdown, destination, assets, without_toc, true)
}

fn assemble_markdown(
    template: &Path,
    markdown: &str,
    destination: &Path,
    assets: Option<&MarkdownAssetContext>,
    without_toc: bool,
    conversion: bool,
) -> Result<Vec<MarkdownAssetDigest>> {
    let loaded = load_template(template)?;
    let (frontmatter, body) = if conversion {
        super::frontmatter::parse_conversion_frontmatter(markdown)?
    } else {
        super::parse_markdown_frontmatter(markdown)?
    };
    let mut variables = frontmatter.template_variables();
    if conversion
        && loaded.entries.iter().any(|entry| {
            (entry.name.ends_with(".xml") || entry.name.ends_with(".rels"))
                && entry.bytes.windows(b"{DATE}".len()).any(|w| w == b"{DATE}")
        })
    {
        let date = frontmatter
            .variables
            .get("last_modified")
            .filter(|value| {
                value.len() == 10
                    && value.bytes().enumerate().all(|(i, b)| {
                        if i == 4 || i == 7 {
                            b == b'-'
                        } else {
                            b.is_ascii_digit()
                        }
                    })
                    && chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
            })
            .ok_or(DmsError::InvalidConversionDate)?;
        // Conversion-only alias wins over an unrelated optional `date` key.
        variables.insert("{DATE}".to_owned(), date.clone());
    }
    let blocks = parse_markdown_blocks(body)?;
    let ordered_list_numbering = ordered_list_numbering(&loaded, &blocks)?;
    let mut context = RenderContext::new(&loaded, assets)?;
    let rendered = render_blocks(&blocks, &mut context, ordered_list_numbering.as_ref())?;
    let mut document_xml = String::with_capacity(loaded.document_xml.len() + rendered.len());
    document_xml.push_str(&loaded.document_xml[..loaded.prototype_start]);
    document_xml.push_str(&rendered);
    document_xml.push_str(&loaded.document_xml[loaded.prototype_end..]);

    let document_xml = apply_template_variables(&document_xml, &variables);
    let document_xml = if without_toc {
        remove_table_of_contents_sdt(&document_xml)?
    } else {
        document_xml
    };
    let asset_digests = context.asset_digests();
    let media = std::mem::take(&mut context.media);
    let relationships = if media.is_empty() {
        None
    } else {
        let current = loaded
            .entries
            .iter()
            .find(|entry| entry.name == "word/_rels/document.xml.rels")
            .expect("required part checked");
        let text = String::from_utf8(current.bytes.clone()).map_err(|error| {
            DmsError::InvalidMarkdownTemplate(format!(
                "package part word/_rels/document.xml.rels is not UTF-8: {error}"
            ))
        })?;
        Some(apply_template_variables(
            &relationships_with_media(&text, &media)?,
            &variables,
        ))
    };
    let content_types = if media.is_empty() {
        None
    } else {
        let current = loaded
            .entries
            .iter()
            .find(|entry| entry.name == "[Content_Types].xml")
            .expect("required part checked");
        let text = String::from_utf8(current.bytes.clone()).map_err(|error| {
            DmsError::InvalidMarkdownTemplate(format!(
                "package part [Content_Types].xml is not UTF-8: {error}"
            ))
        })?;
        let formats = media.iter().map(|image| image.format).collect::<Vec<_>>();
        Some(apply_template_variables(
            &content_types_with_media(&text, &formats)?,
            &variables,
        ))
    };
    let relationships = if let Some(_numbering) = &ordered_list_numbering {
        let source = relationships
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                String::from_utf8(
                    loaded
                        .entries
                        .iter()
                        .find(|entry| entry.name == "word/_rels/document.xml.rels")
                        .expect("required part checked")
                        .bytes
                        .clone(),
                )
                .expect("validated template relationship XML is UTF-8")
            });
        Some(relationships_with_numbering(&source)?)
    } else {
        relationships
    };
    let content_types = if ordered_list_numbering.is_some() {
        let source = content_types
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| {
                String::from_utf8(
                    loaded
                        .entries
                        .iter()
                        .find(|entry| entry.name == "[Content_Types].xml")
                        .expect("required part checked")
                        .bytes
                        .clone(),
                )
                .expect("validated template content-types XML is UTF-8")
            });
        Some(content_types_with_numbering(&source)?)
    } else {
        content_types
    };

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
        let replaced = match entry.name.as_str() {
            "word/document.xml" => Some(document_xml.clone()),
            "word/_rels/document.xml.rels" => relationships.clone(),
            "[Content_Types].xml" => content_types.clone(),
            "word/numbering.xml" => ordered_list_numbering
                .as_ref()
                .map(|numbering| numbering.xml.clone()),
            _ => None,
        };
        writer
            .start_file(&entry.name, options)
            .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
        if let Some(replaced) = replaced {
            writer
                .write_all(replaced.as_bytes())
                .map_err(|source| DmsError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
        } else if entry.name.ends_with(".xml") || entry.name.ends_with(".rels") {
            let text = String::from_utf8(entry.bytes).map_err(|error| {
                DmsError::InvalidMarkdownTemplate(format!(
                    "package part {} is not UTF-8: {error}",
                    entry.name
                ))
            })?;
            let filled = apply_template_variables(&text, &variables);
            writer
                .write_all(filled.as_bytes())
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
    if let Some(numbering) = &ordered_list_numbering {
        if !numbering.existed_in_template {
            writer
                .start_file("word/numbering.xml", SimpleFileOptions::default())
                .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
            writer
                .write_all(numbering.xml.as_bytes())
                .map_err(|source| DmsError::Io {
                    path: destination.to_path_buf(),
                    source,
                })?;
        }
    }
    for image in &media {
        writer
            .start_file(&image.part_name, SimpleFileOptions::default())
            .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
        writer
            .write_all(&image.bytes)
            .map_err(|source| DmsError::Io {
                path: destination.to_path_buf(),
                source,
            })?;
    }
    writer
        .finish()
        .map_err(|error| DmsError::InvalidDocx(error.to_string()))?;
    validate_docx_package(destination)?;
    Ok(asset_digests)
}

fn apply_template_variables(xml: &str, variables: &BTreeMap<String, String>) -> String {
    if variables.is_empty() {
        return xml.to_owned();
    }
    let mut output = xml.to_owned();
    for (token, value) in variables {
        let escaped = xml_escape(value);
        output = output.replace(token, &escaped);
    }
    output
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

/// Decode one Markdown image destination into a relative filesystem path,
/// rejecting remote, absolute, fragment-bearing and empty references.
fn decode_reference(reference: &str) -> Result<String> {
    if reference.is_empty() {
        return Err(DmsError::InvalidMarkdownImage(
            "image reference is empty".to_owned(),
        ));
    }
    if !reference.starts_with("//") {
        if let Some(colon) = reference.find(':') {
            let scheme = &reference[..colon];
            let valid_scheme = !scheme.is_empty()
                && scheme.starts_with(|character: char| character.is_ascii_alphabetic())
                && scheme.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "+-.".contains(character)
                });
            if valid_scheme {
                return Err(DmsError::InvalidMarkdownImage(format!(
                    "image reference `{reference}` is a remote or unsupported reference"
                )));
            }
        }
    }
    if reference.contains('#') || reference.contains('?') {
        return Err(DmsError::InvalidMarkdownImage(format!(
            "image reference `{reference}` carries a fragment or query and is unsupported"
        )));
    }
    if !reference.contains('%') {
        return Ok(reference.to_owned());
    }
    let bytes = reference.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let decoded = std::str::from_utf8(&bytes[index + 1..index + 3])
                .ok()
                .and_then(|pair| u8::from_str_radix(pair, 16).ok());
            if let Some(value) = decoded {
                output.push(value);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(output).map_err(|_| {
        DmsError::InvalidMarkdownImage(format!(
            "image reference `{reference}` is not valid UTF-8 after percent decoding"
        ))
    })
}

/// Check PNG/JPEG structure and read pixel dimensions, independent of extension.
/// These checks reject incomplete containers, not every invalid compressed pixel
/// stream; they do not replace a decoder or installed-Word rendering verification.
fn image_dimensions(path: &Path, bytes: &[u8]) -> Result<(ImageFormat, u32, u32)> {
    if bytes.starts_with(PNG_SIGNATURE) {
        return png_dimensions(path, bytes)
            .map(|(width, height)| (ImageFormat::Png, width, height));
    }
    if bytes.starts_with(&[0xFF, 0xD8]) {
        return jpeg_dimensions(path, bytes)
            .map(|(width, height)| (ImageFormat::Jpeg, width, height));
    }
    Err(DmsError::InvalidMarkdownImage(format!(
        "image asset {} has an unsupported image format; only PNG and JPEG are supported",
        path.display()
    )))
}

const PNG_SIGNATURE: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const JPEG_END: &[u8] = &[0xFF, 0xD9];

fn png_dimensions(path: &Path, bytes: &[u8]) -> Result<(u32, u32)> {
    let corrupt = |detail: &str| {
        DmsError::InvalidMarkdownImage(format!(
            "image asset {} is corrupt: {detail}",
            path.display()
        ))
    };
    if bytes.len() < 33 || &bytes[8..16] != b"\0\0\0\rIHDR" {
        return Err(corrupt("the PNG header chunk is missing"));
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width == 0 || height == 0 {
        return Err(corrupt("the PNG header declares an empty image"));
    }
    let valid_depth = match bytes[25] {
        0 => matches!(bytes[24], 1 | 2 | 4 | 8 | 16),
        2 | 4 | 6 => matches!(bytes[24], 8 | 16),
        3 => matches!(bytes[24], 1 | 2 | 4 | 8),
        _ => false,
    };
    if !valid_depth || bytes[26] != 0 || bytes[27] != 0 || bytes[28] > 1 {
        return Err(corrupt("the PNG header has invalid encoding fields"));
    }
    let mut offset = PNG_SIGNATURE.len();
    let mut has_data = false;
    while bytes.len() - offset >= 12 {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        if length > bytes.len() - offset - 12 {
            return Err(corrupt("a PNG chunk is truncated"));
        }
        let end = offset + 8 + length;
        let kind = &bytes[offset + 4..offset + 8];
        let expected_crc = u32::from_be_bytes(bytes[end..end + 4].try_into().unwrap());
        if png_crc32(&bytes[offset + 4..end]) != expected_crc {
            return Err(corrupt("a PNG chunk checksum is invalid"));
        }
        match kind {
            b"IHDR" if offset != PNG_SIGNATURE.len() => {
                return Err(corrupt("the PNG header is duplicated"));
            }
            b"IDAT" => has_data |= length > 0,
            b"IEND" => {
                if length != 0 || end + 4 != bytes.len() || !has_data {
                    return Err(corrupt("the PNG has no image data or an invalid end chunk"));
                }
                return Ok((width, height));
            }
            _ => {}
        }
        offset = end + 4;
    }
    Err(corrupt("the PNG end-of-image chunk is missing"))
}

fn png_crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn jpeg_dimensions(path: &Path, bytes: &[u8]) -> Result<(u32, u32)> {
    let corrupt = |detail: &str| {
        DmsError::InvalidMarkdownImage(format!(
            "image asset {} is corrupt: {detail}",
            path.display()
        ))
    };
    if !bytes.ends_with(JPEG_END) {
        return Err(corrupt("the JPEG end-of-image marker is missing"));
    }
    let mut index = 2;
    let mut dimensions = None;
    let mut has_scan = false;
    while index + 4 <= bytes.len() {
        if bytes[index] != 0xFF {
            return Err(corrupt("the JPEG segment marker is missing"));
        }
        let marker = bytes[index + 1];
        if marker == 0xFF {
            index += 1;
            continue;
        }
        if marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            index += 2;
            continue;
        }
        let length = usize::from(u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]));
        if length < 2 || index + 2 + length > bytes.len() {
            return Err(corrupt("a JPEG segment length is invalid"));
        }
        let is_start_of_frame = matches!(
            marker,
            0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF
        );
        if is_start_of_frame {
            if length < 8 || length != 8 + 3 * usize::from(bytes[index + 9]) {
                return Err(corrupt("the JPEG frame header length is invalid"));
            }
            let height = u32::from(u16::from_be_bytes([bytes[index + 5], bytes[index + 6]]));
            let width = u32::from(u16::from_be_bytes([bytes[index + 7], bytes[index + 8]]));
            if width == 0 || height == 0 {
                return Err(corrupt("the JPEG frame header declares an empty image"));
            }
            dimensions = Some((width, height));
        }
        index += 2 + length;
        if marker == 0xDA {
            let start = index - length;
            if dimensions.is_none()
                || length < 6
                || bytes[start + 2] == 0
                || length != 6 + 2 * usize::from(bytes[start + 2])
            {
                return Err(corrupt("the JPEG scan header is invalid"));
            }
            // Walk entropy-coded data through byte stuffing and restart markers,
            // stopping at the next segment (including another progressive scan).
            let mut has_data = false;
            while index + 1 < bytes.len() {
                if bytes[index] != 0xFF {
                    has_data = true;
                    index += 1;
                } else if bytes[index + 1] == 0 {
                    has_data = true;
                    index += 2;
                } else if (0xD0..=0xD7).contains(&bytes[index + 1]) {
                    index += 2;
                } else if bytes[index + 1] == 0xFF {
                    index += 1;
                } else {
                    break;
                }
            }
            if !has_data {
                return Err(corrupt("the JPEG scan has no image data"));
            }
            has_scan = true;
        }
    }
    if has_scan && bytes.get(index..) == Some(JPEG_END) {
        return dimensions.ok_or_else(|| corrupt("no JPEG frame header was found"));
    }
    Err(corrupt("the JPEG image data or end marker is missing"))
}

/// One inline drawing referencing an embedded media part, sized in EMU with a
/// description that carries the Markdown alternative text.
fn render_image_run(
    relationship_id: &str,
    drawing_id: u32,
    width: u64,
    height: u64,
    description: &str,
) -> String {
    let description = xml_escape(description);
    format!(
        concat!(
            "<w:r><w:drawing><wp:inline ",
            "xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" ",
            "distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">",
            "<wp:extent cx=\"{width}\" cy=\"{height}\"/>",
            "<wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>",
            "<wp:docPr id=\"{drawing_id}\" name=\"Bild {drawing_id}\" descr=\"{description}\"/>",
            "<wp:cNvGraphicFramePr><a:graphicFrameLocks ",
            "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" noChangeAspect=\"1\"/>",
            "</wp:cNvGraphicFramePr><a:graphic ",
            "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">",
            "<a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">",
            "<pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">",
            "<pic:nvPicPr><pic:cNvPr id=\"{drawing_id}\" name=\"Bild {drawing_id}\" ",
            "descr=\"{description}\"/><pic:cNvPicPr/></pic:nvPicPr>",
            "<pic:blipFill><a:blip ",
            "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" ",
            "r:embed=\"{relationship_id}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>",
            "<pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{width}\" cy=\"{height}\"/>",
            "</a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr>",
            "</pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"
        ),
        width = width,
        height = height,
        drawing_id = drawing_id,
        description = description,
        relationship_id = relationship_id
    )
}

fn max_relationship_id(relationships: &str) -> u32 {
    let mut maximum = 0;
    for (position, _) in relationships.match_indices("Id=\"rId") {
        let digits = relationships[position + "Id=\"rId".len()..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if let Ok(value) = digits.parse::<u32>() {
            maximum = maximum.max(value);
        }
    }
    maximum
}

fn max_drawing_id(entries: &[PackageEntry]) -> Result<u32> {
    let mut maximum = 0;
    // Drawing identities also occur in preserved story parts, not just the body.
    for entry in entries
        .iter()
        .filter(|entry| !entry.is_dir && entry.name.ends_with(".xml"))
    {
        let invalid = |error| {
            DmsError::InvalidMarkdownTemplate(format!(
                "package part {} has invalid drawing XML: {error}",
                entry.name
            ))
        };
        let mut reader = NsReader::from_reader(entry.bytes.as_slice());
        loop {
            match reader
                .read_resolved_event()
                .map_err(|error| invalid(error.to_string()))?
            {
                (
                    ResolveResult::Bound(Namespace(namespace)),
                    XmlEvent::Start(element) | XmlEvent::Empty(element),
                ) if matches!(
                    (namespace, element.local_name().as_ref()),
                    (
                        b"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing",
                        b"docPr"
                    ) | (
                        b"http://schemas.openxmlformats.org/drawingml/2006/picture",
                        b"cNvPr"
                    )
                ) =>
                {
                    for attribute in element.attributes() {
                        let attribute = attribute.map_err(|error| invalid(error.to_string()))?;
                        if attribute.key.as_ref() == b"id" {
                            let value = attribute
                                .unescape_value()
                                .map_err(|error| invalid(error.to_string()))?;
                            if let Ok(value) = value.parse::<u32>() {
                                maximum = maximum.max(value);
                            }
                        }
                    }
                }
                (_, XmlEvent::Eof) => break,
                _ => {}
            }
        }
    }
    Ok(maximum)
}

fn relationships_with_media(relationships: &str, media: &[EmbeddedImage]) -> Result<String> {
    if media.is_empty() {
        return Ok(relationships.to_owned());
    }
    let mut additions = String::new();
    for image in media {
        let target = image
            .part_name
            .strip_prefix("word/")
            .expect("media parts live below word/");
        additions.push_str(&format!(
            concat!(
                "<Relationship Id=\"{id}\" ",
                "Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" ",
                "Target=\"{target}\"/>"
            ),
            id = image.relationship_id,
            target = target
        ));
    }
    let end = relationships.rfind("</Relationships>").ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate(
            "package part word/_rels/document.xml.rels has no closing Relationships element"
                .to_owned(),
        )
    })?;
    Ok(format!(
        "{}{}{}",
        &relationships[..end],
        additions,
        &relationships[end..]
    ))
}

fn content_types_with_media(content_types: &str, formats: &[ImageFormat]) -> Result<String> {
    if formats.is_empty() {
        return Ok(content_types.to_owned());
    }
    let mut declared = BTreeMap::new();
    let mut reader = Reader::from_str(content_types);
    loop {
        match reader.read_event() {
            Ok(XmlEvent::Start(element) | XmlEvent::Empty(element))
                if element.name().as_ref() == b"Default" =>
            {
                let mut extension = None;
                let mut content_type = None;
                for attribute in element.attributes() {
                    let attribute = attribute.map_err(|error| {
                        DmsError::InvalidMarkdownTemplate(format!(
                            "package part [Content_Types].xml is invalid: {error}"
                        ))
                    })?;
                    let value = attribute
                        .unescape_value()
                        .map_err(|error| {
                            DmsError::InvalidMarkdownTemplate(format!(
                                "package part [Content_Types].xml is invalid: {error}"
                            ))
                        })?
                        .into_owned();
                    match attribute.key.as_ref() {
                        b"Extension" => extension = Some(value),
                        b"ContentType" => content_type = Some(value),
                        _ => {}
                    }
                }
                if let (Some(extension), Some(content_type)) = (extension, content_type) {
                    let extension = extension.to_ascii_lowercase();
                    if let Some(previous) = declared.insert(extension.clone(), content_type.clone())
                    {
                        if !content_type.eq_ignore_ascii_case(&previous) {
                            return Err(DmsError::InvalidMarkdownTemplate(format!(
                                "package part [Content_Types].xml has conflicting declarations for extension {extension}"
                            )));
                        }
                    }
                }
            }
            Ok(XmlEvent::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(DmsError::InvalidMarkdownTemplate(format!(
                    "package part [Content_Types].xml is invalid: {error}"
                )))
            }
        }
    }
    let mut additions = String::new();
    for format in formats {
        match declared.get(format.extension()) {
            Some(content_type) if content_type.eq_ignore_ascii_case(format.content_type()) => {}
            Some(content_type) => {
                return Err(DmsError::InvalidMarkdownTemplate(format!(
                    "package part [Content_Types].xml declares extension {} as {content_type}",
                    format.extension()
                )))
            }
            None => {
                additions.push_str(&format!(
                    "<Default Extension=\"{}\" ContentType=\"{}\"/>",
                    format.extension(),
                    format.content_type()
                ));
                declared.insert(
                    format.extension().to_owned(),
                    format.content_type().to_owned(),
                );
            }
        }
    }
    if additions.is_empty() {
        return Ok(content_types.to_owned());
    }
    let end = content_types.rfind("</Types>").ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate(
            "package part [Content_Types].xml has no closing Types element".to_owned(),
        )
    })?;
    Ok(format!(
        "{}{}{}",
        &content_types[..end],
        additions,
        &content_types[end..]
    ))
}

fn ordered_list_numbering(
    template: &LoadedTemplate,
    blocks: &[Block],
) -> Result<Option<OrderedListNumbering>> {
    let mut starts = BTreeMap::new();
    for block in blocks {
        if let Block::ListItem {
            ordered: true,
            list_id,
            number,
            ..
        } = block
        {
            starts.entry(*list_id).or_insert(*number);
        }
    }
    if starts.is_empty() {
        return Ok(None);
    }

    let existing = template
        .entries
        .iter()
        .find(|entry| entry.name == "word/numbering.xml");
    let source = existing
        .map(|entry| {
            String::from_utf8(entry.bytes.clone()).map_err(|error| {
                DmsError::InvalidMarkdownTemplate(format!(
                    "word/numbering.xml is not UTF-8: {error}"
                ))
            })
        })
        .transpose()?;
    let (mut abstract_id, mut num_id) = source
        .as_deref()
        .map(|xml| {
            (
                max_numbering_attribute(xml, "abstractNumId"),
                max_numbering_attribute(xml, "numId"),
            )
        })
        .unwrap_or((0, 0));
    abstract_id = abstract_id.checked_add(1).ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate("no unused Word abstract numbering IDs remain".to_owned())
    })?;

    let namespace = WORDPROCESSINGML_MAIN_NAMESPACE;
    let mut num_ids = BTreeMap::new();
    let mut definitions = format!(
        concat!(
            "<w:abstractNum xmlns:w=\"{namespace}\" w:abstractNumId=\"{abstract_id}\">",
            "<w:multiLevelType w:val=\"singleLevel\"/><w:lvl w:ilvl=\"0\">",
            "<w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>",
            "<w:lvlText w:val=\"%1.\"/><w:lvlJc w:val=\"left\"/>",
            "<w:pPr><w:tabs><w:tab w:val=\"num\" w:pos=\"720\"/></w:tabs>",
            "<w:ind w:left=\"720\" w:hanging=\"360\"/></w:pPr>",
            "</w:lvl></w:abstractNum>"
        ),
        namespace = namespace,
        abstract_id = abstract_id
    );
    for (list_id, start) in starts {
        num_id = num_id.checked_add(1).ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate("no unused Word numbering IDs remain".to_owned())
        })?;
        num_ids.insert(list_id, num_id);
        definitions.push_str(&format!(
            concat!(
                "<w:num xmlns:w=\"{namespace}\" w:numId=\"{num_id}\">",
                "<w:abstractNumId w:val=\"{abstract_id}\"/>",
                "<w:lvlOverride w:ilvl=\"0\"><w:startOverride w:val=\"{start}\"/>",
                "</w:lvlOverride></w:num>"
            ),
            namespace = namespace,
            num_id = num_id,
            abstract_id = abstract_id,
            start = start
        ));
    }
    let xml = if let Some(source) = source {
        let end = source.rfind("</").ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate(
                "word/numbering.xml has no closing numbering element".to_owned(),
            )
        })?;
        format!("{}{}{}", &source[..end], definitions, &source[end..])
    } else {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:numbering xmlns:w=\"{namespace}\">{definitions}</w:numbering>"
        )
    };
    Ok(Some(OrderedListNumbering {
        xml,
        existed_in_template: existing.is_some(),
        num_ids,
    }))
}

fn max_numbering_attribute(xml: &str, attribute: &str) -> u32 {
    let needle = format!("{attribute}=\"");
    xml.match_indices(&needle)
        .filter_map(|(offset, _)| {
            xml[offset + needle.len()..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0)
}

fn relationships_with_numbering(relationships: &str) -> Result<String> {
    if relationships.contains("relationships/numbering") {
        return Ok(relationships.to_owned());
    }
    let id = max_relationship_id(relationships)
        .checked_add(1)
        .ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate("no unused relationship IDs remain".to_owned())
        })?;
    let end = relationships.rfind("</Relationships>").ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate(
            "package part word/_rels/document.xml.rels has no closing Relationships element"
                .to_owned(),
        )
    })?;
    Ok(format!(
        "{}<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" Target=\"numbering.xml\"/>{}",
        &relationships[..end],
        &relationships[end..]
    ))
}

fn content_types_with_numbering(content_types: &str) -> Result<String> {
    if content_types.contains("wordprocessingml.numbering+xml") {
        return Ok(content_types.to_owned());
    }
    let end = content_types.rfind("</Types>").ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate(
            "package part [Content_Types].xml has no closing Types element".to_owned(),
        )
    })?;
    Ok(format!(
        "{}<Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>{}",
        &content_types[..end],
        &content_types[end..]
    ))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Inline {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
    link: bool,
    image: Option<ImageInline>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ImageInline {
    reference: String,
    alt_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Block {
    Heading(u8, Vec<Inline>),
    Paragraph(Vec<Inline>),
    ListItem {
        ordered: bool,
        list_id: usize,
        number: u64,
        inlines: Vec<Inline>,
    },
    Code(Vec<Inline>),
    Table(Vec<Vec<Vec<Inline>>>),
}

#[derive(Clone, Copy, Debug)]
struct ListState {
    ordered: bool,
    next_number: u64,
    id: usize,
}

fn parse_markdown_blocks(markdown: &str) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut block_kind = None;
    let mut bold_depth = 0usize;
    let mut italic_depth = 0usize;
    let mut code_block = false;
    let mut link_stack = Vec::new();
    let mut list_stack = Vec::<ListState>::new();
    let mut next_list_id = 0usize;
    let mut table = None::<Vec<Vec<Vec<Inline>>>>;
    let mut row = None::<Vec<Vec<Inline>>>;
    let mut cell = None::<Vec<Inline>>;
    let mut events = Parser::new_ext(markdown, Options::all());
    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                start_block(
                    &mut blocks,
                    &mut block_kind,
                    &mut current,
                    BlockKind::Heading(heading_number(level)),
                );
            }
            Event::End(TagEnd::Heading(_)) => {
                finish_block(&mut blocks, &mut block_kind, &mut current)
            }
            Event::Start(Tag::Paragraph) if table.is_none() && block_kind.is_none() => {
                start_block(
                    &mut blocks,
                    &mut block_kind,
                    &mut current,
                    BlockKind::Paragraph,
                );
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
            Event::Start(Tag::Image { dest_url, .. }) => {
                let alt_text = collect_image_alt_text(&mut events)?;
                push_inline(
                    &mut current,
                    &mut cell,
                    Inline {
                        image: Some(ImageInline {
                            reference: dest_url.into_string(),
                            alt_text,
                        }),
                        ..Inline::default()
                    },
                );
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                start_block(&mut blocks, &mut block_kind, &mut current, BlockKind::Code);
                if let CodeBlockKind::Fenced(language) = kind {
                    if !language.is_empty() {
                        current.push(Inline {
                            text: format!("{language}\n"),
                            code: true,
                            ..Inline::default()
                        });
                    }
                }
                code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                code_block = false;
                finish_block(&mut blocks, &mut block_kind, &mut current);
            }
            Event::Start(Tag::List(start)) => {
                list_stack.push(ListState {
                    ordered: start.is_some(),
                    next_number: start.unwrap_or(1),
                    id: next_list_id,
                });
                next_list_id += 1;
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }
            Event::Start(Tag::Item) => {
                let list = list_stack.last().expect("list item has a containing list");
                start_block(
                    &mut blocks,
                    &mut block_kind,
                    &mut current,
                    BlockKind::ListItem {
                        ordered: list.ordered,
                        list_id: list.id,
                        number: list.next_number,
                    },
                );
            }
            Event::End(TagEnd::Item) => {
                finish_block(&mut blocks, &mut block_kind, &mut current);
                list_stack
                    .last_mut()
                    .expect("list item has a containing list")
                    .next_number += 1;
            }
            Event::Start(Tag::Table(_)) => {
                flush_paragraph(&mut blocks, &mut block_kind, &mut current);
                table = Some(Vec::new());
            }
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
            Event::Text(text) => {
                if block_kind.is_none() && table.is_none() {
                    block_kind = Some(BlockKind::Paragraph);
                }
                push_inline(
                    &mut current,
                    &mut cell,
                    Inline {
                        text: text.into_string(),
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        code: code_block,
                        link: !link_stack.is_empty(),
                        image: None,
                    },
                );
            }
            Event::Code(text) => {
                if block_kind.is_none() && table.is_none() {
                    block_kind = Some(BlockKind::Paragraph);
                }
                push_inline(
                    &mut current,
                    &mut cell,
                    Inline {
                        text: text.into_string(),
                        bold: bold_depth > 0,
                        italic: italic_depth > 0,
                        code: true,
                        link: !link_stack.is_empty(),
                        image: None,
                    },
                );
            }
            Event::SoftBreak | Event::HardBreak => push_inline(
                &mut current,
                &mut cell,
                Inline {
                    text: "\n".to_owned(),
                    ..Inline::default()
                },
            ),
            Event::Rule => {
                flush_paragraph(&mut blocks, &mut block_kind, &mut current);
                blocks.push(Block::Paragraph(vec![Inline {
                    text: "—".to_owned(),
                    ..Inline::default()
                }]));
            }
            // Raw HTML is preserved verbatim as text so no construct is
            // dropped silently, but a raw HTML picture is an unsupported
            // placement and never degrades into markup or missing pixels.
            Event::Html(html) | Event::InlineHtml(html) => {
                let html = html.into_string();
                if html.to_ascii_lowercase().contains("<img") {
                    return Err(DmsError::InvalidMarkdownImage(format!(
                        "raw HTML image markup `{html}` is not a supported placement"
                    )));
                }
                if block_kind.is_none() && table.is_none() {
                    block_kind = Some(BlockKind::Paragraph);
                }
                push_inline(
                    &mut current,
                    &mut cell,
                    Inline {
                        text: html,
                        ..Inline::default()
                    },
                );
            }
            Event::InlineMath(math) | Event::DisplayMath(math) => {
                if block_kind.is_none() && table.is_none() {
                    block_kind = Some(BlockKind::Paragraph);
                }
                push_inline(
                    &mut current,
                    &mut cell,
                    Inline {
                        text: format!("${math}$"),
                        ..Inline::default()
                    },
                );
            }
            Event::FootnoteReference(label) => {
                if block_kind.is_none() && table.is_none() {
                    block_kind = Some(BlockKind::Paragraph);
                }
                push_inline(
                    &mut current,
                    &mut cell,
                    Inline {
                        text: format!("[^{label}]"),
                        ..Inline::default()
                    },
                );
            }
            Event::TaskListMarker(checked) => {
                if block_kind.is_none() && table.is_none() {
                    block_kind = Some(BlockKind::Paragraph);
                }
                let marker = if checked { "[x] " } else { "[ ] " };
                push_inline(
                    &mut current,
                    &mut cell,
                    Inline {
                        text: marker.to_owned(),
                        ..Inline::default()
                    },
                );
            }
            _ => {}
        }
    }
    if block_kind.is_some() {
        finish_block(&mut blocks, &mut block_kind, &mut current);
    }
    flush_paragraph(&mut blocks, &mut block_kind, &mut current);
    Ok(blocks)
}

/// Consume the image's complete inline subtree without changing body formatting
/// or emitting body runs. Nested images contribute their alternative text only.
fn collect_image_alt_text<'a>(events: &mut impl Iterator<Item = Event<'a>>) -> Result<String> {
    let mut alt_text = String::new();
    let mut depth = 1;
    for event in events {
        match event {
            Event::Start(Tag::Image { .. }) => depth += 1,
            Event::End(TagEnd::Image) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Event::Text(text) | Event::Code(text) => alt_text.push_str(&text),
            Event::SoftBreak | Event::HardBreak => alt_text.push(' '),
            Event::Html(html) | Event::InlineHtml(html) => {
                if html.to_ascii_lowercase().contains("<img") {
                    return Err(DmsError::InvalidMarkdownImage(format!(
                        "raw HTML image markup `{html}` is not a supported placement"
                    )));
                }
                alt_text.push_str(&html);
            }
            Event::InlineMath(math) | Event::DisplayMath(math) => {
                alt_text.push_str(&format!("${math}$"));
            }
            Event::FootnoteReference(label) => alt_text.push_str(&format!("[^{label}]")),
            Event::TaskListMarker(checked) => {
                alt_text.push_str(if checked { "[x] " } else { "[ ] " })
            }
            Event::Rule => alt_text.push_str(" — "),
            Event::Start(_) | Event::End(_) => {}
        }
    }
    Ok(alt_text)
}

#[derive(Clone, Copy, Debug)]
enum BlockKind {
    Heading(u8),
    Paragraph,
    ListItem {
        ordered: bool,
        list_id: usize,
        number: u64,
    },
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
        BlockKind::ListItem {
            ordered,
            list_id,
            number,
        } => Block::ListItem {
            ordered,
            list_id,
            number,
            inlines,
        },
        BlockKind::Code => Block::Code(inlines),
    });
}

/// Close text that arrived without an explicit block container (for example a
/// definition-list title, metadata block, or raw HTML line) as its own
/// paragraph so no visible content is dropped.
fn flush_paragraph(
    blocks: &mut Vec<Block>,
    kind: &mut Option<BlockKind>,
    inlines: &mut Vec<Inline>,
) {
    if kind.is_none() && !inlines.is_empty() {
        *kind = Some(BlockKind::Paragraph);
        finish_block(blocks, kind, inlines);
    }
}

fn start_block(
    blocks: &mut Vec<Block>,
    kind: &mut Option<BlockKind>,
    inlines: &mut Vec<Inline>,
    next: BlockKind,
) {
    // A nested container (for example a nested list inside a list item) closes
    // the enclosing block instead of discarding the text collected so far.
    if !inlines.is_empty() {
        if kind.is_none() {
            *kind = Some(BlockKind::Paragraph);
        }
        finish_block(blocks, kind, inlines);
    }
    inlines.clear();
    *kind = Some(next);
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

fn render_blocks(
    blocks: &[Block],
    context: &mut RenderContext<'_>,
    ordered_list_numbering: Option<&OrderedListNumbering>,
) -> Result<String> {
    let mut output = String::new();
    for (index, block) in blocks.iter().enumerate() {
        let template = context.template;
        match block {
            Block::Heading(level, inlines) => {
                let prototype = template
                    .paragraphs
                    .get(&format!("heading{level}"))
                    .expect("validated heading prototype")
                    .clone();
                output.push_str(&render_paragraph(&prototype, inlines, None, context)?)
            }
            Block::Paragraph(inlines) => {
                let prototype = template
                    .paragraphs
                    .get("paragraph")
                    .expect("validated paragraph")
                    .clone();
                output.push_str(&render_paragraph(&prototype, inlines, None, context)?)
            }
            Block::ListItem {
                ordered,
                list_id,
                inlines,
                ..
            } => {
                let prototype = template
                    .paragraphs
                    .get(if *ordered { "paragraph" } else { "list" })
                    .expect("validated list/paragraph prototype")
                    .clone();
                let prototype = if *ordered {
                    let num_id = ordered_list_numbering
                        .and_then(|numbering| numbering.num_ids.get(list_id))
                        .expect("ordered list has generated Word numbering");
                    paragraph_with_numbering(&prototype, *num_id)?
                } else {
                    prototype
                };
                output.push_str(&render_paragraph(&prototype, inlines, None, context)?)
            }
            Block::Code(inlines) => {
                let mut inlines = inlines.clone();
                for inline in &mut inlines {
                    inline.code = true;
                }
                let prototype = template
                    .paragraphs
                    .get("paragraph")
                    .expect("validated paragraph")
                    .clone();
                output.push_str(&render_paragraph(&prototype, &inlines, None, context)?);
            }
            Block::Table(rows) => {
                output.push_str(&render_table(rows, context)?);
                if matches!(blocks.get(index + 1), Some(Block::Table(_))) {
                    let prototype = template
                        .paragraphs
                        .get("paragraph")
                        .expect("validated paragraph")
                        .clone();
                    output.push_str(&render_paragraph(&prototype, &[], None, context)?);
                }
            }
        }
    }
    Ok(output)
}

fn paragraph_with_numbering(prototype: &str, num_id: u32) -> Result<String> {
    let opening_end = prototype.find('>').ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate("paragraph prototype has no opening tag".to_owned())
    })? + 1;
    let closing = prototype.rfind("</w:p>").ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate("paragraph prototype has no closing tag".to_owned())
    })?;
    let numbering =
        format!("<w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"{num_id}\"/></w:numPr>");
    if let Some(properties_start) = prototype[opening_end..closing].find("<w:pPr") {
        let properties_start = opening_end + properties_start;
        let properties_end = prototype[properties_start..closing]
            .find('>')
            .ok_or_else(|| {
                DmsError::InvalidMarkdownTemplate(
                    "paragraph prototype has an incomplete property element".to_owned(),
                )
            })?
            + properties_start
            + 1;
        return Ok(format!(
            "{}{}{}",
            &prototype[..properties_end],
            numbering,
            &prototype[properties_end..]
        ));
    }
    Ok(format!(
        "{}<w:pPr>{}</w:pPr>{}",
        &prototype[..opening_end],
        numbering,
        &prototype[opening_end..]
    ))
}

fn render_paragraph(
    prototype: &str,
    inlines: &[Inline],
    prefix: Option<&str>,
    context: &mut RenderContext<'_>,
) -> Result<String> {
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
        output.push_str(&render_run(
            &Inline {
                text: prefix.to_owned(),
                ..Inline::default()
            },
            context,
        )?);
    }
    for inline in inlines {
        output.push_str(&render_run(inline, context)?);
    }
    output.push_str("</w:p>");
    Ok(output)
}

fn render_run(inline: &Inline, context: &mut RenderContext<'_>) -> Result<String> {
    if let Some(image) = &inline.image {
        return context.image_run(image);
    }
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
        Ok(format!(
            "<w:r><w:t xml:space=\"preserve\">{text}</w:t></w:r>"
        ))
    } else {
        Ok(format!(
            "<w:r><w:rPr>{properties}</w:rPr><w:t xml:space=\"preserve\">{text}</w:t></w:r>"
        ))
    }
}

fn render_table(rows: &[Vec<Vec<Inline>>], context: &mut RenderContext<'_>) -> Result<String> {
    let prototype = context.template.table.clone();
    let row_span = element_span(&prototype, "<w:tr", "</w:tr>")?;
    let row_prototype = &prototype[row_span.0..row_span.1];
    let cells = all_element_spans(row_prototype, "<w:tc", "</w:tc>")?;
    let columns = rows.first().map_or(0, Vec::len);
    if columns == 0 || rows.iter().any(|row| row.len() != columns) {
        return Err(DmsError::InvalidMarkdownTemplate(
            "Markdown table rows must have a consistent nonzero column count".to_owned(),
        ));
    }
    // Every generated table spans the template's printable body width with a
    // fixed layout, including two-column tables.
    let width = table_printable_width(context.template)?;
    if columns > width as usize {
        return Err(DmsError::InvalidMarkdownTemplate(
            "Markdown table has too many columns for the template's printable width".to_owned(),
        ));
    }
    let widths = (0..columns)
        .map(|column| width / columns as u32 + u32::from((column as u32) < width % columns as u32))
        .collect::<Vec<_>>();
    let mut rendered_rows = String::new();
    for row in rows {
        rendered_rows.push_str(&row_prototype[..cells[0].0]);
        for (column, inlines) in row.iter().enumerate() {
            if column == 1 {
                rendered_rows.push_str(&row_prototype[cells[0].1..cells[1].0]);
            }
            let prototype_column = usize::from(column != 0);
            let span = cells[prototype_column];
            let token = if prototype_column == 0 {
                TABLE_COLUMN_1
            } else {
                TABLE_COLUMN_2
            };
            let mut cell =
                render_table_cell(&row_prototype[span.0..span.1], token, inlines, context)?;
            let properties = xml_child(&cell, "w:tcPr")?.unwrap_or("<w:tcPr/>");
            let properties = set_xml_child(
                properties,
                "w:tcW",
                &format!("<w:tcW w:w=\"{}\" w:type=\"dxa\"/>", widths[column]),
                &["w:cnfStyle"],
            )?;
            cell = set_xml_child(&cell, "w:tcPr", &properties, &[])?;
            rendered_rows.push_str(&cell);
        }
        rendered_rows.push_str(&row_prototype[cells[1].1..]);
    }
    let mut output = format!(
        "{}{}{}",
        &prototype[..row_span.0],
        rendered_rows,
        &prototype[row_span.1..]
    );
    {
        let properties = xml_child(&output, "w:tblPr")?.unwrap_or("<w:tblPr/>");
        let properties = set_xml_child(
            properties,
            "w:tblW",
            &format!("<w:tblW w:w=\"{width}\" w:type=\"dxa\"/>"),
            &[
                "w:tblStyle",
                "w:tblpPr",
                "w:tblOverlap",
                "w:bidiVisual",
                "w:tblStyleRowBandSize",
                "w:tblStyleColBandSize",
            ],
        )?;
        let properties = set_xml_child(
            &properties,
            "w:tblLayout",
            "<w:tblLayout w:type=\"fixed\"/>",
            &[
                "w:tblStyle",
                "w:tblpPr",
                "w:tblOverlap",
                "w:bidiVisual",
                "w:tblStyleRowBandSize",
                "w:tblStyleColBandSize",
                "w:tblW",
                "w:jc",
                "w:tblCellSpacing",
                "w:tblInd",
                "w:tblBorders",
                "w:shd",
            ],
        )?;
        output = set_xml_child(&output, "w:tblPr", &properties, &[])?;
        let grid = widths
            .iter()
            .map(|width| format!("<w:gridCol w:w=\"{width}\"/>"))
            .collect::<String>();
        output = set_xml_child(
            &output,
            "w:tblGrid",
            &format!("<w:tblGrid>{grid}</w:tblGrid>"),
            &["w:tblPr"],
        )?;
    }
    Ok(output)
}

fn render_table_cell(
    prototype: &str,
    token: &str,
    inlines: &[Inline],
    context: &mut RenderContext<'_>,
) -> Result<String> {
    if inlines.iter().all(|inline| {
        !inline.bold && !inline.italic && !inline.code && !inline.link && inline.image.is_none()
    }) {
        return Ok(prototype.replace(token, &plain_inline_text(inlines)));
    }
    let (start, end) = enclosing_element(prototype, token, "<w:r", "</w:r>")?;
    let run = &prototype[start..end];
    let properties = xml_child(run, "w:rPr")?.unwrap_or("<w:rPr/>");
    let mut runs = String::new();
    for inline in inlines {
        let mut rendered = render_run(inline, context)?;
        // A drawing run carries no template text styling.
        if inline.image.is_none() {
            let mut merged = properties.to_owned();
            if let Some(overrides) = xml_child(&rendered, "w:rPr")? {
                for (name, span) in xml_children(overrides)? {
                    merged = set_xml_child(&merged, &name, &overrides[span.0..span.1], &[])?;
                }
            }
            rendered = set_xml_child(&rendered, "w:rPr", &merged, &[])?;
        }
        runs.push_str(&rendered);
    }
    // Split the marker's text/run, keeping any surrounding template content once.
    let position = run.find(token).expect("validated cell marker");
    let text_start = find_last_opening_element(&run[..position], "<w:t").ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate("table marker must be inside a text run".to_owned())
    })?;
    let text_opening = &run[text_start..run[text_start..].find('>').unwrap() + text_start + 1];
    let run_opening = &run[..run.find('>').unwrap() + 1];
    Ok(format!(
        "{}{}{}{}{}{}{}{}{}",
        &prototype[..start],
        &run[..position],
        "</w:t></w:r>",
        runs,
        run_opening,
        properties,
        text_opening,
        &run[position + token.len()..],
        &prototype[end..]
    ))
}

fn table_printable_width(template: &LoadedTemplate) -> Result<u32> {
    // Section properties follow the content they govern, not the preceding cover.
    let suffix = &template.document_xml[template.prototype_end..];
    let span = element_span(suffix, "<w:sectPr", "</w:sectPr>")?;
    let section = &suffix[span.0..span.1];
    let size = xml_child(section, "w:pgSz")?.unwrap_or_default();
    let margins = xml_child(section, "w:pgMar")?.unwrap_or_default();
    let width = xml_dimension(size, "w:w", None)?;
    let left = xml_dimension(margins, "w:left", None)?;
    let right = xml_dimension(margins, "w:right", None)?;
    let gutter = xml_dimension(margins, "w:gutter", Some(0))?;
    width
        .checked_sub(left)
        .and_then(|width| width.checked_sub(right))
        .and_then(|width| width.checked_sub(gutter))
        .filter(|width| *width > 0)
        .ok_or_else(|| {
            DmsError::InvalidMarkdownTemplate(
                "template has no positive printable table width".to_owned(),
            )
        })
}

fn xml_dimension(element: &str, name: &str, default: Option<u32>) -> Result<u32> {
    let mut reader = Reader::from_str(element);
    if let Ok(XmlEvent::Start(element) | XmlEvent::Empty(element)) = reader.read_event() {
        for attribute in element.attributes() {
            let attribute =
                attribute.map_err(|error| DmsError::InvalidMarkdownTemplate(error.to_string()))?;
            if attribute.key.as_ref() == name.as_bytes() {
                return attribute
                    .unescape_value()
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .ok_or_else(|| {
                        DmsError::InvalidMarkdownTemplate(format!(
                            "template dimension {name} must be an unsigned twip value"
                        ))
                    });
            }
        }
    }
    default.ok_or_else(|| {
        DmsError::InvalidMarkdownTemplate(format!(
            "template is missing page dimension {name} for table layout"
        ))
    })
}

fn xml_children(xml: &str) -> Result<Vec<(String, (usize, usize))>> {
    let mut reader = Reader::from_str(xml);
    let mut depth = 0usize;
    let mut child = None;
    let mut children = Vec::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader
            .read_event()
            .map_err(|error| DmsError::InvalidMarkdownTemplate(error.to_string()))?
        {
            XmlEvent::Start(element) => {
                if depth == 1 {
                    child = Some((
                        String::from_utf8_lossy(element.name().as_ref()).into_owned(),
                        start,
                    ));
                }
                depth += 1;
            }
            XmlEvent::End(_) => {
                depth -= 1;
                if depth == 1 {
                    let (name, start) = child.take().expect("direct child start");
                    children.push((name, (start, reader.buffer_position() as usize)));
                }
            }
            XmlEvent::Empty(element) if depth == 1 => children.push((
                String::from_utf8_lossy(element.name().as_ref()).into_owned(),
                (start, reader.buffer_position() as usize),
            )),
            XmlEvent::Eof => break,
            _ => {}
        }
    }
    Ok(children)
}

fn xml_child<'a>(xml: &'a str, name: &str) -> Result<Option<&'a str>> {
    Ok(xml_children(xml)?
        .into_iter()
        .find(|(child, _)| child == name)
        .map(|(_, (start, end))| &xml[start..end]))
}

fn set_xml_child(
    xml: &str,
    name: &str,
    replacement: &str,
    predecessors: &[&str],
) -> Result<String> {
    let children = xml_children(xml)?;
    if let Some((_, (start, end))) = children.iter().find(|(child, _)| child == name) {
        return Ok(format!("{}{}{}", &xml[..*start], replacement, &xml[*end..]));
    }
    let opening_end = xml.find('>').expect("validated XML element") + 1;
    if xml[..opening_end].ends_with("/>") {
        let tag = xml[1..]
            .split(|character: char| character.is_whitespace() || character == '/')
            .next()
            .unwrap();
        return Ok(format!("{}>{replacement}</{tag}>", &xml[..opening_end - 2]));
    }
    let position = children
        .iter()
        .filter(|(child, _)| predecessors.contains(&child.as_str()))
        .map(|(_, (_, end))| *end)
        .max()
        .unwrap_or(opening_end);
    Ok(format!(
        "{}{}{}",
        &xml[..position],
        replacement,
        &xml[position..]
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

/// Remove every template Table of Contents structured-document tag: an `sdt`
/// in the WordprocessingML namespace whose `docPartGallery` value is
/// `Table of Contents`. Namespace-aware matching leaves other structured
/// documents, source-authored static TOC content and every other package part
/// untouched. A document without a matching tag is returned unchanged.
fn remove_table_of_contents_sdt(xml: &str) -> Result<String> {
    let mut reader = NsReader::from_reader(xml.as_bytes());
    let mut open: Vec<(usize, bool, Vec<u8>)> = Vec::new();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader
            .read_resolved_event()
            .map_err(|error| DmsError::InvalidMarkdownTemplate(error.to_string()))?
        {
            (ResolveResult::Bound(Namespace(namespace)), XmlEvent::Start(element))
                if namespace == WORDPROCESSINGML_MAIN_NAMESPACE.as_bytes()
                    && element.local_name().as_ref() == b"sdt" =>
            {
                open.push((start, false, element.name().as_ref().to_vec()));
            }
            (
                ResolveResult::Bound(Namespace(namespace)),
                XmlEvent::Start(element) | XmlEvent::Empty(element),
            ) if namespace == WORDPROCESSINGML_MAIN_NAMESPACE.as_bytes()
                && element.local_name().as_ref() == b"docPartGallery" =>
            {
                let gallery = element.attributes().flatten().find_map(|attribute| {
                    let (namespace, local) = reader.resolve_attribute(attribute.key);
                    (local.as_ref() == b"val"
                        && namespace
                            == ResolveResult::Bound(Namespace(
                                WORDPROCESSINGML_MAIN_NAMESPACE.as_bytes(),
                            )))
                    .then(|| {
                        attribute
                            .unescape_value()
                            .ok()
                            .map(|value| value.into_owned())
                    })
                    .flatten()
                });
                if gallery.as_deref() == Some(TABLE_OF_CONTENTS_GALLERY) {
                    if let Some(tag) = open.last_mut() {
                        tag.1 = true;
                    }
                }
            }
            (_, XmlEvent::End(element)) => {
                let closes_open = open
                    .last()
                    .is_some_and(|(_, _, name)| element.name().as_ref() == name.as_slice());
                if closes_open {
                    let (start, matched, _) = open.pop().expect("open sdt tag");
                    if matched {
                        spans.push((start, reader.buffer_position() as usize));
                    }
                }
            }
            (_, XmlEvent::Eof) => break,
            _ => {}
        }
    }
    if spans.is_empty() {
        return Ok(xml.to_owned());
    }
    spans.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in spans {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    let mut output = xml.to_owned();
    for (start, end) in merged.into_iter().rev() {
        output.replace_range(start..end, "");
    }
    Ok(output)
}

/// Normalize a relative path to portable `/` separators, dropping any `.` or
/// root/prefix components.
fn portable_relative_path(path: &Path) -> PathBuf {
    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::ParentDir => relative.push(".."),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    relative
}
