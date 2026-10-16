//! Shared installed-Office export adapter.
//!
//! This crate owns the single implementation of controlled-placeholder filling
//! and native Microsoft Word automation. Both the standalone conversion
//! (`InstalledWord`, driven through the core `ConversionExporter` port) and the
//! desktop release path (`LocalPdfExporter` in `dms-desktop`) consume it, so no
//! COM/AppleScript logic is duplicated. It stays outside `dms-core` and is
//! independent of Tauri.

use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

#[cfg(any(windows, target_os = "macos"))]
use std::process::Command;

use dms_core::{ConversionExporter, ExportChrome};
use quick_xml::{events::Event, Reader, Writer};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

/// Fill the reserved controlled-field tokens into a copy of an OOXML package.
///
/// `{TITLE}`, `{DOCUMENT_NUMBER}`, `{VERSION}` and `{CONFIDENTIALITY}` are
/// replaced across every XML part (body, headers/footers and Word custom
/// document-property values). The source package is never modified.
pub fn fill_office_placeholders(
    source: &Path,
    destination: &Path,
    chrome: &ExportChrome,
) -> Result<(), String> {
    let source_file = fs::File::open(source)
        .map_err(|error| format!("cannot open Office draft {}: {error}", source.display()))?;
    let mut archive = ZipArchive::new(source_file).map_err(|error| {
        format!(
            "Office draft {} is not valid OOXML: {error}",
            source.display()
        )
    })?;
    let destination_file = fs::File::create(destination).map_err(|error| {
        format!(
            "cannot create temporary Office copy {}: {error}",
            destination.display()
        )
    })?;
    let mut writer = ZipWriter::new(destination_file);

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("cannot read Office package entry: {error}"))?;
        let name = entry.name().to_owned();
        let options = SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            writer
                .add_directory(name, options)
                .map_err(|error| format!("cannot copy Office package directory: {error}"))?;
            continue;
        }
        writer
            .start_file(&name, options)
            .map_err(|error| format!("cannot copy Office package entry {name}: {error}"))?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| format!("cannot read Office package entry {name}: {error}"))?;
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let text = String::from_utf8(bytes)
                .map_err(|error| format!("Office XML entry {name} is not UTF-8: {error}"))?;
            let filled = fill_xml_placeholders(&text, chrome)
                .map_err(|error| format!("cannot fill Office XML entry {name}: {error}"))?;
            writer
                .write_all(&filled)
                .map_err(|error| format!("cannot write Office package entry {name}: {error}"))?;
        } else {
            writer
                .write_all(&bytes)
                .map_err(|error| format!("cannot write Office package entry {name}: {error}"))?;
        }
    }
    writer
        .finish()
        .map_err(|error| format!("cannot finish temporary Office copy: {error}"))?;
    Ok(())
}

fn fill_xml_placeholders(xml: &str, chrome: &ExportChrome) -> Result<Vec<u8>, String> {
    let mut reader = Reader::from_str(xml);
    let mut events = Vec::new();
    let mut text_values = Vec::new();
    loop {
        let event = reader
            .read_event()
            .map_err(|error| format!("invalid XML: {error}"))?;
        match &event {
            Event::Text(text) => text_values.push(
                text.unescape()
                    .map_err(|error| format!("invalid XML text: {error}"))?
                    .into_owned(),
            ),
            Event::Eof => {
                events.push(event.into_owned());
                break;
            }
            _ => {}
        }
        events.push(event.into_owned());
    }

    replace_across_text_nodes(
        &mut text_values,
        &[
            ("{CONFIDENTIALITY}", chrome.confidentiality.label.as_str()),
            ("{VERSION}", chrome.version_label.as_str()),
            ("{TITLE}", chrome.title.as_str()),
            (
                "{DOCUMENT_NUMBER}",
                chrome.document_number.as_deref().unwrap_or_default(),
            ),
        ],
    );

    let mut output = Writer::new(Vec::new());
    let mut text_index = 0;
    for event in events {
        let event = if matches!(&event, Event::Text(_)) {
            let value = &text_values[text_index];
            text_index += 1;
            Event::Text(quick_xml::events::BytesText::new(value))
        } else {
            event
        };
        output
            .write_event(event)
            .map_err(|error| format!("cannot write XML: {error}"))?;
    }
    Ok(output.into_inner())
}

fn replace_across_text_nodes(values: &mut [String], replacements: &[(&str, &str)]) {
    // Match original tokens only: inserted values cannot recurse or cascade.
    let combined = values.concat();
    let mut matches = Vec::new();
    for (token, replacement) in replacements {
        for (start, _) in combined.match_indices(token) {
            matches.push((start, start + token.len(), *replacement));
        }
    }
    matches.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
    for (start, end, replacement) in matches {
        let mut cursor = 0;
        let mut start_node = None;
        let mut end_node = None;
        for (index, value) in values.iter().enumerate() {
            let next = cursor + value.len();
            if start_node.is_none() && start < next {
                start_node = Some((index, start - cursor));
            }
            if end_node.is_none() && end <= next {
                end_node = Some((index, end - cursor));
                break;
            }
            cursor = next;
        }
        let (start_index, start_offset) = start_node.expect("token start belongs to a text node");
        let (end_index, end_offset) = end_node.expect("token end belongs to a text node");
        if start_index == end_index {
            values[start_index].replace_range(start_offset..end_offset, replacement);
            continue;
        }
        let prefix = values[start_index][..start_offset].to_owned();
        let suffix = values[end_index][end_offset..].to_owned();
        values[start_index] = format!("{prefix}{replacement}");
        for value in &mut values[start_index + 1..end_index] {
            value.clear();
        }
        values[end_index] = suffix;
    }
}

/// Shared installed-Word adapter for standalone batch conversion.
///
/// The core `ConversionExporter` port receives an already assembled final
/// `docx` in the caller-owned output tree. `export` fills the controlled-field
/// placeholders in place through a sibling staging copy, has Word update fields,
/// tables of contents and pagination and save that same document, then exports
/// the identical document to PDF and returns the Word version receipt.
#[derive(Default)]
pub struct InstalledWord<A = NativeWord> {
    automation: A,
}

impl InstalledWord<NativeWord> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<A: WordAutomation> InstalledWord<A> {
    pub fn with_automation(automation: A) -> Self {
        Self { automation }
    }
}

/// Explicit adapter seam. Production uses NativeWord, never an environment fake.
pub trait WordAutomation {
    fn preflight(&self) -> Result<(), String>;
    fn export_pdf(&mut self, docx: &Path, pdf: &Path, retain_docx: bool) -> Result<String, String>;
}

#[derive(Default)]
pub struct NativeWord;

impl WordAutomation for NativeWord {
    fn preflight(&self) -> Result<(), String> {
        installed_word_preflight()
    }
    fn export_pdf(&mut self, docx: &Path, pdf: &Path, retain_docx: bool) -> Result<String, String> {
        export_with_installed_word(docx, pdf, retain_docx)
    }
}

impl<A: WordAutomation> ConversionExporter for InstalledWord<A> {
    fn preflight(&self) -> std::result::Result<(), String> {
        self.automation.preflight()
    }

    fn export(
        &mut self,
        docx: &Path,
        pdf: &Path,
        chrome: &ExportChrome,
    ) -> std::result::Result<String, String> {
        self.automation.preflight()?;
        fill_in_place(docx, chrome)?;
        clear_reserved_output(pdf)?;
        let version = self.automation.export_pdf(docx, pdf, true)?;
        validate_docx(docx)?;
        validate_pdf(pdf)?;
        Ok(version)
    }
}

/// Report whether this host can drive installed Microsoft Word. Fails clearly on
/// unsupported hosts and when Word is not installed or not reachable.
pub fn installed_word_preflight() -> Result<(), String> {
    preflight_installed_word()
}

fn clear_reserved_output(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() == 0 => {}
        Ok(_) => return Err("PDF destination is not an empty owned reservation".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("cannot inspect reserved PDF: {error}")),
    }
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "cannot clear reserved output {}: {error}",
            path.display()
        )),
    }
}

fn fill_in_place(docx: &Path, chrome: &ExportChrome) -> Result<(), String> {
    let parent = docx.parent().ok_or("DOCX has no parent directory")?;
    let staging = tempfile::tempdir_in(parent)
        .map_err(|error| format!("cannot create private DOCX staging directory: {error}"))?;
    let staged = staging.path().join("filled.docx");
    fill_office_placeholders(docx, &staged, chrome)?;
    fs::rename(&staged, docx).map_err(|error| format!("cannot replace converted DOCX: {error}"))?;
    Ok(())
}

fn validate_docx(path: &Path) -> Result<(), String> {
    let file = fs::File::open(path)
        .map_err(|error| format!("cannot read converted DOCX {}: {error}", path.display()))?;
    let mut archive = ZipArchive::new(file).map_err(|error| {
        format!(
            "converted DOCX {} is not a valid OOXML package: {error}",
            path.display()
        )
    })?;
    archive.by_name("word/document.xml").map_err(|error| {
        format!(
            "converted DOCX {} is missing word/document.xml: {error}",
            path.display()
        )
    })?;
    Ok(())
}

fn validate_pdf(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read generated PDF {}: {error}", path.display()))?;
    if bytes.len() <= 5 || !bytes.starts_with(b"%PDF-") {
        return Err(format!(
            "export adapter did not produce a valid PDF at {}",
            path.display()
        ));
    }
    Ok(())
}

/// Installed-Microsoft-Word release export used by the desktop adapter.
///
/// Mirrors the desktop release contract: the temporary copy is opened read-only,
/// fields and tables of contents are refreshed, and the document is exported to
/// PDF without modifying the supplied copy.
pub fn export_installed_office_pdf(source_copy: &Path, output: &Path) -> Result<(), String> {
    export_with_installed_word(source_copy, output, false).map(|_| ())
}

#[cfg(any(windows, test))]
const WINDOWS_WORD_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$retain = $env:DMS_OFFICE_RETAIN_DOCX -eq '1'
$word = $null
$document = $null
$wordPidsBefore = @(Get-Process -Name WINWORD -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
$startedWord = $false
try {
  $word = New-Object -ComObject Word.Application
  $startedWord = @(Get-Process -Name WINWORD -ErrorAction SilentlyContinue | Where-Object { $wordPidsBefore -notcontains $_.Id }).Count -gt 0
  # Never mutate application-wide visibility or alerts: process observation
  # cannot prove that this COM object is the newly observed Word process.
  $document = $word.Documents.Open($env:DMS_OFFICE_SOURCE, $false, (-not $retain))
  if ($retain -and $document.ReadOnly) { throw 'Retained DOCX opened read-only' }
  foreach ($tableOfContents in $document.TablesOfContents) {
    $tableOfContents.Update() | Out-Null
  }
  foreach ($story in $document.StoryRanges) {
    $range = $story
    while ($null -ne $range) {
      $range.Fields.Update() | Out-Null
      $range = $range.NextStoryRange
    }
  }
  if ($retain) {
    $document.Repaginate()
    foreach ($tableOfContents in $document.TablesOfContents) {
      $tableOfContents.UpdatePageNumbers() | Out-Null
    }
    $document.Save()
    if (-not $document.Saved) { throw 'Retained DOCX save was not confirmed' }
  }
  $document.ExportAsFixedFormat($env:DMS_OFFICE_OUTPUT, 17)
  [Console]::Out.Write($word.Version)
} finally {
  try {
    if ($null -ne $document) { $saveChanges = 0; $document.Close([ref]$saveChanges) }
  } finally {
    # A Close failure must not skip the owned-instance cleanup. If the document
    # count cannot be read, do not guess: keeping an instance is safer than
    # quitting one that may have documents opened by the operator.
    $documentCount = $null
    if ($startedWord -and ($null -ne $word)) {
      try { $documentCount = $word.Documents.Count } catch {}
      if ($documentCount -eq 0) { $saveChanges = 0; $word.Quit([ref]$saveChanges) }
    }
  }
}
"#;

#[cfg(windows)]
fn export_with_installed_word(
    source_copy: &Path,
    output: &Path,
    retain_docx: bool,
) -> Result<String, String> {
    run_office_command_capture(
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                WINDOWS_WORD_SCRIPT,
            ])
            .env("DMS_OFFICE_SOURCE", office_compatible_path(source_copy))
            .env("DMS_OFFICE_OUTPUT", office_compatible_path(output))
            .env(
                "DMS_OFFICE_RETAIN_DOCX",
                if retain_docx { "1" } else { "0" },
            ),
        "Microsoft Word",
    )
}

#[cfg(any(target_os = "macos", test))]
const MACOS_WORD_SCRIPT: &str = r#"
set sourcePath to system attribute "DMS_OFFICE_SOURCE"
set outputPath to system attribute "DMS_OFFICE_OUTPUT"
set retainDocx to (system attribute "DMS_OFFICE_RETAIN_DOCX") is "1"
set sourceFile to (POSIX file sourcePath) as alias
set sourceDocument to missing value
set wasRunning to application "Microsoft Word" is running
tell application "Microsoft Word"
  try
    if (not retainDocx) and (not wasRunning) then set visible to false
    set openedDocument to open file name sourcePath
    if openedDocument is missing value then error "Word did not return the opened document"
    set openedName to full name of openedDocument as text
    if openedName starts with "/" then
      set openedFile to (POSIX file openedName) as alias
    else
      set openedFile to openedName as alias
    end if
    if openedFile is not sourceFile then error "Word returned a different document"
    set sourceDocument to openedDocument
    update field every field of sourceDocument
    repeat with documentSection in sections of sourceDocument
      repeat with headerKind in {header footer primary, header footer first page, header footer even pages}
        set storyHeader to get header of documentSection index headerKind
        update field every field of text object of storyHeader
        set storyFooter to get footer of documentSection index headerKind
        update field every field of text object of storyFooter
      end repeat
    end repeat
    repeat with documentToc in tables of contents of sourceDocument
      update documentToc
    end repeat
    if retainDocx then
      repaginate sourceDocument
      repeat with documentToc in tables of contents of sourceDocument
        update page numbers documentToc
      end repeat
      save sourceDocument
      if not saved of sourceDocument then error "Retained DOCX save was not confirmed"
    end if
    set appVersion to version
    save as sourceDocument file name outputPath file format format PDF
    close sourceDocument saving no
    set sourceDocument to missing value
    try
      if (not wasRunning) and (count of documents is 0) then quit
    end try
    return appVersion
  on error errorMessage number errorNumber
    if sourceDocument is not missing value then
      try
        close sourceDocument saving no
      end try
    end if
    try
      if (not wasRunning) and (count of documents is 0) then quit
    end try
    error "Microsoft Word conversion failed" number errorNumber
  end try
end tell
"#;

#[cfg(target_os = "macos")]
fn export_with_installed_word(
    source_copy: &Path,
    output: &Path,
    retain_docx: bool,
) -> Result<String, String> {
    run_office_command_capture(
        Command::new("osascript")
            .args(["-e", MACOS_WORD_SCRIPT])
            .env("DMS_OFFICE_SOURCE", source_copy)
            .env("DMS_OFFICE_OUTPUT", output)
            .env(
                "DMS_OFFICE_RETAIN_DOCX",
                if retain_docx { "1" } else { "0" },
            ),
        "Microsoft Word",
    )
}

#[cfg(not(any(windows, target_os = "macos")))]
fn export_with_installed_word(
    _source_copy: &Path,
    _output: &Path,
    _retain_docx: bool,
) -> Result<String, String> {
    Err("installed Office export is supported only on Windows and macOS".to_owned())
}

#[cfg(windows)]
fn preflight_installed_word() -> Result<(), String> {
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
if ($null -eq [type]::GetTypeFromProgID('Word.Application')) { throw 'Microsoft Word is not installed' }
"#;
    let result = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .map_err(|error| format!("cannot check Microsoft Word installation: {error}"))?;
    if !result.status.success() {
        return Err("Microsoft Word is not installed or unavailable to automation".to_owned());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn preflight_installed_word() -> Result<(), String> {
    let result = Command::new("osascript")
        .args(["-e", "id of application \"Microsoft Word\""])
        .output()
        .map_err(|error| format!("cannot check Microsoft Word installation: {error}"))?;
    if !result.status.success() {
        return Err("Microsoft Word is not installed or unavailable to automation".to_owned());
    }
    Ok(())
}

#[cfg(not(any(windows, target_os = "macos")))]
fn preflight_installed_word() -> Result<(), String> {
    Err("installed Office export is supported only on Windows and macOS".to_owned())
}

#[cfg(windows)]
fn office_compatible_path(path: &Path) -> std::ffi::OsString {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    const VERBATIM_PREFIX: &[u16] = &[b'\\' as u16, b'\\' as u16, b'?' as u16, b'\\' as u16];
    const VERBATIM_UNC_PREFIX: &[u16] = &[
        b'\\' as u16,
        b'\\' as u16,
        b'?' as u16,
        b'\\' as u16,
        b'U' as u16,
        b'N' as u16,
        b'C' as u16,
        b'\\' as u16,
    ];

    let path = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if let Some(remainder) = path.strip_prefix(VERBATIM_UNC_PREFIX) {
        let mut compatible = vec![b'\\' as u16, b'\\' as u16];
        compatible.extend_from_slice(remainder);
        std::ffi::OsString::from_wide(&compatible)
    } else if let Some(remainder) = path.strip_prefix(VERBATIM_PREFIX) {
        std::ffi::OsString::from_wide(remainder)
    } else {
        std::ffi::OsString::from_wide(&path)
    }
}

#[cfg(any(windows, target_os = "macos"))]
fn run_office_command_capture(command: &mut Command, application: &str) -> Result<String, String> {
    let result = command
        .output()
        .map_err(|error| format!("cannot start {application} automation: {error}"))?;
    if !result.status.success() {
        return Err(office_failure(application, &result));
    }
    let version = String::from_utf8_lossy(&result.stdout).trim().to_owned();
    if version.is_empty()
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return Err(format!(
            "{application} did not return a valid version receipt"
        ));
    }
    Ok(version)
}

#[cfg(any(windows, target_os = "macos"))]
fn office_failure(application: &str, result: &std::process::Output) -> String {
    format!("{application} automation failed with {}; check Word installation, licensing and document accessibility", result.status)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use dms_core::{ConfidentialitySnapshot, ExportChrome};
    use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

    use super::*;

    fn chrome() -> ExportChrome {
        ExportChrome {
            version_label: "2.3".to_owned(),
            confidentiality: ConfidentialitySnapshot {
                type_id: "restricted".to_owned(),
                label: "Vertraulich & intern".to_owned(),
            },
            title: "Policy <West>".to_owned(),
            document_number: Some("POL-007".to_owned()),
        }
    }

    fn write_docx(path: &Path) {
        let file = fs::File::create(path).unwrap();
        let mut archive = ZipWriter::new(file);
        for (name, content) in [
            ("[Content_Types].xml", "<Types/>"),
            (
                "word/document.xml",
                "<w:document><w:body><w:p><w:r><w:t>{CONFI</w:t></w:r><w:r><w:t>DENTIALITY}</w:t></w:r><w:r><w:t>{VERSION}</w:t></w:r></w:p></w:body></w:document>",
            ),
            (
                "word/header1.xml",
                "<w:hdr><w:p><w:r><w:t>{CONFIDENTIALITY} · {VERSION}</w:t></w:r></w:p></w:hdr>",
            ),
            (
                "word/footer1.xml",
                "<w:ftr><w:p><w:r><w:t>{VERSION}</w:t></w:r></w:p></w:ftr>",
            ),
            (
                "docProps/custom.xml",
                "<Properties><property>{TITLE}</property><property>{DOCUMENT_NUMBER}</property><property>{VERSION}</property><property>{CONFIDENTIALITY}</property></Properties>",
            ),
        ] {
            archive
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            archive.write_all(content.as_bytes()).unwrap();
        }
        archive.finish().unwrap();
    }

    fn zip_text(bytes: &[u8], name: &str) -> String {
        let mut archive = ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut text = String::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    #[test]
    fn fill_office_placeholders_replaces_tokens_across_parts_without_touching_the_source() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.docx");
        let destination = directory.path().join("filled.docx");
        write_docx(&source);
        let original = fs::read(&source).unwrap();

        fill_office_placeholders(&source, &destination, &chrome()).unwrap();

        assert_eq!(
            fs::read(&source).unwrap(),
            original,
            "source package must stay byte-identical"
        );
        let filled = fs::read(&destination).unwrap();
        for name in [
            "word/document.xml",
            "word/header1.xml",
            "word/footer1.xml",
            "docProps/custom.xml",
        ] {
            let text = zip_text(&filled, name);
            assert!(!text.contains("{CONFIDENTIALITY}"), "{name}");
            assert!(!text.contains("{VERSION}"), "{name}");
            assert!(!text.contains("{TITLE}"), "{name}");
            assert!(!text.contains("{DOCUMENT_NUMBER}"), "{name}");
            if name != "word/footer1.xml" {
                assert!(text.contains("Vertraulich &amp; intern"), "{name}");
            }
            assert!(text.contains("2.3"), "{name}");
            if name == "docProps/custom.xml" {
                assert!(text.contains("Policy &lt;West&gt;"), "{name}");
                assert!(text.contains("POL-007"), "{name}");
            }
        }
    }

    #[test]
    fn office_placeholder_fill_supports_word_document_property_values() {
        let xml = "<Properties><property>{TITLE}</property><property>{DOCUMENT_NUMBER}</property><property>{VERSION}</property><property>{CONFIDENTIALITY}</property></Properties>";

        let filled = String::from_utf8(fill_xml_placeholders(xml, &chrome()).unwrap()).unwrap();

        assert_eq!(
            filled,
            "<Properties><property>Policy &lt;West&gt;</property><property>POL-007</property><property>2.3</property><property>Vertraulich &amp; intern</property></Properties>"
        );
    }

    #[test]
    fn private_staging_does_not_overwrite_predictable_sibling_files() {
        let directory = tempfile::tempdir().unwrap();
        let docx = directory.path().join("policy.docx");
        let sibling = directory.path().join(".policy.docx.dms-export.tmp");
        write_docx(&docx);
        fs::write(&sibling, b"operator file").unwrap();
        fill_in_place(&docx, &chrome()).unwrap();
        assert_eq!(fs::read(sibling).unwrap(), b"operator file");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn reserved_output_placeholder_is_cleared_when_present_and_absent() {
        let directory = tempfile::tempdir().unwrap();
        let reserved = directory.path().join("policy.pdf");
        assert!(clear_reserved_output(&reserved).is_ok(), "missing is fine");
        fs::write(&reserved, b"").unwrap();
        assert!(clear_reserved_output(&reserved).is_ok());
        assert!(!reserved.exists(), "empty placeholder is removed");
        fs::write(&reserved, b"operator PDF").unwrap();
        assert!(clear_reserved_output(&reserved).is_err());
        assert_eq!(fs::read(&reserved).unwrap(), b"operator PDF");
    }

    #[test]
    fn placeholder_values_do_not_recurse_or_override_other_fields() {
        let mut fields = chrome();
        fields.title = "literal {TITLE} and {VERSION}".to_owned();
        fields.confidentiality.label = "literal {DOCUMENT_NUMBER}".to_owned();
        let xml = "<p><t>{TI</t><t>TLE}</t><t>{VERSION}</t><t>{CONFIDENTIALITY}</t></p>";
        let filled = String::from_utf8(fill_xml_placeholders(xml, &fields).unwrap()).unwrap();
        assert_eq!(filled, "<p><t>literal {TITLE} and {VERSION}</t><t></t><t>2.3</t><t>literal {DOCUMENT_NUMBER}</t></p>");
    }

    #[test]
    fn word_double_receives_filled_retained_path_and_adapter_keeps_its_saved_bytes() {
        use std::{cell::RefCell, rc::Rc};
        struct RecordingWord {
            expected: std::path::PathBuf,
            saved: Rc<RefCell<Vec<u8>>>,
        }
        impl WordAutomation for RecordingWord {
            fn preflight(&self) -> Result<(), String> {
                Ok(())
            }
            fn export_pdf(
                &mut self,
                docx: &Path,
                pdf: &Path,
                retain: bool,
            ) -> Result<String, String> {
                assert!(retain);
                assert_eq!(docx, self.expected);
                assert!(!pdf.exists());
                let filled = fs::read(docx).unwrap();
                let properties = zip_text(&filled, "docProps/custom.xml");
                assert!(properties.contains("Policy &lt;West&gt;"));
                assert!(!properties.contains("{TITLE}"));
                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(docx)
                    .unwrap();
                let mut zip = ZipWriter::new_append(file).unwrap();
                zip.start_file("docProps/test-word-save.xml", SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(b"<saved>Updated fields and pagination</saved>")
                    .unwrap();
                zip.finish().unwrap();
                *self.saved.borrow_mut() = fs::read(docx).unwrap();
                // A port double, not a real PDF smoke. Core tests separately
                // require a parsed PDF before recording a successful pair.
                fs::write(pdf, b"%PDF-1.7\nWord test double").unwrap();
                Ok("test Word".to_owned())
            }
        }
        let directory = tempfile::tempdir().unwrap();
        let docx = directory.path().join("retained.docx");
        let pdf = directory.path().join("retained.pdf");
        write_docx(&docx);
        fs::write(&pdf, b"").unwrap();
        let saved = Rc::new(RefCell::new(Vec::new()));
        let mut exporter = InstalledWord::with_automation(RecordingWord {
            expected: docx.clone(),
            saved: saved.clone(),
        });
        assert_eq!(
            exporter.export(&docx, &pdf, &chrome()).unwrap(),
            "test Word"
        );
        assert_eq!(fs::read(docx).unwrap(), *saved.borrow());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn macos_script_binds_only_the_opened_source_before_mutation_or_cleanup() {
        let script = MACOS_WORD_SCRIPT;
        assert!(!script.contains("active document"));
        assert!(script.contains("set sourceFile to (POSIX file sourcePath) as alias"));
        let initialized = script.find("set sourceDocument to missing value").unwrap();
        let opened = script
            .find("set openedDocument to open file name sourcePath")
            .unwrap();
        let missing = script
            .find("if openedDocument is missing value then error")
            .unwrap();
        let validated = script
            .find("if openedFile is not sourceFile then error")
            .unwrap();
        let bound = script.find("set sourceDocument to openedDocument").unwrap();
        assert!(
            initialized < opened && opened < missing && missing < validated && validated < bound
        );
        assert_eq!(
            script
                .matches("set sourceDocument to openedDocument")
                .count(),
            1
        );
        for line in script.lines().map(str::trim) {
            if line.starts_with("set sourceDocument to ") {
                assert!(matches!(
                    line,
                    "set sourceDocument to missing value" | "set sourceDocument to openedDocument"
                ));
            }
            if line.starts_with("save ") || line.starts_with("close ") {
                assert!(
                    line.starts_with("save sourceDocument")
                        || line.starts_with("save as sourceDocument")
                        || line == "close sourceDocument saving no"
                );
            }
        }
        let (success, error) = script
            .split_once("on error errorMessage number errorNumber")
            .unwrap();
        assert!(
            bound
                < success
                    .find("update field every field of sourceDocument")
                    .unwrap()
        );
        assert!(success
            .contains("close sourceDocument saving no\n    set sourceDocument to missing value"));
        assert!(error.contains("if sourceDocument is not missing value then\n      try\n        close sourceDocument saving no\n      end try\n    end if"));
        assert!(!error.contains("save "));
        assert!(!error.contains("openedDocument"));
    }

    #[test]
    fn platform_review_macos_identity_normalizes_posix_and_hfs_before_binding() {
        let script = MACOS_WORD_SCRIPT;
        let opened = script
            .find("set openedDocument to open file name sourcePath")
            .unwrap();
        let bound = script.find("set sourceDocument to openedDocument").unwrap();
        let identity = &script[opened..bound];
        assert!(identity.contains("set openedName to full name of openedDocument as text"));
        assert!(identity.contains(concat!(
            "if openedName starts with \"/\" then\n",
            "      set openedFile to (POSIX file openedName) as alias\n",
            "    else\n",
            "      set openedFile to openedName as alias\n",
            "    end if\n",
            "    if openedFile is not sourceFile then error \"Word returned a different document\"\n"
        )));
        assert!(!identity.contains("close "));
        assert!(!identity.contains("save "));
        assert!(!script.contains("active document"));
        assert!(!script.contains("close openedDocument"));
        assert_eq!(script.matches("close sourceDocument saving no").count(), 2);
    }

    #[test]
    fn platform_review_macos_updates_both_modes_and_saves_only_retained_docx() {
        let script = MACOS_WORD_SCRIPT;
        let (common, retained) = script.split_once("    if retainDocx then\n").unwrap();
        let (retained, export) = retained.split_once("    end if\n").unwrap();
        for update in [
            "update field every field of sourceDocument",
            "repeat with documentSection in sections of sourceDocument",
            "repeat with headerKind in {header footer primary, header footer first page, header footer even pages}",
            "update field every field of text object of storyHeader",
            "update field every field of text object of storyFooter",
            "update documentToc",
        ] {
            assert!(common.contains(update), "update must run in both modes: {update}");
            assert!(!retained.contains(update), "update must not be retained-only: {update}");
        }
        for command in [
            "repaginate sourceDocument",
            "update page numbers documentToc",
            "save sourceDocument",
            "if not saved of sourceDocument then error \"Retained DOCX save was not confirmed\"",
        ] {
            assert!(retained.contains(command), "retained-only: {command}");
            assert!(!common.contains(command));
            assert!(!export.contains(command));
        }
        assert!(
            export.contains("save as sourceDocument file name outputPath file format format PDF")
        );
    }

    #[test]
    fn windows_avoids_global_visibility_and_alert_mutation() {
        let script = WINDOWS_WORD_SCRIPT;
        let snapshot = script.find("$wordPidsBefore = @(Get-Process -Name WINWORD -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)").unwrap();
        let initialized = script.find("$startedWord = $false").unwrap();
        let created = script
            .find("$word = New-Object -ComObject Word.Application")
            .unwrap();
        let observed = script
            .find("$startedWord = @(Get-Process -Name WINWORD -ErrorAction SilentlyContinue | Where-Object { $wordPidsBefore -notcontains $_.Id }).Count -gt 0")
            .unwrap();
        let opened = script.find("$document = $word.Documents.Open(").unwrap();
        assert!(
            snapshot < created && initialized < created && created < observed && observed < opened
        );
        assert!(
            script.contains("cannot prove that this COM object is the newly observed Word process")
        );
        assert!(!script.contains("$word.Visible"));
        assert!(!script.contains("$word.DisplayAlerts"));
    }

    #[test]
    fn retained_cleanup_windows_finally_reaches_owned_quit_after_close_failure() {
        let script = WINDOWS_WORD_SCRIPT;
        let (operation, cleanup) = script.split_once("} finally {").unwrap();
        assert!(operation.contains("$document = $null"));
        assert!(operation.contains(
            "$document = $word.Documents.Open($env:DMS_OFFICE_SOURCE, $false, (-not $retain))"
        ));
        assert!(operation.contains("$document.ExportAsFixedFormat("));
        // Both Word lifecycle methods have a ByRef SaveChanges parameter. The
        // nested finally means a close failure cannot skip the owned-instance
        // cleanup, while an unreadable document count fails safe (no guessed quit).
        assert_eq!(cleanup.trim(), concat!(
            "try {\n",
            "    if ($null -ne $document) { $saveChanges = 0; $document.Close([ref]$saveChanges) }\n",
            "  } finally {\n",
            "    # A Close failure must not skip the owned-instance cleanup. If the document\n",
            "    # count cannot be read, do not guess: keeping an instance is safer than\n",
            "    # quitting one that may have documents opened by the operator.\n",
            "    $documentCount = $null\n",
            "    if ($startedWord -and ($null -ne $word)) {\n",
            "      try { $documentCount = $word.Documents.Count } catch {}\n",
            "      if ($documentCount -eq 0) { $saveChanges = 0; $word.Quit([ref]$saveChanges) }\n",
            "    }\n",
            "  }\n",
            "}"
        ));
        assert_eq!(script.matches(".Close(").count(), 1);
        assert_eq!(script.matches(".Quit(").count(), 1);
        assert_eq!(script.matches("$startedWord =").count(), 2);
        assert!(!script.contains("ActiveDocument"));
    }

    #[test]
    fn windows_owned_instance_cleanup_uses_bindable_byref_arguments() {
        let script = WINDOWS_WORD_SCRIPT;
        // Word's Close and Quit expose SaveChanges as a ByRef parameter. A literal
        // value raises inside `finally`, so both calls use the bindable [ref] form.
        assert!(!script.contains("$document.Close($false)"));
        assert!(!script.contains("$word.Quit(0)"));
        assert!(script.contains("$document.Close([ref]$saveChanges)"));
        assert!(script.contains("$word.Quit([ref]$saveChanges)"));
        assert_eq!(script.matches("$document.Close(").count(), 1);
        assert_eq!(script.matches("$word.Quit(").count(), 1);
    }

    #[test]
    fn platform_review_windows_preflight_does_not_start_or_quit_word() {
        // Include the cfg-disabled Windows function on Linux, not just its caller.
        let source = include_str!("lib.rs");
        let (_, preflight) = source
            .split_once("#[cfg(windows)]\nfn preflight_installed_word()")
            .unwrap();
        let (preflight, _) = preflight
            .split_once("#[cfg(target_os = \"macos\")]")
            .unwrap();
        assert!(preflight.contains("[type]::GetTypeFromProgID('Word.Application')"));
        assert!(!preflight.contains("New-Object"));
        assert!(!preflight.contains("Quit"));
        assert!(!preflight.contains("Visible"));
    }

    // Remove the release-only visibility command; cleanup applies to both modes.
    fn macos_retained_script() -> String {
        MACOS_WORD_SCRIPT
            .lines()
            .filter(|line| {
                !matches!(
                    line.trim(),
                    "if (not retainDocx) and (not wasRunning) then set visible to false"
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn macos_visibility_changes_are_release_only_and_owned() {
        let script = MACOS_WORD_SCRIPT;
        let running = script
            .find("set wasRunning to application \"Microsoft Word\" is running")
            .unwrap();
        let tell = script.find("tell application \"Microsoft Word\"").unwrap();
        let hide = script
            .find("if (not retainDocx) and (not wasRunning) then set visible to false")
            .unwrap();
        let open = script.find("open file name sourcePath").unwrap();
        assert!(running < tell && tell < hide && hide < open);
        assert_eq!(script.matches("set visible").count(), 1);
        let retained = macos_retained_script();
        assert!(!retained.contains("set visible"));
        assert!(!script.contains("display alerts"));
        assert!(!script.contains("activate"));
        let (_, retained_updates) = script.split_once("    if retainDocx then\n").unwrap();
        let (retained_updates, release_tail) = retained_updates.split_once("    end if\n").unwrap();
        assert!(retained_updates.contains("save sourceDocument"));
        assert!(!release_tail.contains("save sourceDocument"));
        assert!(release_tail
            .contains("save as sourceDocument file name outputPath file format format PDF"));
    }

    #[test]
    fn retained_cleanup_macos_success_closes_before_quitting_empty_owned_word() {
        let (success, _) = MACOS_WORD_SCRIPT
            .split_once("on error errorMessage number errorNumber")
            .unwrap();
        let (_, cleanup) = success
            .split_once("save as sourceDocument file name outputPath file format format PDF")
            .unwrap();
        assert_eq!(
            cleanup.trim(),
            concat!(
                "close sourceDocument saving no\n",
                "    set sourceDocument to missing value\n",
                "    try\n",
                "      if (not wasRunning) and (count of documents is 0) then quit\n",
                "    end try\n",
                "    return appVersion"
            )
        );
        assert_eq!(success.matches("then quit").count(), 1);
        assert!(success.contains(
            "try\n      if (not wasRunning) and (count of documents is 0) then quit\n    end try"
        ));
        assert_eq!(MACOS_WORD_SCRIPT.matches("set wasRunning to").count(), 1);
    }

    #[test]
    fn retained_cleanup_macos_error_closes_only_verified_document_then_quits_empty_owned_word() {
        let (_, error) = MACOS_WORD_SCRIPT
            .split_once("on error errorMessage number errorNumber")
            .unwrap();
        assert_eq!(
            error.trim(),
            concat!(
                "if sourceDocument is not missing value then\n",
                "      try\n",
                "        close sourceDocument saving no\n",
                "      end try\n",
                "    end if\n",
                "    try\n",
                "      if (not wasRunning) and (count of documents is 0) then quit\n",
                "    end try\n",
                "    error \"Microsoft Word conversion failed\" number errorNumber\n",
                "  end try\n",
                "end tell"
            )
        );
        assert_eq!(MACOS_WORD_SCRIPT.matches("quit").count(), 2);
    }

    #[test]
    fn native_scripts_save_after_updates_before_export_and_do_not_resave() {
        for (script, update, paginate, save, export) in [
            (
                WINDOWS_WORD_SCRIPT,
                "$range.Fields.Update()",
                "$document.Repaginate()",
                "$document.Save()",
                "$document.ExportAsFixedFormat(",
            ),
            (
                MACOS_WORD_SCRIPT,
                "update field every field of sourceDocument",
                "repaginate sourceDocument",
                "save sourceDocument",
                "save as sourceDocument file name outputPath",
            ),
        ] {
            let update = script.find(update).unwrap();
            let paginate = script.find(paginate).unwrap();
            let save_at = script.find(save).unwrap();
            let export_at = script.find(export).unwrap();
            assert!(update < paginate && paginate < save_at && save_at < export_at);
            assert!(!script[export_at..].contains(save));
        }
        assert!(!macos_retained_script().contains("set visible"));
        assert!(WINDOWS_WORD_SCRIPT.contains("$document.Close([ref]$saveChanges)"));
        assert!(!WINDOWS_WORD_SCRIPT.contains("AutomationSecurity"));
    }

    #[cfg(windows)]
    #[test]
    fn word_automation_receives_win32_paths_instead_of_verbatim_paths() {
        assert_eq!(
            office_compatible_path(Path::new(r"\\?\C:\Users\Operator\release.pdf")),
            std::ffi::OsString::from(r"C:\Users\Operator\release.pdf")
        );
        assert_eq!(
            office_compatible_path(Path::new(r"\\?\UNC\server\share\release.pdf")),
            std::ffi::OsString::from(r"\\server\share\release.pdf")
        );
        assert_eq!(
            office_compatible_path(Path::new(r"C:\Users\Operator\release.pdf")),
            std::ffi::OsString::from(r"C:\Users\Operator\release.pdf")
        );
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    #[test]
    fn unsupported_host_reports_a_clear_error_for_preflight_and_export() {
        assert_eq!(
            installed_word_preflight().unwrap_err(),
            "installed Office export is supported only on Windows and macOS"
        );
        let directory = tempfile::tempdir().unwrap();
        let docx = directory.path().join("policy.docx");
        let pdf = directory.path().join("policy.pdf");
        assert_eq!(
            InstalledWord::new()
                .export(&docx, &pdf, &chrome())
                .unwrap_err(),
            "installed Office export is supported only on Windows and macOS"
        );
    }
}
