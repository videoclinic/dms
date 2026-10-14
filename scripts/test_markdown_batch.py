"""Synthetic-only independent verifier regressions; no Word or real workspace.

uv run --no-project --with markdown-it-py==4.0.0 --with mdit-py-plugins==0.5.0 --with PyMuPDF==1.26.5 \
    python -m unittest discover -s scripts -p 'test_markdown_batch.py' -v
"""
import argparse
import base64
import contextlib
import copy
import io
import json
import re
import subprocess
import tempfile
import unittest
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path
from unittest.mock import patch

import pymupdf
import verify_markdown_batch as v

PNG = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=")
SOURCE = """---
title: Synthetic policy
version: 1.0
confidentiality: internal
---
# Synthetic heading
Synthetic verification text.
![one](assets/image.png) ![two][picture]

[picture]: assets/image.png

## Versionsverlauf
| Version | Date | Author | Change | Review |
| --- | --- | --- | --- | --- |
| 1.0 | 2026 | Person | Initial | Done |
"""
META = {"title": "Synthetic policy", "document_number": None, "version": "1.0",
        "confidentiality": {"type_id": "internal", "label": "Internal"}}


def paragraph(text):
    return f"<w:p><w:r><w:t>{text}</w:t></w:r></w:p>"


def docx(path, template=False, image_count=2, wrong_cell=False, numbered=False):
    body = paragraph("Synthetic heading") + paragraph("Synthetic verification text.")
    body += paragraph("Versionsverlauf")
    body += '<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/><w:tblW w:type="dxa" w:w="9000"/></w:tblPr><w:tblGrid>'
    body += '<w:gridCol w:w="1800"/>' * 5 + '</w:tblGrid>'
    for row in (["Version", "Date", "Author", "Change", "Review"], ["1.0", "2026", "Person", "WRONG" if wrong_cell else "Initial", "Done"]):
        body += '<w:tr>' + ''.join('<w:tc>' + paragraph(c) + '</w:tc>' for c in row) + '</w:tr>'
    body += '</w:tbl>'
    body += '<w:sdt><w:sdtPr><w:docPartObj><w:docPartGallery w:val="Table of Contents"/></w:docPartObj></w:sdtPr><w:sdtContent>'
    body += '<w:p><w:r><w:instrText> TOC </w:instrText><w:t>Synthetic heading 1</w:t></w:r></w:p></w:sdtContent></w:sdt>'
    for _ in range(0 if template else image_count):
        body += '<w:p><w:r><w:drawing><wp:inline><wp:extent cx="500000" cy="500000"/><a:blip r:embed="source"/></wp:inline></w:drawing></w:r></w:p>'
    body += '<w:sectPr><w:footerReference w:type="default" r:id="footer"/><w:pgSz w:w="11000"/><w:pgMar w:left="1000" w:right="1000"/></w:sectPr>'
    if numbered:
        body = body.replace('<w:p><w:r><w:t>Synthetic heading', '<w:p><w:pPr><w:numPr><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>1. Synthetic heading')
    prefix = f'xmlns:w="{v.W}" xmlns:r="{v.R}" xmlns:a="{v.A}" xmlns:wp="{v.WP}"'
    rel = f'<Relationships xmlns="{v.REL}">'
    parts = {
        "[Content_Types].xml": f'<Types xmlns="{v.CT}"><Default Extension="xml" ContentType="application/xml"/>'
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="png" ContentType="image/png"/><Override PartName="/word/document.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>',
        "_rels/.rels": rel + '<Relationship Id="doc" Type="officeDocument" Target="word/document.xml"/></Relationships>',
        "word/document.xml": f'<w:document {prefix}><w:body>{body}</w:body></w:document>',
        "word/_rels/document.xml.rels": rel + '<Relationship Id="footer" Type="footer" Target="footer1.xml"/>' + ('<Relationship Id="source" Type="image" Target="media/source.png"/>' if not template else '') + '</Relationships>',
        "word/header1.xml": f'<w:hdr {prefix}><w:p><w:r><w:drawing><a:blip r:embed="logo"/></w:drawing></w:r></w:p>{paragraph("Static header")}</w:hdr>',
        "word/footer1.xml": f'<w:ftr {prefix}>{paragraph("{VERSION} {CONFIDENTIALITY}" if template else "1.0 Internal")}</w:ftr>',
        "word/_rels/header1.xml.rels": rel + '<Relationship Id="logo" Type="image" Target="media/logo.png"/></Relationships>',
        "word/media/logo.png": PNG,
        "docProps/custom.xml": '<Properties>' + ''.join(f'<property name="{key}"><value>{value}</value></property>' for key, value in
            {"DMS_TITLE": "Synthetic policy", "DMS_VERSION": "1.0", "DMS_DOCUMENT_NUMBER": "", "DMS_CONFIDENTIALITY": "Internal"}.items()) + '</Properties>',
    }
    if not template:
        parts["word/media/source.png"] = PNG
    with zipfile.ZipFile(path, "w") as archive:
        for key, value in parts.items():
            archive.writestr(key, value)


def pdf(path, omit=""):
    with pymupdf.open() as document:
        page = document.new_page()
        lines = ["Synthetic policy", "Synthetic heading", "Synthetic verification text.", "Versionsverlauf",
                 "Version Date Author Change Review", "1.0 2026 Person Initial Done"]
        page.insert_text((50, 70), "\n".join(line for line in lines if line != omit))
        page.insert_text((50, 800), "1.0 Internal")
        document.save(path)


class VerifierTests(unittest.TestCase):
    def setUp(self):
        scratch = Path(__file__).resolve().parents[1] / "target" / "smoke-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=scratch)
        self.root = Path(self.temp.name).resolve()
        self.source = self.root / "input"
        self.source.mkdir()
        (self.source / "assets").mkdir()
        (self.source / "assets/image.png").write_bytes(PNG)
        (self.source / "policy.md").write_text(SOURCE, encoding="utf-8")
        self.inventory = v.source_inventory(self.source / "policy.md", self.source)
        self.template = self.root / "template.docx"
        docx(self.template, template=True)
        self.doc = self.root / "final.docx"
        docx(self.doc)
        self.config = {"schema_version": 19, "markdown_template": {"id": "abcdef01-0000-0000-0000-000000000001", "relative_path": "template.docx",
                       "sha256": v.file_sha(self.template), "contract_version": 1},
                       "confidentiality_types": {"internal": {"id": "internal", "label": "Internal", "enabled": True}}}

    def tearDown(self):
        self.temp.cleanup()

    def check_doc(self):
        return v.check_docx(v.package(self.doc), v.package(self.template), self.inventory, META)

    def manifest(self):
        return {"conversion_only": True, "dry_run": False, "without_toc": False,
                "renderer_version": "dms-core/0.1.0", "configuration_schema": 19,
                "template": self.config["markdown_template"], "results": [{
                    "input": "policy.md", "input_sha256": self.inventory["sha256"],
                    "assets": [{"relative_path": "assets/image.png", "sha256": v.sha(PNG)}],
                    "metadata": META, "docx": None, "pdf": None, "word_version": "16.0",
                    "status": "succeeded", "error": None, "retained_diagnostics": []}]}

    def run_fixture(self, mutate=None):
        output = self.root / "output"
        output.mkdir()
        (output / "policy.docx").write_bytes(self.doc.read_bytes())
        pdf(output / "policy.pdf")
        manifest = self.manifest()
        for kind in ("docx", "pdf"):
            manifest["results"][0][kind] = {"relative_path": "policy." + kind, "sha256": v.file_sha(output / ("policy." + kind))}
        if mutate:
            mutate(manifest)
        (output / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
        renders = self.root / "renders"
        renders.mkdir()
        process = subprocess.CompletedProcess([], 0, json.dumps(manifest).encode(), b"private diagnostic MUST NOT LEAK")
        return v.check_run(output, process, ["policy.md"], {"policy.md": self.inventory}, self.config,
                           v.package(self.template), renders)

    def test_repeated_reference_images_not_template_logo(self):
        result = self.check_doc()
        self.assertEqual((result["source_references"], result["source_drawings"], result["template_drawings"]), (2, 2, 1))
        self.assertEqual(result["history_columns"], 5)

    def test_logo_cannot_cover_missing_source_drawing(self):
        docx(self.doc, image_count=1)
        with self.assertRaisesRegex(v.CheckError, "drawing_accounting"):
            self.check_doc()

    def test_table_text_and_column_order_are_checked(self):
        docx(self.doc, wrong_cell=True)
        with self.assertRaises(v.CheckError):
            self.check_doc()

    def test_duplicate_source_and_word_numbering_rejected(self):
        docx(self.doc, numbered=True)
        self.inventory["headings"][0] = (1, "1. Synthetic heading")
        with self.assertRaisesRegex(v.CheckError, "duplicate_heading_numbering"):
            self.check_doc()

    def test_source_history_required(self):
        path = self.source / "policy.md"
        path.write_text(SOURCE.replace("Versionsverlauf", "Other"), encoding="utf-8")
        with self.assertRaisesRegex(v.CheckError, "versionsverlauf"):
            v.source_inventory(path, self.source)

    def test_fenced_image_syntax_not_counted(self):
        path = self.source / "policy.md"
        path.write_text(SOURCE + '\n```\n![not an image](missing.png)\n```\n', encoding="utf-8")
        self.assertEqual(len(v.source_inventory(path, self.source)["images"]), 2)

    def test_smart_punctuation_matches_actual_assembler_text_without_hiding_loss(self):
        source = 'Speed -- and power --- and range... says "fast" and don\'t stop.'
        rendered = 'Speed – and power — and range… says “fast” and don’t stop.'
        self.assertEqual(v.fidelity_text(source), v.fidelity_text(rendered))
        self.assertNotEqual(v.fidelity_text(source), v.fidelity_text(rendered.replace("power", "")))

    def test_extension_inventory_matches_actual_assembler_visible_text(self):
        path = self.source / "policy.md"
        path.write_text(SOURCE + '\n# Documents {#docs .wide}\n\n'
                        'Struck ~~struck out~~ text and [the portal](https://example.test) link.\n\n'
                        '- [X] completed task\n- [ ] pending task\n\n'
                        'Inline math $x^2$ and a footnote [^n].\n\n[^n]: Footnote body.\n', encoding="utf-8")
        inv = v.source_inventory(path, self.source)
        self.assertIn((1, "Documents"), inv["headings"])
        for text in ('Struck struck out text and the portal (https://example.test) link.',
                     '[x] completed task', '[ ] pending task',
                     'Inline math $x^2$ and a footnote [^n].', 'Footnote body.'):
            self.assertIn(text, inv["chunks"])

    def test_smart_punctuation_in_docx_body_and_table(self):
        path = self.source / "policy.md"
        path.write_text(SOURCE.replace("Synthetic verification text.", 'Speed -- says "fast"...')
                        .replace("Initial", "Initial -- revised"), encoding="utf-8")
        self.inventory = v.source_inventory(path, self.source)
        doc = v.package(self.doc)
        for node in doc["xml"]["word/document.xml"].findall(".//w:t", v.NS):
            if node.text == "Synthetic verification text.":
                node.text = 'Speed – says “fast”…'
            if node.text == "Initial":
                node.text = "Initial – revised"
        v.check_docx(doc, v.package(self.template), self.inventory, META)

    def test_pdf_smart_punctuation_matches_but_text_loss_fails(self):
        path = self.root / "punctuation.pdf"
        with pymupdf.open() as document:
            page = document.new_page()
            page.insert_text((50, 70), "Synthetic policy")
            page.insert_htmlbox(pymupdf.Rect(50, 100, 500, 200), '<p>Speed – says “fast”…</p>')
            page.insert_text((50, 800), "1.0 Internal")
            document.save(path)
        inv = {"chunks": ['Speed -- says "fast"...']}
        v.check_pdf(path, inv, META, self.root, "file-001")
        inv["chunks"].append("Missing content must still fail.")
        with self.assertRaisesRegex(v.CheckError, "pdf_source_text_missing"):
            v.check_pdf(path, inv, META, self.root, "file-002")

    def test_link_target_in_table_and_extension_literals_in_code(self):
        path = self.source / "policy.md"
        path.write_text(SOURCE.replace("Initial", "[Initial](https://example.test)")
                        + '\n```text\n# Literal {#id}\n~~literal~~ [X] literal\n```\n', encoding="utf-8")
        inv = v.source_inventory(path, self.source)
        self.assertEqual(inv["history"][1][3], "Initial (https://example.test)")
        self.assertIn("# Literal {#id}\n~~literal~~ [X] literal\n", inv["chunks"])

    def test_source_asset_escape_rejected(self):
        (self.root / "outside.png").write_bytes(PNG)
        path = self.source / "policy.md"
        path.write_text(SOURCE.replace("assets/image.png", "../outside.png"), encoding="utf-8")
        with self.assertRaisesRegex(v.CheckError, "escape"):
            v.source_inventory(path, self.source)

    def test_manifest_selection_duplicate_and_release_flags(self):
        for mutation in (lambda m: m.update(conversion_only=False), lambda m: m.update(released=True),
                         lambda m: m["results"].append(copy.deepcopy(m["results"][0]))):
            value = self.manifest()
            mutation(value)
            with self.assertRaises(v.CheckError):
                v.manifest_check(value, ["policy.md"], self.config)

    def test_full_synthetic_pair(self):
        result = self.run_fixture()
        self.assertEqual(result["verified"], 1, result)
        self.assertTrue(result["no_unexpected_artifacts"])
        self.assertEqual(result["files"][0]["pdf_pages"], 1)
        self.assertNotIn("Synthetic", json.dumps(result))
        self.assertNotIn("private diagnostic", json.dumps(result))

    def test_manifest_template_uuid_spelling_is_not_identity(self):
        manifest = self.manifest()
        manifest["template"] = dict(manifest["template"])
        self.config["markdown_template"]["id"] = self.config["markdown_template"]["id"].upper()
        v.manifest_check(manifest, ["policy.md"], self.config)

    def test_manifest_template_contract_fields_still_checked(self):
        for key, value in (("id", "00000000-0000-0000-0000-000000000001"),
                           ("relative_path", "other.docx"), ("sha256", "0" * 64), ("contract_version", 2)):
            manifest = copy.deepcopy(self.manifest())
            manifest["template"][key] = value
            with self.assertRaisesRegex(v.CheckError, "template_mismatch"):
                v.manifest_check(manifest, ["policy.md"], self.config)

    def test_repeated_manifest_asset_is_rejected_not_counted_as_occurrence(self):
        result = self.run_fixture(lambda m: m["results"][0]["assets"].append(dict(m["results"][0]["assets"][0])))
        self.assertEqual(result["files"][0]["failure"], "manifest_asset_digests")

    def test_missing_and_whitespace_document_numbers_are_null(self):
        for line, expected in (("", None), ("document_number: '   '\n", None),
                               ("document_number: ' DOC-001 '\n", " DOC-001 ")):
            path = self.source / "policy.md"
            path.write_text(SOURCE.replace("version: 1.0", line + "version: 1.0"), encoding="utf-8")
            inv = v.source_inventory(path, self.source)
            self.assertEqual(v.metadata_for(inv, self.config)["document_number"], expected)

    def test_explicit_empty_document_number_scalar_is_rejected(self):
        for line in ("document_number:", "document_number: ''", 'document_number: ""'):
            with self.assertRaises(v.CheckError):
                v.source_fields(SOURCE.replace("version: 1.0", line + "\nversion: 1.0"))

    def test_footer_values_cannot_be_supplied_by_header(self):
        doc = v.package(self.doc)
        doc["xml"]["word/header1.xml"].append(ET.fromstring(f'<w:p xmlns:w="{v.W}"><w:r><w:t>1.0 Internal</w:t></w:r></w:p>'))
        doc["xml"]["word/footer1.xml"].find(".//w:t", v.NS).text = "Wrong footer"
        with self.assertRaisesRegex(v.CheckError, "docx_controlled_footer"):
            v.check_docx(doc, v.package(self.template), self.inventory, META)

    def test_cover_without_controlled_footer_and_body_with_footer(self):
        template = v.package(self.template)
        section = template["xml"]["word/document.xml"].find(".//w:sectPr", v.NS)
        ET.SubElement(section, f"{{{v.W}}}titlePg")
        profile = v.footer_profile(template)
        self.assertFalse(profile["cover_required"])
        path = self.root / "cover.pdf"
        with pymupdf.open() as document:
            document.new_page().insert_text((50, 70), "Synthetic policy")
            page = document.new_page()
            page.insert_text((50, 70), "Synthetic body")
            page.insert_text((50, 800), "1.0 Internal")
            document.save(path)
        renders = self.root / "renders"
        renders.mkdir()
        result = v.check_pdf(path, {"chunks": ["Synthetic body"]}, META, renders, "file-001", profile)
        self.assertEqual(result["footer_checked_pages"], [2])

    def test_missing_body_footer_still_fails_with_cover_exemption(self):
        path = self.root / "body.pdf"
        with pymupdf.open() as document:
            document.new_page().insert_text((50, 70), "Synthetic policy")
            document.new_page().insert_text((50, 70), "Synthetic body")
            document.save(path)
        with self.assertRaisesRegex(v.CheckError, "pdf_footer_controlled_chrome"):
            v.check_pdf(path, {"chunks": []}, META, self.root, "file-001", {"cover_required": False})

    def test_each_relevant_footer_is_checked_but_static_first_story_is_not(self):
        template, doc = v.package(self.template), v.package(self.doc)
        for package in (template, doc):
            section = package["xml"]["word/document.xml"].find(".//w:sectPr", v.NS)
            ET.SubElement(section, f"{{{v.W}}}titlePg")
            ET.SubElement(section, f"{{{v.W}}}footerReference", {f"{{{v.W}}}type": "first", f"{{{v.R}}}id": "first"})
            package["relationships"]["word/_rels/document.xml.rels"]["first"] = "word/footer2.xml"
            package["xml"]["word/footer2.xml"] = ET.fromstring(f'<w:ftr xmlns:w="{v.W}"><w:p><w:r><w:t>Static cover</w:t></w:r></w:p></w:ftr>')
        v.check_docx(doc, template, self.inventory, META)
        template["xml"]["word/footer2.xml"].find(".//w:t", v.NS).text = "{CONFIDENTIALITY}"
        with self.assertRaisesRegex(v.CheckError, "docx_controlled_footer"):
            v.check_docx(doc, template, self.inventory, META)
        doc["xml"]["word/footer2.xml"].find(".//w:t", v.NS).text = "Internal"
        v.check_docx(doc, template, self.inventory, META)
        self.assertEqual(v.footer_profile(template)["cover_fields"], {"CONFIDENTIALITY"})

    def test_success_digest_lie_rejected(self):
        result = self.run_fixture(lambda m: m["results"][0]["docx"].update(sha256="0" * 64))
        self.assertEqual(result["verified"], 0)
        self.assertEqual(result["files"][0]["failure"], "manifest_final_artifact_digest")

    def test_failed_entry_never_success(self):
        result = self.run_fixture(lambda m: m["results"][0].update(status="failed", error="private body"))
        self.assertEqual(result["failed"], 1)
        self.assertEqual(result["verified"], 0)
        self.assertNotIn("private body", json.dumps(result))

    def test_pdf_missing_text_rejected(self):
        path = self.root / "sample.pdf"
        pdf(path, omit="Synthetic verification text.")
        renders = self.root / "renders"
        renders.mkdir()
        with self.assertRaisesRegex(v.CheckError, "pdf_source_text_missing"):
            v.check_pdf(path, self.inventory, META, renders, "file-001")

    def test_snapshot_catches_hidden_release_addition_and_byte_change(self):
        before = v.snapshot(self.source)
        hidden = self.source / ".dms"
        hidden.mkdir()
        (hidden / "release.json").write_text('{}')
        self.assertNotEqual(before, v.snapshot(self.source))
        other = v.snapshot(self.source)
        (self.source / "assets/image.png").write_bytes(b"changed")
        self.assertNotEqual(other, v.snapshot(self.source))

    def test_fresh_refuses_existing_even_empty(self):
        with self.assertRaisesRegex(v.CheckError, "already_exists"):
            v.fresh(self.source)

    def test_manifest_paths_cannot_escape(self):
        for name in ("../template.docx", str(self.template), "C:/secret", "assets\\image.png"):
            with self.assertRaises(v.CheckError):
                v.relative_file(self.source, name)

    def test_zip_and_xml_malformed_rejected(self):
        self.doc.write_bytes(b"not a zip")
        with self.assertRaises(zipfile.BadZipFile):
            v.package(self.doc)
        with zipfile.ZipFile(self.doc, "w") as archive:
            archive.writestr("word/document.xml", "<unclosed>")
        with self.assertRaises(ET.ParseError):
            v.package(self.doc)

    def test_cli_arguments_not_shell_and_diagnostics_captured(self):
        with patch.object(v.subprocess, "run", return_value=None) as run:
            v.run_cli(Path("dms.exe"), Path("configuration with spaces.json"), Path("input & data"), Path("out"))
        args, kwargs = run.call_args
        self.assertEqual(args[0][5], "configuration with spaces.json")
        self.assertFalse(kwargs.get("shell", False))
        self.assertTrue(kwargs["capture_output"])

    def test_literal_frontmatter_not_yaml_unescaped(self):
        fields, _ = v.source_fields('---\ntitle: "A\\nB"\nversion: 1.0\n---\n')
        self.assertEqual(fields["title"], "A\\nB")

    def execution_args(self, expected=1):
        edit = self.root / "edit"
        (edit / ".dms").mkdir(parents=True)
        publish = self.root / "publish"
        publish.mkdir()
        (edit / "template.docx").write_bytes(self.template.read_bytes())
        config = {**self.config, "edit_root": str(edit), "publish_root": str(publish),
                  "identities": ["SYNTHETIC_PRIVATE_IDENTITY_DO_NOT_LOG"]}
        config_path = edit / ".dms/workspace.json"
        config_path.write_text(json.dumps(config), encoding="utf-8")
        dms = self.root / "dms.exe"
        dms.write_bytes(b"synthetic executable placeholder, never executed")
        policy_manifest = self.root / "policies.json"
        policy_manifest.write_text(json.dumps(["policy.md"]), encoding="utf-8")
        return argparse.Namespace(workspace_config=str(config_path), input_root=str(self.source),
                                  output_root=str(self.root / "run"), expected_count=expected, dms=str(dms),
                                  policy_manifest=str(policy_manifest))

    def test_unlisted_markdown_is_not_silently_a_policy(self):
        args = self.execution_args(expected=2)
        (self.source / "README.md").write_text(SOURCE, encoding="utf-8")
        with (patch.object(v.sys, "platform", "win32"), patch.object(v, "run_cli") as run,
              self.assertRaisesRegex(v.CheckError, "selected_count|policy_selection")):
            v.execute(args)
        run.assert_not_called()
        self.assertFalse((self.root / "run-evidence").exists())

    def test_same_count_different_policy_set_refused(self):
        args = self.execution_args()
        Path(args.policy_manifest).write_text(json.dumps(["different.md"]), encoding="utf-8")
        with (patch.object(v.sys, "platform", "win32"), patch.object(v, "run_cli") as run,
              self.assertRaisesRegex(v.CheckError, "policy_selection")):
            v.execute(args)
        run.assert_not_called()

    def test_policy_manifest_duplicates_paths_and_case_collisions_refused(self):
        path = self.root / "policies.json"
        for names in (["policy.md", "policy.md"], ["policy.md", "POLICY.md"],
                      ["../policy.md"], ["assets/policy.md"], ["policy.txt"]):
            path.write_text(json.dumps(names), encoding="utf-8")
            with self.assertRaises(v.CheckError):
                v.select_policies(self.source, path, len(names))

    def test_count_drift_fails_before_cli_or_output(self):
        args = self.execution_args(expected=49)
        with (patch.object(v.sys, "platform", "win32"), patch.object(v, "run_cli") as run,
              self.assertRaisesRegex(v.CheckError, "selected_count")):
            v.execute(args)
        run.assert_not_called()
        self.assertFalse((self.root / "run").exists())
        self.assertFalse((self.root / "run-evidence").exists())

    def test_48_policies_do_not_satisfy_required_49(self):
        args = self.execution_args(expected=49)
        names = ["policy.md"] + [f"policy-{i:02}.md" for i in range(47)]
        for name in names[1:]:
            (self.source / name).write_text(SOURCE, encoding="utf-8")
        Path(args.policy_manifest).write_text(json.dumps(names), encoding="utf-8")
        with (patch.object(v.sys, "platform", "win32"), patch.object(v, "run_cli") as run,
              self.assertRaisesRegex(v.CheckError, "selected_count")):
            v.execute(args)
        run.assert_not_called()
        self.assertFalse((self.root / "run").exists())
        self.assertFalse((self.root / "run-evidence").exists())

    def test_cli_failure_still_checks_snapshots_and_sanitizes(self):
        args = self.execution_args()
        def failed_cli(*_args):
            (self.root / "publish/unexpected-release.pdf").write_bytes(b"synthetic mutation")
            return subprocess.CompletedProcess([], 1, b"SYNTHETIC_PRIVATE_BODY", b"SYNTHETIC_PRIVATE_ERROR")
        stdout = io.StringIO()
        with patch.object(v.sys, "platform", "win32"), patch.object(v, "run_cli", side_effect=failed_cli), contextlib.redirect_stdout(stdout):
            self.assertEqual(v.execute(args), 1)
        result = json.loads(stdout.getvalue())
        self.assertEqual(result["failure"], "protected_tree_changed_or_unreadable")
        self.assertFalse(result["unchanged"]["publish"])
        self.assertFalse(result.get("no_dms_release_evidence", False))
        self.assertNotIn("SYNTHETIC_PRIVATE", stdout.getvalue())
        self.assertEqual(v.load_json(self.root / "run-evidence/evidence.json"), result)

    def test_selection_receipt_and_all_protected_snapshots_survive_failure(self):
        args = self.execution_args()
        before = {key: v.snapshot(path) for key, path in {
            "input": self.source, "edit": self.root / "edit", "publish": self.root / "publish",
            "configuration": Path(args.workspace_config), "template": self.root / "edit/template.docx",
            "policy_manifest": Path(args.policy_manifest)}.items()}
        stdout = io.StringIO()
        process = subprocess.CompletedProcess([], 1, b"SYNTHETIC_PRIVATE_BODY", b"SYNTHETIC_PRIVATE_ERROR")
        with (patch.object(v.sys, "platform", "win32"), patch.object(v, "run_cli", return_value=process),
              contextlib.redirect_stdout(stdout)):
            self.assertEqual(v.execute(args), 1)
        result = json.loads(stdout.getvalue())
        self.assertEqual(result["unchanged"], dict.fromkeys(before, True))
        self.assertEqual(result["snapshots"], {k: v.snapshot_digest(value) for k, value in before.items()})
        self.assertEqual(result["selected_set_sha256"], v.sha(b'["policy.md"]'))
        self.assertEqual(result["selection"], [{"file_id": "file-001", "source_path_sha256": v.sha(b"policy.md"),
                                               "input_sha256": v.file_sha(self.source / "policy.md")}])
        self.assertNotIn("policy.md", stdout.getvalue())
        self.assertNotIn("SYNTHETIC_PRIVATE", stdout.getvalue())

    def test_unreadable_snapshot_subtree_is_not_silently_skipped(self):
        def fail_walk(*_args, **kwargs):
            kwargs["onerror"](PermissionError("private path"))
        with (patch.object(v.os, "walk", side_effect=fail_walk),
              self.assertRaisesRegex(v.CheckError, "snapshot_unreadable_subtree")):
            v.snapshot(self.source)

    def test_negative_probes_require_actual_refusal_cause(self):
        output = self.root / "existing"
        output.mkdir()
        with (patch.object(v, "run_cli", return_value=subprocess.CompletedProcess([], 1, b"", b"Word unavailable")),
              self.assertRaisesRegex(v.CheckError, "existing_output_not_refused")):
            v.negative_probes(None, None, self.config, self.source, output, self.root)

    def test_controlled_failure_and_continuation(self):
        self.negative_fixture(false_success=False)

    def test_controlled_failure_cannot_claim_success_hash(self):
        with self.assertRaisesRegex(v.CheckError, "controlled_failure_false_success"):
            self.negative_fixture(false_success=True)

    def negative_fixture(self, false_success):
        output = self.root / "existing"
        output.mkdir()
        def fake_cli(_dms, _config, source, target):
            if target == output:
                return subprocess.CompletedProcess([], 1, b"", b"output root already exists; overwrite and resume are not supported")
            target.mkdir()
            value = self.manifest()
            bad = copy.deepcopy(value["results"][0])
            bad.update(input="a-failure.md", input_sha256=v.file_sha(source / "a-failure.md"),
                       status="failed", error="Markdown image rejected: synthetic missing image", word_version=None)
            if false_success:
                bad["docx"] = {"relative_path": "a-failure.docx", "sha256": "0" * 64}
            good = copy.deepcopy(value["results"][0])
            good["input"] = "b-success.md"
            docx(target / "b-success.docx")
            pdf(target / "b-success.pdf")
            for kind in ("docx", "pdf"):
                good[kind] = {"relative_path": "b-success." + kind, "sha256": v.file_sha(target / ("b-success." + kind))}
            value["results"] = [bad, good]
            (target / "manifest.json").write_text(json.dumps(value), encoding="utf-8")
            return subprocess.CompletedProcess([], 1, json.dumps(value).encode(), b"private diagnostic")
        with patch.object(v, "run_cli", side_effect=fake_cli):
            result = v.negative_probes(None, None, self.config, self.source, output, self.root)
        self.assertEqual(result["controlled_failed"], 1)
        self.assertEqual(result["continued_success"], 1)
        self.assertTrue(result["rerun_unchanged"])

    @unittest.skipUnless(v.sys.platform == "win32", "native Windows path spelling")
    def test_windows_verbatim_paths_compare_as_same_root(self):
        root = v.canonical(self.root)
        text = str(root)
        verbatim = "\\\\?\\UNC\\" + text[2:] if text.startswith("\\\\") else "\\\\?\\" + text
        self.assertEqual(v.canonical(Path(verbatim)), root)

    @unittest.skipUnless(v.sys.platform == "win32", "native PowerShell and uv argv")
    def test_powershell_parse_parameters_and_uv_script_argv(self):
        # Parse the real harness, then exercise the same uv argument shape with
        # synthetic values and a Python argv recorder. Do not execute the
        # harness body or alter/bypass host PowerShell execution policy.
        probe_dir = self.root / "argv probe & data"
        probe_dir.mkdir()
        (probe_dir / "verify_markdown_batch.py").write_text(
            '# /// script\n# requires-python = ">=3.11"\n# dependencies = []\n# ///\n'
            'import json, sys\nprint(json.dumps(sys.argv[1:]))\n', encoding="utf-8")
        exe = probe_dir / "synthetic dms.exe"
        exe.write_bytes(b"not executed")
        def quote(path):
            return "'" + str(path).replace("'", "''") + "'"
        harness = Path(__file__).with_name("Test-MarkdownBatchExport.ps1").resolve()
        command = f"""
        $ErrorActionPreference = 'Stop'
        $tokens = $null; $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile({quote(harness)}, [ref]$tokens, [ref]$errors)
        if ($errors.Count) {{ throw 'parse errors' }}
        $mandatory = @($ast.ParamBlock.Parameters | Where-Object {{ $_.Attributes.Extent.Text -match 'Mandatory = \\$true' }} | ForEach-Object {{ $_.Name.VariablePath.UserPath }})
        if (($mandatory | Sort-Object) -join ',' -ne 'ExpectedCount,InputRoot,OutputRoot,PolicyManifest,WorkspaceConfig') {{ throw 'mandatory parameters' }}
        $PSScriptRoot = {quote(probe_dir)}
        $WorkspaceConfig = 'config with spaces & data.json'; $InputRoot = 'input with spaces & data'
        $OutputRoot = 'output with spaces & data'; $PolicyManifest = 'policy list & data.json'
        $ExpectedCount = 49; $DmsPath = {quote(exe)}
        $arguments = @('run', '--quiet', '--no-project', '--script',
            (Join-Path $PSScriptRoot 'verify_markdown_batch.py'),
            '--workspace-config', $WorkspaceConfig, '--input-root', $InputRoot,
            '--output-root', $OutputRoot, '--expected-count', $ExpectedCount.ToString(),
            '--policy-manifest', $PolicyManifest,
            '--dms', (Resolve-Path -LiteralPath $DmsPath).ProviderPath)
        $uv = Get-Command uv.exe -CommandType Application
        & $uv.Source @arguments
        exit $LASTEXITCODE
        """
        assignment = command[command.index("$arguments ="):command.index("$uv =")].strip()
        self.assertIn(re.sub(r"\s+", " ", assignment), re.sub(r"\s+", " ", harness.read_text(encoding="utf-8")))
        process = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command],
                                 capture_output=True, check=False)
        self.assertEqual(process.returncode, 0, process.stderr.decode(errors="replace"))
        self.assertEqual(json.loads(process.stdout), ["--workspace-config", "config with spaces & data.json",
                         "--input-root", "input with spaces & data", "--output-root", "output with spaces & data",
                         "--expected-count", "49", "--policy-manifest", "policy list & data.json", "--dms", str(exe)])


if __name__ == "__main__":
    unittest.main()
