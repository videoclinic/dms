use std::{fs, path::Path};

use dms_core::{assemble_markdown_docx, ExportRequest, PdfExporter};

pub use dms_export::fill_office_placeholders;

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
                assemble_markdown_docx(
                    template,
                    &markdown,
                    &assembled,
                    request.markdown_assets.as_ref(),
                )
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

#[derive(Default)]
pub struct InstalledOfficeAutomation;

impl OfficeAutomation for InstalledOfficeAutomation {
    fn export_pdf(&mut self, source_copy: &Path, output: &Path) -> Result<(), String> {
        dms_export::export_installed_office_pdf(source_copy, output)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        io::{Read, Write},
        path::PathBuf,
        rc::Rc,
    };

    use dms_core::{ConfidentialitySnapshot, ExportChrome};
    use uuid::Uuid;
    use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

    use super::*;

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
            markdown_assets: None,
            temporary_pdf_path: output.clone(),
            final_pdf_path: output.with_file_name("final.pdf"),
            chrome: chrome(),
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
