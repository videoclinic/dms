use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read},
    path::Path,
};

use chrono::{DateTime, Utc};
use quick_xml::{
    escape::unescape,
    events::{BytesStart, Event as XmlEvent},
    Reader,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::{DmsError, Result};

const MAX_PACKAGE_PARTS: usize = 512;
const MAX_PACKAGE_UNCOMPRESSED_BYTES: u64 = 32 * 1024 * 1024;
const MAX_XML_PART_BYTES: usize = 2 * 1024 * 1024;
const MAX_DISPLAY_AUTHOR_CHARS: usize = 256;
const MAX_OBSERVATIONS: usize = 3;

type InspectedPackage<'a> = (ZipArchive<Cursor<&'a [u8]>>, BTreeSet<String>);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceHistoryFormat {
    Docx,
    Xlsx,
    Pptx,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceHistoryScanOutcome {
    AttributableRevisions,
    NoRevisionData,
    UnattributedRevisionData,
    MalformedPackage,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SourceChangeKind {
    WordInsertion,
    WordDeletion,
    WordMove,
    WordRowPropertyChange,
    WordParagraphPropertyChange,
    WordRunPropertyChange,
    WordMixedRevision,
    SpreadsheetRevision,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceChangeObservation {
    pub display_author: String,
    pub occurred_at: DateTime<Utc>,
    pub kind: SourceChangeKind,
    pub record_count: u32,
    pub timestamp_tied: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceHistory {
    pub imported_source_sha256: String,
    pub source_format: SourceHistoryFormat,
    pub scan_outcome: SourceHistoryScanOutcome,
    pub observations: Vec<SourceChangeObservation>,
}

impl SourceHistory {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.imported_source_sha256.len() != 64
            || !self
                .imported_source_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(DmsError::InvalidSourceHistory(
                "imported source digest must be a lowercase SHA-256 hex value".to_owned(),
            ));
        }
        if self.observations.len() > MAX_OBSERVATIONS {
            return Err(DmsError::InvalidSourceHistory(
                "source-history observations exceed the three-row limit".to_owned(),
            ));
        }
        let observations_are_attributable = !self.observations.is_empty();
        if observations_are_attributable
            != matches!(
                self.scan_outcome,
                SourceHistoryScanOutcome::AttributableRevisions
            )
        {
            return Err(DmsError::InvalidSourceHistory(
                "scan outcome does not match source-history observations".to_owned(),
            ));
        }
        for observation in &self.observations {
            if normalized_author(&observation.display_author).is_none()
                || observation.record_count == 0
                || !kind_matches_format(observation.kind, self.source_format)
            {
                return Err(DmsError::InvalidSourceHistory(
                    "source-history observation is invalid".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

pub(crate) fn capture_source_history(path: &Path) -> Result<Option<SourceHistory>> {
    let Some(source_format) = source_history_format(path) else {
        return Ok(None);
    };
    let bytes = fs::read(path).map_err(|source| DmsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let imported_source_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let (scan_outcome, observations) = scan_package(source_format, &bytes);
    Ok(Some(SourceHistory {
        imported_source_sha256,
        source_format,
        scan_outcome,
        observations,
    }))
}

fn source_history_format(path: &Path) -> Option<SourceHistoryFormat> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "docx" => Some(SourceHistoryFormat::Docx),
        "xlsx" => Some(SourceHistoryFormat::Xlsx),
        "pptx" => Some(SourceHistoryFormat::Pptx),
        _ => None,
    }
}

fn scan_package(
    source_format: SourceHistoryFormat,
    bytes: &[u8],
) -> (SourceHistoryScanOutcome, Vec<SourceChangeObservation>) {
    let Ok((mut archive, names)) = inspect_package(bytes) else {
        return (SourceHistoryScanOutcome::MalformedPackage, Vec::new());
    };
    match source_format {
        SourceHistoryFormat::Docx => match scan_word_revisions(&mut archive, &names) {
            Ok(observations) if observations.is_empty() => {
                (SourceHistoryScanOutcome::NoRevisionData, observations)
            }
            Ok(observations) => (
                SourceHistoryScanOutcome::AttributableRevisions,
                observations,
            ),
            Err(()) => (SourceHistoryScanOutcome::MalformedPackage, Vec::new()),
        },
        SourceHistoryFormat::Xlsx => match scan_spreadsheet_revisions(&mut archive, &names) {
            Ok(observations) if observations.is_empty() => {
                (SourceHistoryScanOutcome::NoRevisionData, observations)
            }
            Ok(observations) => (
                SourceHistoryScanOutcome::AttributableRevisions,
                observations,
            ),
            Err(()) => (SourceHistoryScanOutcome::MalformedPackage, Vec::new()),
        },
        SourceHistoryFormat::Pptx => {
            if has_powerpoint_revision_information(&mut archive, &names) {
                (
                    SourceHistoryScanOutcome::UnattributedRevisionData,
                    Vec::new(),
                )
            } else {
                (SourceHistoryScanOutcome::NoRevisionData, Vec::new())
            }
        }
    }
}

fn inspect_package(bytes: &[u8]) -> std::result::Result<InspectedPackage<'_>, ()> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| ())?;
    if archive.len() > MAX_PACKAGE_PARTS {
        return Err(());
    }
    let mut total_uncompressed = 0_u64;
    let mut names = BTreeSet::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|_| ())?;
        let name = entry.name();
        if !safe_part_name(name) || !names.insert(name.to_owned()) {
            return Err(());
        }
        total_uncompressed = total_uncompressed.checked_add(entry.size()).ok_or(())?;
        if total_uncompressed > MAX_PACKAGE_UNCOMPRESSED_BYTES {
            return Err(());
        }
    }
    Ok((archive, names))
}

fn safe_part_name(name: &str) -> bool {
    let name = name.strip_suffix('/').unwrap_or(name);
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

fn read_xml_part(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> std::result::Result<Vec<u8>, ()> {
    let part = archive.by_name(name).map_err(|_| ())?;
    if part.is_dir() || part.size() > MAX_XML_PART_BYTES as u64 {
        return Err(());
    }
    let mut bytes = Vec::with_capacity(part.size() as usize);
    part.take(MAX_XML_PART_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() > MAX_XML_PART_BYTES {
        return Err(());
    }
    Ok(bytes)
}

fn scan_word_revisions(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    names: &BTreeSet<String>,
) -> std::result::Result<Vec<SourceChangeObservation>, ()> {
    let mut grouped = BTreeMap::<(String, DateTime<Utc>), WordRevisionGroup>::new();
    for name in names
        .iter()
        .filter(|name| name.starts_with("word/") && name.ends_with(".xml"))
    {
        let bytes = read_xml_part(archive, name)?;
        let mut reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(XmlEvent::Start(element)) | Ok(XmlEvent::Empty(element)) => {
                    let Some(kind) = word_change_kind(local_name(element.name().as_ref())) else {
                        buffer.clear();
                        continue;
                    };
                    let attributes = attributes(&element)?;
                    let Some(author) =
                        attribute_by_local_name(&attributes, "author").and_then(normalized_author)
                    else {
                        buffer.clear();
                        continue;
                    };
                    let Some(occurred_at) =
                        attribute_by_local_name(&attributes, "date").and_then(parse_utc_timestamp)
                    else {
                        buffer.clear();
                        continue;
                    };
                    let group = grouped.entry((author, occurred_at)).or_default();
                    group.kinds.insert(kind);
                    group.record_count = group.record_count.checked_add(1).ok_or(())?;
                }
                Ok(XmlEvent::DocType(_)) => return Err(()),
                Ok(XmlEvent::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(()),
            }
            buffer.clear();
        }
    }
    Ok(finalize_observations(
        grouped
            .into_iter()
            .map(
                |((display_author, occurred_at), group)| PendingObservation {
                    display_author,
                    occurred_at,
                    kind: word_group_kind(&group.kinds),
                    record_count: group.record_count,
                },
            )
            .collect(),
    ))
}

#[derive(Default)]
struct WordRevisionGroup {
    kinds: BTreeSet<SourceChangeKind>,
    record_count: u32,
}

fn word_change_kind(name: &str) -> Option<SourceChangeKind> {
    match name {
        "ins" => Some(SourceChangeKind::WordInsertion),
        "del" => Some(SourceChangeKind::WordDeletion),
        "moveFrom" | "moveTo" | "moveFromRangeStart" | "moveFromRangeEnd" | "moveToRangeStart"
        | "moveToRangeEnd" => Some(SourceChangeKind::WordMove),
        "trPrChange" => Some(SourceChangeKind::WordRowPropertyChange),
        "pPrChange" => Some(SourceChangeKind::WordParagraphPropertyChange),
        "rPrChange" => Some(SourceChangeKind::WordRunPropertyChange),
        _ => None,
    }
}

fn word_group_kind(kinds: &BTreeSet<SourceChangeKind>) -> SourceChangeKind {
    if kinds.len() == 1 {
        *kinds.iter().next().expect("one kind")
    } else {
        SourceChangeKind::WordMixedRevision
    }
}

fn scan_spreadsheet_revisions(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    names: &BTreeSet<String>,
) -> std::result::Result<Vec<SourceChangeObservation>, ()> {
    let user_names = if names.contains("xl/revisions/userNames.xml") {
        parse_spreadsheet_user_names(&read_xml_part(archive, "xl/revisions/userNames.xml")?)?
    } else {
        Vec::new()
    };
    let revision_parts = names
        .iter()
        .filter(|name| {
            name.starts_with("xl/revisions/")
                && name.ends_with(".xml")
                && name.as_str() != "xl/revisions/userNames.xml"
        })
        .cloned()
        .collect::<Vec<_>>();
    if revision_parts.is_empty() {
        return Ok(Vec::new());
    }
    let mut grouped = BTreeMap::<(String, DateTime<Utc>), u32>::new();
    for name in revision_parts {
        let bytes = read_xml_part(archive, &name)?;
        let mut reader = Reader::from_reader(bytes.as_slice());
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(XmlEvent::Start(element)) | Ok(XmlEvent::Empty(element)) => {
                    let attributes = attributes(&element)?;
                    let Some(user_id) = attribute_by_local_name(&attributes, "userId")
                        .or_else(|| attribute_by_local_name(&attributes, "user"))
                        .and_then(|value| value.parse::<usize>().ok())
                    else {
                        buffer.clear();
                        continue;
                    };
                    let Some(display_author) = user_names
                        .get(user_id.saturating_sub(1))
                        .and_then(|value| value.as_deref())
                    else {
                        buffer.clear();
                        continue;
                    };
                    let Some(occurred_at) = attribute_by_local_name(&attributes, "date")
                        .or_else(|| attribute_by_local_name(&attributes, "dttm"))
                        .or_else(|| attribute_by_local_name(&attributes, "timestamp"))
                        .and_then(parse_utc_timestamp)
                    else {
                        buffer.clear();
                        continue;
                    };
                    let count = grouped
                        .entry((display_author.to_owned(), occurred_at))
                        .or_default();
                    *count = count.checked_add(1).ok_or(())?;
                }
                Ok(XmlEvent::DocType(_)) => return Err(()),
                Ok(XmlEvent::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(()),
            }
            buffer.clear();
        }
    }
    Ok(finalize_observations(
        grouped
            .into_iter()
            .map(
                |((display_author, occurred_at), record_count)| PendingObservation {
                    display_author,
                    occurred_at,
                    kind: SourceChangeKind::SpreadsheetRevision,
                    record_count,
                },
            )
            .collect(),
    ))
}

fn parse_spreadsheet_user_names(bytes: &[u8]) -> std::result::Result<Vec<Option<String>>, ()> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut users = Vec::new();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(XmlEvent::Start(element)) | Ok(XmlEvent::Empty(element)) => {
                if local_name(element.name().as_ref()) == "user" {
                    let attributes = attributes(&element)?;
                    users.push(
                        attribute_by_local_name(&attributes, "userName")
                            .or_else(|| attribute_by_local_name(&attributes, "name"))
                            .and_then(normalized_author),
                    );
                }
            }
            Ok(XmlEvent::DocType(_)) => return Err(()),
            Ok(XmlEvent::Eof) => break,
            Ok(_) => {}
            Err(_) => return Err(()),
        }
        buffer.clear();
    }
    Ok(users)
}

fn has_powerpoint_revision_information(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    names: &BTreeSet<String>,
) -> bool {
    if names
        .iter()
        .any(|name| name.starts_with("ppt/revisionInfo/"))
    {
        return true;
    }
    let Some(relationships) = names
        .iter()
        .find(|name| name.as_str() == "ppt/_rels/presentation.xml.rels")
    else {
        return false;
    };
    let Ok(bytes) = read_xml_part(archive, relationships) else {
        return true;
    };
    let mut reader = Reader::from_reader(bytes.as_slice());
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(XmlEvent::Start(element)) | Ok(XmlEvent::Empty(element)) => {
                if let Ok(attributes) = attributes(&element) {
                    if attribute_by_local_name(&attributes, "Type")
                        .is_some_and(|value| value.ends_with("/revisionInfo"))
                    {
                        return true;
                    }
                }
            }
            Ok(XmlEvent::Eof) => return false,
            Ok(_) => {}
            Err(_) => return true,
        }
        buffer.clear();
    }
}

#[derive(Clone)]
struct PendingObservation {
    display_author: String,
    occurred_at: DateTime<Utc>,
    kind: SourceChangeKind,
    record_count: u32,
}

fn finalize_observations(
    mut observations: Vec<PendingObservation>,
) -> Vec<SourceChangeObservation> {
    observations.sort_by(|left, right| {
        right
            .occurred_at
            .cmp(&left.occurred_at)
            .then_with(|| left.display_author.cmp(&right.display_author))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    let mut retained = Vec::new();
    let mut start = 0;
    while start < observations.len() && retained.len() < MAX_OBSERVATIONS {
        let timestamp = observations[start].occurred_at;
        let mut end = start + 1;
        while end < observations.len() && observations[end].occurred_at == timestamp {
            end += 1;
        }
        let timestamp_tied = end - start > 1;
        if retained.len() + (end - start) > MAX_OBSERVATIONS {
            break;
        }
        retained.extend(observations[start..end].iter().map(|observation| {
            SourceChangeObservation {
                display_author: observation.display_author.clone(),
                occurred_at: observation.occurred_at,
                kind: observation.kind,
                record_count: observation.record_count,
                timestamp_tied,
            }
        }));
        start = end;
    }
    retained
}

fn attributes(element: &BytesStart<'_>) -> std::result::Result<BTreeMap<String, String>, ()> {
    let mut values = BTreeMap::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|_| ())?;
        let name = std::str::from_utf8(attribute.key.as_ref()).map_err(|_| ())?;
        let value = std::str::from_utf8(attribute.value.as_ref()).map_err(|_| ())?;
        let value = unescape(value).map_err(|_| ())?.into_owned();
        if values.insert(name.to_owned(), value).is_some() {
            return Err(());
        }
    }
    Ok(values)
}

fn attribute_by_local_name<'a>(
    attributes: &'a BTreeMap<String, String>,
    name: &str,
) -> Option<&'a str> {
    attributes
        .iter()
        .find_map(|(key, value)| (local_name(key.as_bytes()) == name).then_some(value.as_str()))
}

fn local_name(name: &[u8]) -> &str {
    std::str::from_utf8(name)
        .ok()
        .and_then(|name| name.rsplit(':').next())
        .unwrap_or("")
}

fn normalized_author(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()
        && value.chars().count() <= MAX_DISPLAY_AUTHOR_CHARS
        && !value.chars().any(char::is_control))
    .then(|| value.to_owned())
}

fn parse_utc_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value.trim())
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn kind_matches_format(kind: SourceChangeKind, source_format: SourceHistoryFormat) -> bool {
    matches!(
        (kind, source_format),
        (
            SourceChangeKind::SpreadsheetRevision,
            SourceHistoryFormat::Xlsx
        ) | (
            SourceChangeKind::WordInsertion
                | SourceChangeKind::WordDeletion
                | SourceChangeKind::WordMove
                | SourceChangeKind::WordRowPropertyChange
                | SourceChangeKind::WordParagraphPropertyChange
                | SourceChangeKind::WordRunPropertyChange
                | SourceChangeKind::WordMixedRevision,
            SourceHistoryFormat::Docx
        )
    )
}
