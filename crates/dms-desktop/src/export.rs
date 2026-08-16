use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

#[cfg(any(windows, target_os = "macos"))]
use std::process::Command;

use dms_core::{assemble_markdown_docx, ExportChrome, ExportRequest, PdfExporter};
use quick_xml::{events::Event, Reader, Writer};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

pub trait OfficeAutomation {
    fn export_pdf(&mut self, source_copy: &Path, output: &Path) -> Result<(), String>;
}

pub struct LocalPdfExporter<O> {
    office: O,
}

impl<O> LocalPdfExporter<O> {
    pub fn new(office: O) -> Self {
        Self { office }
    }
}

impl<O: OfficeAutomation> PdfExporter for LocalPdfExporter<O> {
    fn export(&mut self, request: &ExportRequest) -> Result<(), String> {
        let extension = request
            .source_path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| "source draft has no supported extension".to_owned())?;
        let result = match extension.as_str() {
            "md" => {
                let markdown = fs::read_to_string(&request.source_path).map_err(|error| {
                    format!(
                        "cannot read Markdown draft {}: {error}",
                        request.source_path.display()
                    )
                })?;
                let template = request.markdown_template_path.as_deref().ok_or_else(|| {
                    "Markdown export requires a validated workspace Word template".to_owned()
                })?;
                let directory = tempfile::tempdir().map_err(|error| {
                    format!("cannot create Markdown export directory: {error}")
                })?;
                let assembled = directory.path().join("assembled.docx");
                let filled = directory.path().join("release.docx");
                assemble_markdown_docx(template, &markdown, &assembled)
                    .map_err(|error| format!("cannot assemble Markdown Word document: {error}"))?;
                fill_office_placeholders(&assembled, &filled, &request.chrome)?;
                self.office
                    .export_pdf(&filled, &request.temporary_pdf_path)
            }
            "docx" => {
                let directory = tempfile::tempdir()
                    .map_err(|error| format!("cannot create Office export directory: {error}"))?;
                let source_copy = directory.path().join("draft.docx");
                fill_office_placeholders(&request.source_path, &source_copy, &request.chrome)?;
                self.office
                    .export_pdf(&source_copy, &request.temporary_pdf_path)
            }
            _ => Err(format!(
                "PDF export is not implemented for .{extension}; supported formats are .md and .docx"
            )),
        };
        result?;
        validate_pdf(&request.temporary_pdf_path)
    }
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
        "{CONFIDENTIALITY}",
        &chrome.confidentiality.label,
    );
    replace_across_text_nodes(&mut text_values, "{VERSION}", &chrome.version_label);
    replace_across_text_nodes(&mut text_values, "{TITLE}", &chrome.title);
    replace_across_text_nodes(
        &mut text_values,
        "{DOCUMENT_NUMBER}",
        chrome.document_number.as_deref().unwrap_or_default(),
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

fn replace_across_text_nodes(values: &mut [String], token: &str, replacement: &str) {
    loop {
        let combined = values.concat();
        let Some(start) = combined.find(token) else {
            return;
        };
        let end = start + token.len();
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

#[derive(Default)]
pub struct InstalledOfficeAutomation;

impl OfficeAutomation for InstalledOfficeAutomation {
    fn export_pdf(&mut self, source_copy: &Path, output: &Path) -> Result<(), String> {
        export_with_installed_office(source_copy, output)
    }
}

#[cfg(windows)]
fn export_with_installed_office(source_copy: &Path, output: &Path) -> Result<(), String> {
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$word = $null
$document = $null
try {
  $word = New-Object -ComObject Word.Application
  $word.Visible = $false
  $word.DisplayAlerts = 0
  $document = $word.Documents.Open($env:DMS_OFFICE_SOURCE, $false, $true)
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
  $document.ExportAsFixedFormat($env:DMS_OFFICE_OUTPUT, 17)
} finally {
  if ($null -ne $document) { $document.Close($false) }
  if ($null -ne $word) { $word.Quit() }
}
"#;
    run_office_command(
        Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
            .env("DMS_OFFICE_SOURCE", office_compatible_path(source_copy))
            .env("DMS_OFFICE_OUTPUT", office_compatible_path(output)),
        "Microsoft Word",
    )
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

#[cfg(target_os = "macos")]
fn export_with_installed_office(source_copy: &Path, output: &Path) -> Result<(), String> {
    const SCRIPT: &str = r#"
set sourcePath to system attribute "DMS_OFFICE_SOURCE"
set outputPath to system attribute "DMS_OFFICE_OUTPUT"
tell application "Microsoft Word"
  set wasRunning to running
  set visible to false
  open file name sourcePath
  set sourceDocument to active document
  save as sourceDocument file name outputPath file format format PDF
  close sourceDocument saving no
  if not wasRunning then quit
end tell
"#;
    run_office_command(
        Command::new("osascript")
            .args(["-e", SCRIPT])
            .env("DMS_OFFICE_SOURCE", source_copy)
            .env("DMS_OFFICE_OUTPUT", output),
        "Microsoft Word",
    )
}

#[cfg(not(any(windows, target_os = "macos")))]
fn export_with_installed_office(_source_copy: &Path, _output: &Path) -> Result<(), String> {
    Err("installed Office export is supported only on Windows and macOS".to_owned())
}

#[cfg(any(windows, target_os = "macos"))]
fn run_office_command(command: &mut Command, application: &str) -> Result<(), String> {
    let result = command
        .output()
        .map_err(|error| format!("cannot start {application} automation: {error}"))?;
    if !result.status.success() {
        let detail = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        return Err(if detail.is_empty() {
            format!("{application} automation failed with {}", result.status)
        } else {
            format!("{application} automation failed: {detail}")
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use dms_core::{ConfidentialitySnapshot, ExportChrome};
    use uuid::Uuid;

    use super::*;

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

    const MARKDOWN_TEMPLATE: &[u8] =
        include_bytes!("../../dms-core/tests/fixtures/markdown-template.docx");

    #[derive(Default)]
    struct FakeOffice {
        copied_packages: Rc<RefCell<Vec<Vec<u8>>>>,
        invalid_output: bool,
    }

    impl OfficeAutomation for FakeOffice {
        fn export_pdf(&mut self, source_copy: &Path, output: &Path) -> Result<(), String> {
            self.copied_packages
                .borrow_mut()
                .push(fs::read(source_copy).map_err(|error| error.to_string())?);
            let bytes: &[u8] = if self.invalid_output {
                b"not a PDF"
            } else {
                b"%PDF-1.7\nOffice fake"
            };
            fs::write(output, bytes).map_err(|error| error.to_string())
        }
    }

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

    fn request(
        source_path: PathBuf,
        output: PathBuf,
        markdown_template_path: Option<PathBuf>,
    ) -> ExportRequest {
        ExportRequest {
            document_id: Uuid::new_v4(),
            source_path,
            markdown_template_path,
            temporary_pdf_path: output.clone(),
            final_pdf_path: output.with_file_name("final.pdf"),
            chrome: chrome(),
        }
    }

    fn write_docx(path: &Path) {
        let file = fs::File::create(path).unwrap();
        let mut archive = ZipWriter::new(file);
        for (name, content) in [
            ("[Content_Types].xml", "<Types/>") ,
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
    fn markdown_assembles_fills_and_dispatches_a_template_backed_docx_to_office() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.md");
        let template = directory.path().join("template.docx");
        let output = directory.path().join("release.tmp");
        fs::write(
            &source,
            "---\ntitle: source title\ndocument_number: SOURCE-1\nversion: 2.3\nconfidentiality: Vertraulich & intern\nauthor: Source Author\n---\n# Body\n\nVisible text.",
        )
        .unwrap();
        fs::write(&template, MARKDOWN_TEMPLATE).unwrap();
        let source_before = fs::read(&source).unwrap();
        let template_before = fs::read(&template).unwrap();
        let packages = Rc::new(RefCell::new(Vec::new()));
        let office = FakeOffice {
            copied_packages: packages.clone(),
            invalid_output: false,
        };
        let mut exporter = LocalPdfExporter::new(office);

        exporter
            .export(&request(
                source.clone(),
                output.clone(),
                Some(template.clone()),
            ))
            .unwrap();

        assert!(fs::read(&output).unwrap().starts_with(b"%PDF"));
        assert_eq!(fs::read(&source).unwrap(), source_before);
        assert_eq!(fs::read(&template).unwrap(), template_before);
        let packages = packages.borrow();
        assert_eq!(packages.len(), 1);
        let document = zip_text(&packages[0], "word/document.xml");
        assert!(document.contains("Body"));
        assert!(document.contains("Visible text."));
        assert!(!document.contains("source title"));
        assert!(!document.contains("Source Author"));
        assert!(!document.contains("---"));
        assert!(!document.contains("{Heading 1}"));
        let custom = zip_text(&packages[0], "docProps/custom.xml");
        assert!(custom.contains("Policy &lt;West&gt;"));
        assert!(custom.contains("POL-007"));
        assert!(custom.contains("2.3"));
        assert!(custom.contains("Vertraulich &amp; intern"));
        assert_eq!(
            zip_text(&packages[0], "word/styles.xml"),
            zip_text(&template_before, "word/styles.xml")
        );
    }

    #[test]
    fn markdown_export_refuses_a_missing_template_request() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.md");
        fs::write(
            &source,
            "---\nversion: 2.3\nconfidentiality: Vertraulich & intern\n---\n# Policy",
        )
        .unwrap();
        let packages = Rc::new(RefCell::new(Vec::new()));
        let mut exporter = LocalPdfExporter::new(FakeOffice {
            copied_packages: packages.clone(),
            invalid_output: false,
        });

        let error = exporter
            .export(&request(source, directory.path().join("release.tmp"), None))
            .unwrap_err();

        assert!(error.contains("requires a validated workspace Word template"));
        assert!(packages.borrow().is_empty());
    }

    #[test]
    fn docx_export_replaces_body_header_and_footer_tokens_on_a_copy() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.docx");
        let output = directory.path().join("release.tmp");
        write_docx(&source);
        let original = fs::read(&source).unwrap();
        let packages = Rc::new(RefCell::new(Vec::new()));
        let office = FakeOffice {
            copied_packages: packages.clone(),
            invalid_output: false,
        };
        let mut exporter = LocalPdfExporter::new(office);

        exporter
            .export(&request(source.clone(), output, None))
            .unwrap();

        assert_eq!(fs::read(&source).unwrap(), original);
        let packages = packages.borrow();
        assert_eq!(packages.len(), 1);
        for name in [
            "word/document.xml",
            "word/header1.xml",
            "word/footer1.xml",
            "docProps/custom.xml",
        ] {
            let text = zip_text(&packages[0], name);
            assert!(!text.contains("{CONFIDENTIALITY}"));
            assert!(!text.contains("{VERSION}"));
            assert!(!text.contains("{TITLE}"));
            assert!(!text.contains("{DOCUMENT_NUMBER}"));
            if name != "word/footer1.xml" {
                assert!(text.contains("Vertraulich &amp; intern"));
            }
            assert!(text.contains("2.3"));
            if name == "docProps/custom.xml" {
                assert!(text.contains("Policy &lt;West&gt;"));
                assert!(text.contains("POL-007"));
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
    fn unsupported_source_format_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.xlsx");
        fs::write(&source, b"not used").unwrap();
        let mut exporter = LocalPdfExporter::new(FakeOffice::default());

        let error = exporter
            .export(&request(source, directory.path().join("release.tmp"), None))
            .unwrap_err();

        assert!(error.contains("not implemented for .xlsx"));
        assert!(error.contains(".md and .docx"));
    }

    #[test]
    fn malformed_adapter_output_fails_before_core_can_commit_it() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.md");
        let template = directory.path().join("template.docx");
        fs::write(
            &source,
            "---\nversion: 2.3\nconfidentiality: Vertraulich & intern\n---\n# Policy",
        )
        .unwrap();
        fs::write(&template, MARKDOWN_TEMPLATE).unwrap();
        let mut exporter = LocalPdfExporter::new(FakeOffice {
            invalid_output: true,
            ..FakeOffice::default()
        });

        let error = exporter
            .export(&request(
                source,
                directory.path().join("release.tmp"),
                Some(template),
            ))
            .unwrap_err();

        assert!(error.contains("did not produce a valid PDF"));
    }
}
