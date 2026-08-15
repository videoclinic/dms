use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::mpsc,
    time::Duration,
};

#[cfg(any(windows, target_os = "macos"))]
use std::process::Command;

use dms_core::{ExportChrome, ExportRequest, PdfExporter};
use pulldown_cmark::{html, Options, Parser};
use quick_xml::{events::Event, Reader, Writer};
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tempfile::TempDir;
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

const PRINT_SHELL: &str = include_str!("../ui/print/shell.html");
const PRINT_CSS: &str = include_str!("../ui/print/print.css");
const PRINT_LOGO: &[u8] = include_bytes!("../ui/print/logo.svg");
const WEBVIEW_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintDocument {
    pub html: String,
    pub css: &'static str,
    pub logo: &'static [u8],
}

pub trait OfficeAutomation {
    fn export_pdf(&mut self, source_copy: &Path, output: &Path) -> Result<(), String>;
}

pub trait WebviewPdfPrinter {
    fn print_pdf(&mut self, document: &PrintDocument, output: &Path) -> Result<(), String>;
}

pub struct LocalPdfExporter<O, P> {
    office: O,
    markdown: P,
}

impl<O, P> LocalPdfExporter<O, P> {
    pub fn new(office: O, markdown: P) -> Self {
        Self { office, markdown }
    }
}

impl<O: OfficeAutomation, P: WebviewPdfPrinter> PdfExporter for LocalPdfExporter<O, P> {
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
                let document = render_markdown(&markdown, &request.chrome);
                self.markdown
                    .print_pdf(&document, &request.temporary_pdf_path)
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

pub fn render_markdown(markdown: &str, chrome: &ExportChrome) -> PrintDocument {
    let body = strip_yaml_front_matter(markdown);
    let parser = Parser::new_ext(body, Options::all());
    let mut body_html = String::new();
    html::push_html(&mut body_html, parser);

    let document_number = chrome.document_number.as_deref().unwrap_or("");
    let html = PRINT_SHELL
        .replace("{{TITLE}}", &escape_html(&chrome.title))
        .replace("{{DOCUMENT_NUMBER}}", &escape_html(document_number))
        .replace(
            "{{CONFIDENTIALITY}}",
            &escape_html(&chrome.confidentiality.label),
        )
        .replace("{{VERSION}}", &escape_html(&chrome.version_label))
        .replace("{{BODY}}", &body_html);
    PrintDocument {
        html,
        css: PRINT_CSS,
        logo: PRINT_LOGO,
    }
}

fn strip_yaml_front_matter(markdown: &str) -> &str {
    let normalized = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let Some(rest) = normalized
        .strip_prefix("---\n")
        .or_else(|| normalized.strip_prefix("---\r\n"))
    else {
        return normalized;
    };
    let mut consumed = 0;
    for line in rest.split_inclusive('\n') {
        consumed += line.len();
        if matches!(line.trim_end_matches(['\r', '\n']), "---" | "...") {
            return &rest[consumed..];
        }
    }
    normalized
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
  $document.ExportAsFixedFormat($env:DMS_OFFICE_OUTPUT, 17)
} finally {
  if ($null -ne $document) { $document.Close($false) }
  if ($null -ne $word) { $word.Quit() }
}
"#;
    run_office_command(
        Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
            .env("DMS_OFFICE_SOURCE", source_copy)
            .env("DMS_OFFICE_OUTPUT", output),
        "Microsoft Word",
    )
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

#[derive(Clone)]
pub struct NativeWebviewPdfPrinter {
    app: AppHandle,
}

impl NativeWebviewPdfPrinter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl WebviewPdfPrinter for NativeWebviewPdfPrinter {
    fn print_pdf(&mut self, document: &PrintDocument, output: &Path) -> Result<(), String> {
        let directory = materialize_print_document(document)?;
        let html_path = directory.path().join("document.html");
        let url = tauri::Url::from_file_path(&html_path)
            .map_err(|_| format!("cannot create file URL for {}", html_path.display()))?;
        let label = format!("markdown-export-{}", Uuid::new_v4());
        let (loaded_tx, loaded_rx) = mpsc::sync_channel(1);
        let window = WebviewWindowBuilder::new(&self.app, label, WebviewUrl::External(url))
            .visible(false)
            .on_page_load(move |window, payload| {
                if payload.event() == tauri::webview::PageLoadEvent::Finished {
                    let _ = loaded_tx.try_send(window);
                }
            })
            .build()
            .map_err(|error| format!("cannot create Markdown export WebView: {error}"))?;
        let loaded = loaded_rx
            .recv_timeout(WEBVIEW_TIMEOUT)
            .map_err(|_| "Markdown export WebView did not finish loading".to_owned())?;
        let result = print_loaded_webview(&loaded, output);
        let _ = window.close();
        result
    }
}

pub fn platform_pdf_smoke(app: AppHandle) -> Result<(), String> {
    let chrome = ExportChrome {
        version_label: "3.2".to_owned(),
        confidentiality: dms_core::ConfidentialitySnapshot {
            type_id: "restricted".to_owned(),
            label: "Vertraulich".to_owned(),
        },
        title: "PDF export smoke".to_owned(),
        document_number: Some("SMOKE-001".to_owned()),
    };
    let markdown = [
        "# First page\n\nNative WebView export smoke.",
        "<div style=\"break-before: page\"></div>\n\n# Second page\n\nFooter repetition.",
        "<div style=\"break-before: page\"></div>\n\n# Third page\n\nFinal page.",
    ]
    .join("\n\n");
    let document = render_markdown(&markdown, &chrome);
    let directory = tempfile::tempdir()
        .map_err(|error| format!("cannot create PDF smoke directory: {error}"))?;
    let output = directory.path().join("smoke.pdf");
    NativeWebviewPdfPrinter::new(app).print_pdf(&document, &output)?;
    let bytes = fs::read(&output)
        .map_err(|error| format!("cannot read PDF smoke output {}: {error}", output.display()))?;
    if bytes.len() <= 4 || &bytes[..4] != b"%PDF" {
        return Err("native WebView output is not a non-empty PDF".to_owned());
    }
    Ok(())
}

fn materialize_print_document(document: &PrintDocument) -> Result<TempDir, String> {
    let directory = tempfile::tempdir()
        .map_err(|error| format!("cannot create Markdown print directory: {error}"))?;
    fs::write(directory.path().join("document.html"), &document.html)
        .map_err(|error| format!("cannot write Markdown print document: {error}"))?;
    fs::write(directory.path().join("print.css"), document.css)
        .map_err(|error| format!("cannot write Markdown print stylesheet: {error}"))?;
    fs::write(directory.path().join("logo.svg"), document.logo)
        .map_err(|error| format!("cannot write Markdown print logo: {error}"))?;
    Ok(directory)
}

#[cfg(windows)]
fn print_loaded_webview(window: &WebviewWindow, output: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use webview2_com::{
        Microsoft::Web::WebView2::Win32::ICoreWebView2_7, PrintToPdfCompletedHandler,
    };
    use windows::core::{Error as WindowsError, Interface, HRESULT, PCWSTR};

    let output_wide = output
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    window
        .with_webview(move |platform| {
            let result = (|| {
                let controller = platform.controller();
                let core =
                    unsafe { controller.CoreWebView2() }.map_err(|error| error.to_string())?;
                let pdf: ICoreWebView2_7 = core.cast().map_err(|error| error.to_string())?;
                PrintToPdfCompletedHandler::wait_for_async_operation(
                    Box::new(move |handler| unsafe {
                        pdf.PrintToPdf(PCWSTR(output_wide.as_ptr()), None, &handler)?;
                        Ok(())
                    }),
                    Box::new(|status, success| {
                        status?;
                        if !success {
                            return Err(WindowsError::new(
                                HRESULT(0x80004005u32 as i32),
                                "WebView2 did not produce a PDF",
                            ));
                        }
                        Ok(())
                    }),
                )
                .map_err(|error| error.to_string())
            })();
            let _ = result_tx.send(result);
        })
        .map_err(|error| format!("cannot access WebView2: {error}"))?;
    result_rx
        .recv_timeout(WEBVIEW_TIMEOUT)
        .map_err(|_| "WebView2 PDF export timed out".to_owned())?
}

#[cfg(target_os = "macos")]
fn print_loaded_webview(window: &WebviewWindow, output: &Path) -> Result<(), String> {
    use block2::RcBlock;
    use objc2_foundation::{NSData, NSError};
    use objc2_web_kit::WKWebView;

    let output = output.to_path_buf();
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    window
        .with_webview(move |platform| {
            let raw_webview = platform.inner();
            if raw_webview.is_null() {
                let _ = result_tx.send(Err("WKWebView handle is null".to_owned()));
                return;
            }
            let webview: &WKWebView = unsafe { &*raw_webview.cast() };
            let completion = RcBlock::new(move |data: *mut NSData, error: *mut NSError| {
                let result = if !error.is_null() {
                    Err(format!("WKWebView PDF export failed: {:?}", unsafe {
                        &*error
                    }))
                } else if data.is_null() {
                    Err("WKWebView PDF export returned no data".to_owned())
                } else {
                    let bytes = unsafe { (&*data).to_vec() };
                    fs::write(&output, bytes)
                        .map_err(|error| format!("cannot write WKWebView PDF: {error}"))
                };
                let _ = result_tx.send(result);
            });
            unsafe {
                webview.createPDFWithConfiguration_completionHandler(None, &completion);
            }
        })
        .map_err(|error| format!("cannot access WKWebView: {error}"))?;
    result_rx
        .recv_timeout(WEBVIEW_TIMEOUT)
        .map_err(|_| "WKWebView PDF export timed out".to_owned())?
}

#[cfg(not(any(windows, target_os = "macos")))]
fn print_loaded_webview(_window: &WebviewWindow, _output: &Path) -> Result<(), String> {
    Err("native WebView PDF export is supported only on Windows and macOS".to_owned())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use dms_core::{ConfidentialitySnapshot, ExportChrome};

    use super::*;

    #[derive(Default)]
    struct FakeOffice {
        copied_packages: Rc<RefCell<Vec<Vec<u8>>>>,
    }

    impl OfficeAutomation for FakeOffice {
        fn export_pdf(&mut self, source_copy: &Path, output: &Path) -> Result<(), String> {
            self.copied_packages
                .borrow_mut()
                .push(fs::read(source_copy).map_err(|error| error.to_string())?);
            fs::write(output, b"%PDF-1.7\nOffice fake").map_err(|error| error.to_string())
        }
    }

    #[derive(Default)]
    struct FakePrinter {
        documents: Rc<RefCell<Vec<PrintDocument>>>,
        invalid_output: bool,
    }

    impl WebviewPdfPrinter for FakePrinter {
        fn print_pdf(&mut self, document: &PrintDocument, output: &Path) -> Result<(), String> {
            self.documents.borrow_mut().push(document.clone());
            let bytes: &[u8] = if self.invalid_output {
                b"not a PDF"
            } else {
                b"%PDF-1.7\nWebView fake"
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

    fn request(source_path: PathBuf, output: PathBuf) -> ExportRequest {
        ExportRequest {
            document_id: Uuid::new_v4(),
            source_path,
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
    fn markdown_print_shell_strips_front_matter_and_uses_only_release_chrome() {
        let document = render_markdown(
            "---\ntitle: ignored\nconfidentiality: public\n---\n# Body\n\nVisible text.",
            &chrome(),
        );

        assert!(!document.html.contains("title: ignored"));
        assert!(!document.html.contains("confidentiality: public"));
        assert!(document.html.contains("<h1>Body</h1>"));
        assert!(document.html.contains("Policy &lt;West&gt;"));
        assert!(document
            .html
            .contains("Vertraulichkeitsstufe: Vertraulich &amp; intern"));
        assert!(document.html.contains("Version: 2.3"));
        assert!(document.html.contains("POL-007"));
        assert!(document.css.contains("@page"));
        assert!(document.css.contains("position: fixed"));
        assert!(document.css.contains("counter(page)"));
        assert_eq!(document.logo, PRINT_LOGO);
    }

    #[test]
    fn markdown_dispatches_to_webview_printer_at_the_requested_temporary_path() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.md");
        let output = directory.path().join("release.tmp");
        fs::write(&source, "# Policy").unwrap();
        let documents = Rc::new(RefCell::new(Vec::new()));
        let printer = FakePrinter {
            documents: documents.clone(),
            invalid_output: false,
        };
        let mut exporter = LocalPdfExporter::new(FakeOffice::default(), printer);

        exporter.export(&request(source, output.clone())).unwrap();

        assert!(fs::read(&output).unwrap().starts_with(b"%PDF"));
        assert_eq!(documents.borrow().len(), 1);
        assert!(documents.borrow()[0].html.contains("Version: 2.3"));
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
        };
        let mut exporter = LocalPdfExporter::new(office, FakePrinter::default());

        exporter.export(&request(source.clone(), output)).unwrap();

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
        let mut exporter = LocalPdfExporter::new(FakeOffice::default(), FakePrinter::default());

        let error = exporter
            .export(&request(source, directory.path().join("release.tmp")))
            .unwrap_err();

        assert!(error.contains("not implemented for .xlsx"));
        assert!(error.contains(".md and .docx"));
    }

    #[test]
    fn malformed_adapter_output_fails_before_core_can_commit_it() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.md");
        fs::write(&source, "# Policy").unwrap();
        let mut exporter = LocalPdfExporter::new(
            FakeOffice::default(),
            FakePrinter {
                documents: Rc::default(),
                invalid_output: true,
            },
        );

        let error = exporter
            .export(&request(source, directory.path().join("release.tmp")))
            .unwrap_err();

        assert!(error.contains("did not produce a valid PDF"));
    }
}
