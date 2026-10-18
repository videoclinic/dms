use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{DmsError, Result, SourceState, Version, Workspace};

pub const PUBLISH_INDEX_FILENAME: &str = "index.html";
pub const PUBLISH_INDEX_OWNERSHIP_MARKER: &str = "<!-- DMS-owned publish index -->";
const INDEX_PREFIX: &str = "<!doctype html>\n<!-- DMS-owned publish index -->\n";
const LEGACY_TITLE: &str = "Title unrecorded (legacy release)";

/// Immutable release metadata only; no source locator or integrity verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishIndexRow {
    pub document_id: Uuid,
    pub release_id: Uuid,
    pub title: String,
    pub document_number: Option<String>,
    pub version: Version,
    pub released_at: DateTime<Utc>,
    /// Validated, normalized '/'-separated path relative to the publish root.
    pub relative_pdf_path: String,
    pub confidentiality_type_id: String,
}

/// Validated projection. Private storage prevents rendering unvalidated paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishIndex {
    rows: Vec<PublishIndexRow>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishIndexStatus {
    Written,
    Unchanged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PublishIndexOutcome {
    pub path: PathBuf,
    pub status: PublishIndexStatus,
    pub document_count: usize,
}

impl Workspace {
    /// Inspect only recorded current PDF targets, never discover files or read
    /// source/PDF bytes. Missing current PDFs do not resurrect older versions.
    pub fn publish_index(&self) -> Result<PublishIndex> {
        let root = fs::canonicalize(&self.publish_root)
            .map_err(|source| index_io(&self.publish_root, source))?;
        let mut rows = Vec::new();
        for document in self.documents.values() {
            if document.source_state != SourceState::Registered {
                continue;
            }
            let Some(release) = self.current_release(document.id)? else {
                continue;
            };
            let path = normalized_release_path(&release.relative_pdf_path)?;
            if !recorded_pdf_exists(&root, &path)? {
                continue;
            }
            rows.push(PublishIndexRow {
                document_id: document.id,
                release_id: release.id,
                title: release
                    .control
                    .as_ref()
                    .map_or_else(|| LEGACY_TITLE.to_owned(), |control| control.title.clone()),
                document_number: release
                    .control
                    .as_ref()
                    .and_then(|control| control.document_number.clone()),
                version: release.version,
                released_at: release.released_at,
                relative_pdf_path: path,
                confidentiality_type_id: release.confidentiality.type_id.clone(),
            });
        }
        rows.sort_by(|left, right| {
            right
                .released_at
                .cmp(&left.released_at)
                .then_with(|| name_order(left, right))
        });
        Ok(PublishIndex { rows })
    }

    /// Refresh only the owned publish-root index. Callers coordinate workspace
    /// access; this does not acquire a lock, save metadata, or verify releases.
    pub fn refresh_publish_index(&self) -> Result<PublishIndexOutcome> {
        let index = self.publish_index()?;
        let html = index.render_html();
        let path = self.publish_root.join(PUBLISH_INDEX_FILENAME);
        let status = write_index(&path, html.as_bytes(), |file, bytes| {
            file.write_all(bytes)?;
            file.flush()?;
            file.sync_all()
        })?;
        Ok(PublishIndexOutcome {
            path,
            status,
            document_count: index.rows.len(),
        })
    }

    /// Resolve the publish-root index only when an owned index file exists.
    /// Missing index yields `None` so callers can prompt a refresh first.
    /// Foreign, symlinked, or non-regular targets reuse the owned-index
    /// validator and surface `ForeignPublishIndex`.
    pub fn existing_publish_index_path(&self) -> Result<Option<PathBuf>> {
        let path = self.publish_root.join(PUBLISH_INDEX_FILENAME);
        match read_owned_index(&path)? {
            Some(_) => Ok(Some(path)),
            None => Ok(None),
        }
    }
}

fn name_order(left: &PublishIndexRow, right: &PublishIndexRow) -> std::cmp::Ordering {
    left.title
        .to_lowercase()
        .cmp(&right.title.to_lowercase())
        .then_with(|| left.title.cmp(&right.title))
        .then_with(|| left.relative_pdf_path.cmp(&right.relative_pdf_path))
        .then_with(|| left.document_id.cmp(&right.document_id))
}

impl PublishIndex {
    pub fn rows(&self) -> &[PublishIndexRow] {
        &self.rows
    }

    /// Pure, deterministic standalone HTML. Data occurs only in escaped text
    /// and attributes; the inline script is constant, never a data template.
    pub fn render_html(&self) -> String {
        let mut html = String::from(INDEX_PREFIX);
        html.push_str("<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Released document library</title><style>\n");
        html.push_str(include_str!("publish_index.css"));
        html.push_str("</style></head><body><header><h1>Released document library</h1><p>Current released PDFs only. Release history and integrity verification remain in DMS.</p></header><div class=\"library\"><nav aria-label=\"Release folder tree\"><h2>Folders</h2><button type=\"button\" data-folder=\"\" aria-pressed=\"true\">All folders</button>");
        let mut tree = Folder::default();
        for row in &self.rows {
            let parent = row.relative_pdf_path.rsplit_once('/').map(|(p, _)| p);
            if let Some(parent) = parent {
                let mut folder = &mut tree;
                for part in parent.split('/') {
                    folder = folder.children.entry(part.to_owned()).or_default();
                }
            }
        }
        tree.render(&mut html, "");
        html.push_str("</nav><main><section aria-label=\"Library controls\"><label>Search release metadata <input id=\"search\" type=\"search\"></label><label>Sort <select id=\"sort\"><option value=\"date-desc\">Last release date: newest first</option><option value=\"date-asc\">Last release date: oldest first</option><option value=\"name-asc\">Document name: ascending</option><option value=\"name-desc\">Document name: descending</option></select></label><fieldset><legend>Confidentiality type ID (on/off)</legend><label><input id=\"all-types\" type=\"checkbox\" checked> All</label>");
        let types: BTreeSet<_> = self
            .rows
            .iter()
            .map(|r| &r.confidentiality_type_id)
            .collect();
        for type_id in types {
            let escaped = escape_html(type_id);
            write!(html, "<label><input type=\"checkbox\" data-type=\"{escaped}\" checked> {escaped}</label>").unwrap();
        }
        html.push_str(
            "</fieldset></section><p id=\"result-count\" role=\"status\" aria-live=\"polite\">",
        );
        write!(html, "{} documents", self.rows.len()).unwrap();
        html.push_str("</p><p id=\"empty\"");
        if !self.rows.is_empty() {
            html.push_str(" hidden");
        }
        html.push_str(">No released documents match these controls.</p><noscript>Search, filters and sorting require JavaScript. All current PDF links are listed below.</noscript><div class=\"table-scroll\"><table><caption>Current released documents</caption><thead><tr><th scope=\"col\">Document name</th><th scope=\"col\">Document number</th><th scope=\"col\">Version</th><th scope=\"col\">Last release date (UTC)</th><th scope=\"col\">Type ID</th><th scope=\"col\">Relative PDF path</th></tr></thead><tbody id=\"documents\">\n");
        let mut by_name: Vec<_> = self.rows.iter().enumerate().collect();
        by_name.sort_by(|(_, left), (_, right)| name_order(left, right));
        let name_ranks: BTreeMap<_, _> = by_name
            .iter()
            .enumerate()
            .map(|(rank, (index, _))| (*index, rank))
            .collect();
        for (index, row) in self.rows.iter().enumerate() {
            let parent = row
                .relative_pdf_path
                .rsplit_once('/')
                .map_or("", |(p, _)| p);
            writeln!(html, "<tr data-folder=\"{}\" data-type=\"{}\" data-date-order=\"{index}\" data-name-order=\"{}\"><td><a href=\"{}\">{}</a></td><td>{}</td><td>{}</td><td><time>{}</time></td><td>{}</td><td>{}</td></tr>",
                escape_html(parent), escape_html(&row.confidentiality_type_id), name_ranks[&index],
                encoded_href(&row.relative_pdf_path), escape_html(&row.title),
                escape_html(row.document_number.as_deref().unwrap_or("Not recorded")),
                row.version, row.released_at.to_rfc3339_opts(SecondsFormat::Nanos, true),
                escape_html(&row.confidentiality_type_id), escape_html(&row.relative_pdf_path)).unwrap();
        }
        html.push_str("</tbody></table></div></main></div><script>\n");
        html.push_str(include_str!("publish_index.js"));
        html.push_str("</script></body></html>\n");
        html
    }
}

#[derive(Default)]
struct Folder {
    children: BTreeMap<String, Folder>,
}

impl Folder {
    fn render(&self, html: &mut String, parent: &str) {
        if self.children.is_empty() {
            return;
        }
        html.push_str("<ul>");
        for (name, child) in &self.children {
            let path = if parent.is_empty() {
                name.clone()
            } else {
                format!("{parent}/{name}")
            };
            write!(
                html,
                "<li><button type=\"button\" data-folder=\"{}\" aria-pressed=\"false\">{}</button>",
                escape_html(&path),
                escape_html(name)
            )
            .unwrap();
            child.render(html, &path);
            html.push_str("</li>");
        }
        html.push_str("</ul>");
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn encoded_href(path: &str) -> String {
    let mut encoded = String::from("./");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").unwrap();
        }
    }
    encoded
}

fn normalized_release_path(path: &Path) -> Result<String> {
    let invalid = || DmsError::InvalidReleasePath(path.to_path_buf());
    let text = path.to_str().ok_or_else(invalid)?;
    // Windows PathBufs use native separators; Unix backslashes are not paths.
    #[cfg(windows)]
    let text = text.replace('\\', "/");
    if text.contains(['\\', ':']) || text.chars().any(char::is_control) {
        return Err(invalid());
    }
    for segment in text.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." || segment.ends_with(['.', ' '])
        {
            return Err(invalid());
        }
        // Refuse Windows device aliases even when generating on another OS.
        // Windows ignores ASCII spaces before the first dot for this check.
        let stem = segment
            .split('.')
            .next()
            .unwrap()
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|prefix| {
                stem.strip_prefix(prefix).is_some_and(|n| {
                    matches!(
                        n,
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                })
            })
        {
            return Err(invalid());
        }
    }
    if !text.to_ascii_lowercase().ends_with(".pdf") {
        return Err(invalid());
    }
    Ok(text.to_string())
}

fn recorded_pdf_exists(root: &Path, relative: &str) -> Result<bool> {
    let mut path = root.to_path_buf();
    let mut parts = relative.split('/').peekable();
    while let Some(part) = parts.next() {
        path.push(part);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(index_io(&path, error)),
        };
        if metadata.file_type().is_symlink()
            || (parts.peek().is_some() && !metadata.is_dir())
            || (parts.peek().is_none() && !metadata.is_file())
        {
            return Err(DmsError::InvalidReleasePath(path));
        }
        // Also catches platform redirections such as Windows junctions.
        let canonical = fs::canonicalize(&path).map_err(|error| index_io(&path, error))?;
        if !canonical.starts_with(root) || canonical == root {
            return Err(DmsError::InvalidReleasePath(path));
        }
    }
    Ok(true)
}

fn read_owned_index(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(DmsError::ForeignPublishIndex(path.to_path_buf()));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(index_io(path, error)),
    }
    let bytes = fs::read(path).map_err(|error| index_io(path, error))?;
    // Exact prologue, not a marker mentioned in unrelated page content.
    if !bytes.starts_with(INDEX_PREFIX.as_bytes()) {
        return Err(DmsError::ForeignPublishIndex(path.to_path_buf()));
    }
    Ok(Some(bytes))
}

fn write_index(
    path: &Path,
    bytes: &[u8],
    write: impl FnOnce(&mut File, &[u8]) -> io::Result<()>,
) -> Result<PublishIndexStatus> {
    let previous = read_owned_index(path)?;
    if previous.as_deref() == Some(bytes) {
        return Ok(PublishIndexStatus::Unchanged);
    }
    let mut temporary = tempfile::Builder::new()
        .prefix(".dms-index-")
        .suffix(".tmp")
        .tempfile_in(path.parent().expect("index has a publish-root parent"))
        .map_err(|error| index_io(path, error))?;
    write(temporary.as_file_mut(), bytes).map_err(|error| index_io(path, error))?;
    // Refuse a foreign or changed target discovered while preparing the sibling.
    if read_owned_index(path)? != previous {
        return Err(DmsError::PublishIndexChanged(path.to_path_buf()));
    }
    let persisted = if previous.is_some() {
        temporary.persist(path)
    } else {
        temporary.persist_noclobber(path)
    };
    // PersistError owns the temporary file: dropping it cleans failed writes.
    persisted.map_err(|error| index_io(path, error.error))?;
    Ok(PublishIndexStatus::Written)
}

fn index_io(path: &Path, source: io::Error) -> DmsError {
    DmsError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
#[path = "publish_index_tests.rs"]
mod tests;
