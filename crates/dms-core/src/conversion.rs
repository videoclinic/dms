//! Standalone conversion: no Workspace open, migration, persistence or workflow.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    frontmatter::parse_conversion_frontmatter, template::assemble_markdown_conversion_docx,
    validate_markdown_template, ConfidentialitySnapshot, ConfidentialityType, ExportChrome,
    MarkdownAssetContext, MarkdownAssetDigest, MarkdownTemplateAsset,
    MARKDOWN_TEMPLATE_CONTRACT_VERSION, SCHEMA_VERSION,
};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ConversionError(pub String);

type Result<T> = std::result::Result<T, ConversionError>;

/// The adapter fills controlled fields, updates and saves this DOCX, then exports
/// that same document to PDF. It must not edit DOCX after PDF export.
pub trait ConversionExporter {
    fn preflight(&self) -> std::result::Result<(), String>;
    fn export(
        &mut self,
        docx: &Path,
        pdf: &Path,
        chrome: &ExportChrome,
    ) -> std::result::Result<String, String>;
}

#[derive(Clone, Debug)]
pub struct ConversionOptions {
    pub workspace_config: PathBuf,
    pub input_root: PathBuf,
    pub output_root: PathBuf,
    pub recursive: bool,
    pub dry_run: bool,
    pub without_toc: bool,
}

/// Deliberately ignores unrelated workspace fields. This is not a Workspace and
/// has no save/open/migrate method or identity/notification dependencies.
#[derive(Debug, Deserialize)]
struct ConfigurationProjection {
    schema_version: u32,
    edit_root: PathBuf,
    publish_root: PathBuf,
    markdown_template: MarkdownTemplateAsset,
    confidentiality_types: BTreeMap<String, ConfidentialityType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversionMetadata {
    pub title: String,
    pub document_number: Option<String>,
    pub version: String,
    pub confidentiality: ConfidentialitySnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversionArtifact {
    #[serde(with = "crate::relative_path_serde")]
    pub relative_path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionStatus {
    Validated,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversionFileResult {
    #[serde(with = "crate::relative_path_serde")]
    pub input: PathBuf,
    pub input_sha256: Option<String>,
    pub assets: Vec<MarkdownAssetDigest>,
    pub metadata: Option<ConversionMetadata>,
    pub docx: Option<ConversionArtifact>,
    pub pdf: Option<ConversionArtifact>,
    pub word_version: Option<String>,
    pub status: ConversionStatus,
    pub error: Option<String>,
    /// Only paths owned by this run whose cleanup failed.
    pub retained_diagnostics: Vec<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversionManifest {
    pub conversion_only: bool,
    pub dry_run: bool,
    pub without_toc: bool,
    pub renderer_version: String,
    pub configuration_schema: u32,
    pub template: MarkdownTemplateAsset,
    pub results: Vec<ConversionFileResult>,
}

impl ConversionManifest {
    pub fn has_failures(&self) -> bool {
        self.results
            .iter()
            .any(|r| matches!(r.status, ConversionStatus::Failed))
    }
}

pub struct ConversionPlan {
    options: ConversionOptions,
    config: ConfigurationProjection,
    template_bytes: Vec<u8>,
    selected: Vec<PathBuf>,
}

impl ConversionPlan {
    pub fn prepare(options: &ConversionOptions) -> Result<Self> {
        if fs::symlink_metadata(&options.workspace_config)
            .map_err(io_error)?
            .file_type()
            .is_symlink()
        {
            return Err(error("workspace configuration file must not be a symlink"));
        }
        let config_path = canonical_file(&options.workspace_config)?;
        let bytes = read(&config_path)?;
        let config: ConfigurationProjection = serde_json::from_slice(&bytes)
            .map_err(|_| error("invalid conversion configuration projection"))?;
        if ![18, 19, SCHEMA_VERSION].contains(&config.schema_version) {
            return Err(error(
                "unsupported conversion configuration schema (expected 18, 19 or current)",
            ));
        }
        if !config.edit_root.is_absolute() || !config.publish_root.is_absolute() {
            return Err(error("configured roots must be absolute native-host paths"));
        }
        let edit = canonical_dir(&config.edit_root)?;
        let publish = canonical_dir(&config.publish_root)?;
        if config_path != edit.join(".dms/workspace.json") {
            return Err(error(
                "workspace configuration location does not match configured edit root",
            ));
        }
        reject_symlink_components(&edit, &config_path)?;
        if overlaps(&edit, &publish) {
            return Err(error("configured edit and publish roots must be disjoint"));
        }
        for (id, item) in &config.confidentiality_types {
            if id != &item.id
                || crate::validate_portable_id(id, crate::DmsError::InvalidDocumentTypeId).is_err()
                || item.label.trim().is_empty()
            {
                return Err(error("invalid confidentiality catalogue"));
            }
        }
        let input = canonical_dir(&options.input_root)?;
        if overlaps(&input, &edit) || overlaps(&input, &publish) {
            return Err(error(
                "conversion input must be external to workspace edit and publish trees",
            ));
        }
        let output = new_output_path(&options.output_root)?;
        for root in [&input, &edit, &publish] {
            if overlaps(&output, root) {
                return Err(error(
                    "output root must be disjoint from input, edit and publish trees",
                ));
            }
        }
        let template = &config.markdown_template;
        if template.contract_version != MARKDOWN_TEMPLATE_CONTRACT_VERSION {
            return Err(error("unsupported Markdown template contract"));
        }
        if template.relative_path.as_os_str().is_empty()
            || template
                .relative_path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(error("template locator must be edit-root-relative"));
        }
        let template_path = edit.join(&template.relative_path);
        if crate::validate_relative_source_path(&template.relative_path).is_err()
            || crate::is_metadata_path(&template.relative_path)
            || template
                .relative_path
                .extension()
                .is_none_or(|ext| !ext.eq_ignore_ascii_case("docx"))
            || template
                .relative_path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("~$"))
        {
            return Err(error(
                "template must be an ordinary edit-root-relative .docx outside .dms",
            ));
        }
        reject_symlink_components(&edit, &template_path)?;
        let template_bytes = read(&canonical_file(&template_path)?)?;
        if digest(&template_bytes) != template.sha256 {
            return Err(error("configured template SHA-256 mismatch"));
        }
        // Validation and later assembly use exactly these captured bytes, not a
        // second read of an operator-editable template.
        let snapshot = tempfile::tempdir().map_err(io_error)?;
        let snapshot_path = snapshot.path().join("template.docx");
        fs::write(&snapshot_path, &template_bytes).map_err(io_error)?;
        validate_markdown_template(&snapshot_path)
            .map_err(|_| error("configured Markdown template contract is invalid"))?;
        let mut selected = Vec::new();
        discover(&input, &input, options.recursive, &mut selected)?;
        selected.sort();
        if selected.is_empty() {
            return Err(error("input selection contains no Markdown files"));
        }
        validate_destinations(&selected)?;
        let mut options = options.clone();
        options.input_root = input;
        options.output_root = output;
        Ok(Self {
            options,
            config,
            template_bytes,
            selected,
        })
    }

    pub fn selected_paths(&self) -> Vec<PathBuf> {
        self.selected.clone()
    }

    pub fn run(&self, exporter: &mut impl ConversionExporter) -> Result<ConversionManifest> {
        // Recheck no-overwrite after any caller-side selection presentation.
        if fs::symlink_metadata(&self.options.output_root).is_ok() {
            return Err(error(
                "output root already exists; overwrite and resume are not supported",
            ));
        }
        let output_parent = self
            .options
            .output_root
            .parent()
            .expect("canonical output has parent");
        if canonical_dir(output_parent)? != output_parent {
            return Err(error("output parent changed after preflight"));
        }
        if !self.options.dry_run {
            exporter.preflight().map_err(ConversionError)?;
        }
        let scratch = tempfile::tempdir().map_err(io_error)?;
        let template = scratch.path().join("template.docx");
        fs::write(&template, &self.template_bytes).map_err(io_error)?;
        let output = if self.options.dry_run {
            scratch.path().join("validated")
        } else {
            self.options.output_root.clone()
        };
        // A new directory is the ownership boundary; never reuse existing output.
        fs::create_dir(&output).map_err(io_error)?;
        let output = canonical_dir(&output)?;
        if !self.options.dry_run && output != self.options.output_root {
            return Err(error("output parent changed during preflight"));
        }
        let mut manifest = ConversionManifest {
            conversion_only: true,
            dry_run: self.options.dry_run,
            without_toc: self.options.without_toc,
            renderer_version: format!("dms-core/{}", env!("CARGO_PKG_VERSION")),
            configuration_schema: self.config.schema_version,
            template: self.config.markdown_template.clone(),
            results: Vec::new(),
        };
        for relative in &self.selected {
            let mut result = ConversionFileResult {
                input: relative.clone(),
                input_sha256: None,
                assets: Vec::new(),
                metadata: None,
                docx: None,
                pdf: None,
                word_version: None,
                status: ConversionStatus::Failed,
                error: None,
                retained_diagnostics: Vec::new(),
            };
            let mut owned = Vec::new();
            match self.convert_one(
                relative,
                &template,
                &output,
                exporter,
                &mut result,
                &mut owned,
            ) {
                Ok(()) => {
                    result.status = if self.options.dry_run {
                        ConversionStatus::Validated
                    } else {
                        ConversionStatus::Succeeded
                    }
                }
                Err(e) => {
                    result.error = Some(e.to_string());
                    result.docx = None;
                    result.pdf = None;
                    result.word_version = None;
                    for path in owned {
                        let safe_parent = canonical_dir(&output)
                            .is_ok_and(|canonical| canonical == output)
                            && reject_symlink_components(
                                &output,
                                path.parent().expect("artifact has parent"),
                            )
                            .is_ok();
                        if !safe_parent
                            || (fs::symlink_metadata(&path).is_ok()
                                && fs::remove_file(&path).is_err())
                        {
                            result
                                .retained_diagnostics
                                .push(path.strip_prefix(&output).unwrap_or(&path).to_path_buf());
                        }
                    }
                }
            }
            manifest.results.push(result);
        }
        if !self.options.dry_run {
            ensure_output_parent(&output, &output)?;
            let bytes = serde_json::to_vec_pretty(&manifest)
                .map_err(|_| error("manifest serialization failed"))?;
            write_new(&output.join("manifest.json"), &bytes)?;
            let recorded = read(&output.join("manifest.json"))?;
            if recorded != bytes {
                return Err(error("manifest read-back mismatch"));
            }
        }
        Ok(manifest)
    }

    fn convert_one(
        &self,
        relative: &Path,
        template: &Path,
        output: &Path,
        exporter: &mut impl ConversionExporter,
        result: &mut ConversionFileResult,
        owned: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let source = self.options.input_root.join(relative);
        if canonical_dir(&self.options.input_root)? != self.options.input_root {
            return Err(error("input root changed after preflight"));
        }
        reject_symlink_components(&self.options.input_root, &source)?;
        let source = canonical_file(&source)?;
        if !source.starts_with(&self.options.input_root) {
            return Err(error("source escaped the explicit input root"));
        }
        let bytes = read(&source)?;
        result.input_sha256 = Some(digest(&bytes));
        let markdown =
            std::str::from_utf8(&bytes).map_err(|_| error("Markdown source is not UTF-8"))?;
        let metadata = self.metadata(markdown)?;
        let chrome = ExportChrome {
            title: metadata.title.clone(),
            document_number: metadata.document_number.clone(),
            version_label: metadata.version.clone(),
            confidentiality: metadata.confidentiality.clone(),
        };
        result.metadata = Some(metadata);
        let docx_relative = relative.with_extension("docx");
        let pdf_relative = relative.with_extension("pdf");
        let docx = output.join(&docx_relative);
        let pdf = output.join(&pdf_relative);
        ensure_output_parent(output, docx.parent().expect("artifact has parent"))?;
        // Assemble privately, then claim destinations with create_new. The
        // exporter receives the exact retained DOCX path, not the intermediate.
        let assembly = tempfile::tempdir().map_err(io_error)?;
        let assembled = assembly.path().join("assembled.docx");
        result.assets = assemble_markdown_conversion_docx(
            template, markdown, &assembled,
            Some(&MarkdownAssetContext::new(&self.options.input_root, &source)),
            self.options.without_toc,
        ).map_err(|e| match e {
            crate::DmsError::InvalidConversionDate => ConversionError(e.to_string()),
            crate::DmsError::InvalidMarkdownImage(_) => error("Markdown image rejected: check local PNG/JPEG format, existence and input-root containment"),
            _ => error("Markdown assembly failed (unsupported content or template variable)"),
        })?;
        if self.options.dry_run {
            return Ok(());
        }
        write_owned(&docx, &read(&assembled)?, owned)?;
        // Reserve PDF ownership before invoking the adapter; an adapter may
        // replace this owned empty placeholder, never a pre-existing user file.
        write_owned(&pdf, &[], owned)?;
        result.word_version = Some(
            exporter
                .export(&docx, &pdf, &chrome)
                .map_err(ConversionError)?,
        );
        let docx_bytes = read_regular_output(output, &docx)?;
        let pdf_bytes = read_regular_output(output, &pdf)?;
        validate_conversion_pair(&docx_bytes, &pdf_bytes)?;
        result.docx = Some(ConversionArtifact {
            relative_path: docx_relative,
            sha256: digest(&docx_bytes),
        });
        result.pdf = Some(ConversionArtifact {
            relative_path: pdf_relative,
            sha256: digest(&pdf_bytes),
        });
        Ok(())
    }

    fn metadata(&self, markdown: &str) -> Result<ConversionMetadata> {
        // The parser's detailed error can contain a source line. Do not expose
        // that line in the manifest or CLI diagnostics.
        let (fields, _) = parse_conversion_frontmatter(markdown)
            .map_err(|_| error("invalid or missing flat conversion frontmatter"))?;
        let title = fields
            .title
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| error("conversion requires a nonempty title"))?;
        let version = &fields.version;
        let parts: Vec<_> = version.split('.').collect();
        if parts.len() != 2
            || parts.iter().any(|p| {
                p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) || p.parse::<u32>().is_err()
            })
        {
            return Err(error("conversion version must be numeric MAJOR.MINOR"));
        }
        let classification = self
            .config
            .confidentiality_types
            .get(&fields.confidentiality)
            .filter(|c| c.enabled)
            .ok_or_else(|| {
                error("conversion confidentiality must name an enabled configured type ID")
            })?;
        Ok(ConversionMetadata {
            title,
            document_number: fields.document_number.filter(|v| !v.trim().is_empty()),
            version: version.clone(),
            confidentiality: ConfidentialitySnapshot {
                type_id: classification.id.clone(),
                label: classification.label.clone(),
            },
        })
    }
}

/// Parse both complete artifacts, not just their file signatures. This proves
/// package/PDF readability, not pagination or visual fidelity in installed Word.
pub fn validate_conversion_pair(docx: &[u8], pdf: &[u8]) -> Result<()> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx))
        .map_err(|_| error("final DOCX is not a readable ZIP package"))?;
    let mut parts = BTreeSet::new();
    for index in 0..archive.len() {
        let mut part = archive
            .by_index(index)
            .map_err(|_| error("final DOCX has an unreadable part"))?;
        let name = part.name().to_owned();
        if !parts.insert(name.clone()) {
            return Err(error("final DOCX has duplicate package parts"));
        }
        let mut bytes = Vec::new();
        part.read_to_end(&mut bytes)
            .map_err(|_| error("final DOCX failed ZIP integrity validation"))?;
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let text =
                std::str::from_utf8(&bytes).map_err(|_| error("final DOCX XML is not UTF-8"))?;
            let mut reader = quick_xml::reader::NsReader::from_str(text);
            let mut depth = 0_usize;
            let mut roots = 0_usize;
            let mut body = false;
            let mut visible = String::new();
            loop {
                let (namespace, event) = reader
                    .read_resolved_event()
                    .map_err(|_| error("final DOCX contains malformed XML"))?;
                match event {
                    quick_xml::events::Event::Start(ref element)
                    | quick_xml::events::Event::Empty(ref element) => {
                        if depth == 0 {
                            roots += 1;
                            let expected = match name.as_str() {
                                "word/document.xml" => Some((
                                    b"document".as_slice(),
                                    "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
                                )),
                                "[Content_Types].xml" => Some((
                                    b"Types".as_slice(),
                                    "http://schemas.openxmlformats.org/package/2006/content-types",
                                )),
                                "_rels/.rels" => Some((
                                    b"Relationships".as_slice(),
                                    "http://schemas.openxmlformats.org/package/2006/relationships",
                                )),
                                _ => None,
                            };
                            if let Some((local, uri)) = expected {
                                if element.local_name().as_ref() != local
                                    || namespace
                                        != quick_xml::name::ResolveResult::Bound(
                                            quick_xml::name::Namespace(uri.as_bytes()),
                                        )
                                {
                                    return Err(error(
                                        "final DOCX has an invalid required part root",
                                    ));
                                }
                            }
                        }
                        if name == "word/document.xml" && depth == 1 && element.local_name().as_ref() == b"body"
                            && namespace == quick_xml::name::ResolveResult::Bound(quick_xml::name::Namespace(b"http://schemas.openxmlformats.org/wordprocessingml/2006/main")) {
                            body = true;
                        }
                        if matches!(event, quick_xml::events::Event::Start(_)) {
                            depth += 1;
                        }
                    }
                    quick_xml::events::Event::End(_) => {
                        depth = depth
                            .checked_sub(1)
                            .ok_or_else(|| error("final DOCX XML is unbalanced"))?;
                    }
                    quick_xml::events::Event::Text(text) => {
                        let text = text
                            .unescape()
                            .map_err(|_| error("final DOCX XML text is invalid"))?;
                        if depth == 0 && !text.trim().is_empty() {
                            return Err(error("final DOCX XML has text outside its root"));
                        }
                        visible.push_str(&text);
                    }
                    quick_xml::events::Event::Eof => break,
                    _ => {}
                }
            }
            if roots != 1 || depth != 0 || (name == "word/document.xml" && !body) {
                return Err(error("final DOCX XML is incomplete"));
            }
            if crate::RESERVED_MARKDOWN_TEMPLATE_VARIABLES
                .iter()
                .any(|token| visible.contains(&format!("{{{token}}}")))
            {
                return Err(error(
                    "final DOCX contains unfilled controlled placeholders",
                ));
            }
        }
    }
    for required in ["[Content_Types].xml", "_rels/.rels", "word/document.xml"] {
        if !parts.contains(required) {
            return Err(error("final DOCX is missing required package parts"));
        }
    }
    if !pdf.starts_with(b"%PDF-") {
        return Err(error("final PDF signature is invalid"));
    }
    if !pdf.trim_ascii_end().ends_with(b"%%EOF") {
        return Err(error("final PDF is incomplete (missing EOF marker)"));
    }
    let text = pdf_extract::extract_text_from_mem(pdf)
        .map_err(|_| error("final PDF could not be parsed"))?;
    if text.trim().is_empty() {
        return Err(error("final PDF contains no extractable document text"));
    }
    Ok(())
}

fn error(message: &str) -> ConversionError {
    ConversionError(message.to_owned())
}
fn io_error(e: std::io::Error) -> ConversionError {
    ConversionError(format!("conversion I/O failed: {e}"))
}
fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(io_error)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn canonical_file(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize().map_err(io_error)?;
    if !path.is_file() {
        return Err(error("expected a regular input file"));
    }
    Ok(path)
}
fn canonical_dir(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize().map_err(io_error)?;
    if !path.is_dir() {
        return Err(error("expected an existing directory"));
    }
    Ok(path)
}
fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}
fn new_output_path(path: &Path) -> Result<PathBuf> {
    if fs::symlink_metadata(path).is_ok() {
        return Err(error(
            "output root already exists; overwrite and resume are not supported",
        ));
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(error("output root must not contain parent traversal"));
    }
    let name = path
        .file_name()
        .ok_or_else(|| error("output root needs a new directory name"))?;
    portable_name(
        name.to_str()
            .ok_or_else(|| error("output name must be Unicode"))?,
    )?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(canonical_dir(parent)?.join(name))
}
fn reject_symlink_components(root: &Path, path: &Path) -> Result<()> {
    let mut current = root.to_path_buf();
    for part in path
        .strip_prefix(root)
        .map_err(|_| error("input escaped its root"))?
        .components()
    {
        if !matches!(part, Component::Normal(_)) {
            return Err(error("input path is not normalized"));
        }
        current.push(part);
        if fs::symlink_metadata(&current)
            .map_err(io_error)?
            .file_type()
            .is_symlink()
        {
            return Err(error(
                "symlink source/configuration/template paths are not supported",
            ));
        }
    }
    Ok(())
}
fn discover(
    root: &Path,
    directory: &Path,
    recursive: bool,
    selected: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(directory).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let name = entry.file_name();
        if [".dms", "assets"]
            .iter()
            .any(|excluded| name.to_string_lossy().eq_ignore_ascii_case(excluded))
        {
            continue;
        }
        let kind = entry.file_type().map_err(io_error)?;
        if kind.is_symlink() {
            return Err(error(
                "input discovery refuses symlinks outside asset subtrees",
            ));
        }
        if kind.is_dir() && recursive {
            discover(root, &entry.path(), true, selected)?;
        } else if kind.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            selected.push(
                entry
                    .path()
                    .strip_prefix(root)
                    .expect("discovered below root")
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}
fn portable_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return Err(error("output contains a non-portable Windows path name"));
    }
    let stem = name.split('.').next().unwrap_or_default().to_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.chars().count() == 4
            && "123456789¹²³".contains(&stem[3..])
    {
        return Err(error("output contains a reserved Windows device name"));
    }
    Ok(())
}
fn validate_destinations(selected: &[PathBuf]) -> Result<()> {
    let mut files = BTreeSet::from(["manifest.json".to_owned()]);
    let mut directories = BTreeMap::new();
    for source in selected {
        let mut prefix = String::new();
        for component in source.parent().unwrap_or(Path::new("")).components() {
            let part = component
                .as_os_str()
                .to_str()
                .ok_or_else(|| error("input path must be Unicode"))?;
            portable_name(part)?;
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            let key = prefix.to_lowercase();
            if directories
                .insert(key, prefix.clone())
                .is_some_and(|existing| existing != prefix)
            {
                return Err(error("case-insensitive output directory collision"));
            }
        }
        for extension in ["docx", "pdf"] {
            let target = source.with_extension(extension);
            let name = target
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| error("input filename must be Unicode"))?;
            portable_name(name)?;
            let key = target.to_string_lossy().replace('\\', "/").to_lowercase();
            if !files.insert(key) {
                return Err(error("case-insensitive output file collision"));
            }
        }
    }
    if directories
        .keys()
        .any(|directory| files.contains(directory))
    {
        return Err(error("output file/directory collision"));
    }
    Ok(())
}
fn ensure_output_parent(root: &Path, parent: &Path) -> Result<()> {
    if canonical_dir(root)? != root {
        return Err(error("output root changed during conversion"));
    }
    let mut current = root.to_path_buf();
    for part in parent
        .strip_prefix(root)
        .map_err(|_| error("output escaped its root"))?
        .components()
    {
        current.push(part);
        match fs::create_dir(&current) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(io_error(e)),
        }
        if fs::symlink_metadata(&current)
            .map_err(io_error)?
            .file_type()
            .is_symlink()
            || !current.is_dir()
        {
            return Err(error("output parent is not an owned directory"));
        }
    }
    Ok(())
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}
fn write_owned(path: &Path, bytes: &[u8], owned: &mut Vec<PathBuf>) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    owned.push(path.to_path_buf());
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}
fn read_regular_output(root: &Path, path: &Path) -> Result<Vec<u8>> {
    if canonical_dir(root)? != root {
        return Err(error("output root changed during conversion"));
    }
    reject_symlink_components(root, path)?;
    if !fs::symlink_metadata(path).map_err(io_error)?.is_file() {
        return Err(error("final output is not a regular file"));
    }
    read(path)
}
