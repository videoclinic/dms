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
from datetime import date
from pathlib import Path
from unittest.mock import MagicMock, patch

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


def tagged_pdf(path, structure, page_count=1):
    """Real marked content and structure, including deliberately split Word tags."""
    with pymupdf.open() as document:
        for _ in range(page_count):
            page = document.new_page()
            page.insert_text((50, 40), "Synthetic policy")
            page.insert_text((50, 800), "1.0 Internal")
        root = document.get_new_xref()
        document.update_object(root, "<< /Type /StructTreeRoot >>")
        mcids = [0] * page_count
        parents = [[] for _ in range(page_count)]

        def add(role, children, parent):
            xref = document.get_new_xref()
            document.update_object(xref, "<< >>")
            kids = []
            for child in children:
                if isinstance(child[0], str):
                    kids.append(f"{add(*child, xref)} 0 R")
                else:
                    pn, x, y, text = child[:4]
                    direct = len(child) == 5 and child[4] == "parent"
                    page = document[pn]
                    mcid = mcids[pn]
                    mcids[pn] += 1
                    span = xref if direct else document.get_new_xref()
                    if not direct:
                        document.update_object(span, f"<< /Type /StructElem /S /Span /P {xref} 0 R "
                                               f"/Pg {page.xref} 0 R /K {mcid} >>")
                    parents[pn].append(f"{span} 0 R")
                    page.insert_text((x, y), text)
                    stream = page.get_contents()[-1]
                    document.update_stream(stream, f"/Span <</MCID {mcid}>> BDC\n".encode()
                                           + document.xref_stream(stream) + b"\nEMC")
                    kids.append(str(mcid) if direct else f"{span} 0 R")
            document.update_object(xref, f"<< /Type /StructElem /S /{role} /P {parent} 0 R /K [{' '.join(kids)}] >>")
            direct_pages = [child[0] for child in children if len(child) == 5 and child[4] == "parent"]
            if direct_pages:
                assert len(set(direct_pages)) == 1  # Integer K entries use their owner's page.
                document.xref_set_key(xref, "Pg", f"{document[direct_pages[0]].xref} 0 R")
            return xref

        body = add("Document", structure, root)
        document.xref_set_key(root, "K", f"[{body} 0 R]")
        parent_tree = document.get_new_xref()
        nums = " ".join(f"{pn} [{' '.join(refs)}]" for pn, refs in enumerate(parents))
        document.update_object(parent_tree, f"<< /Nums [{nums}] >>")
        document.xref_set_key(root, "ParentTree", f"{parent_tree} 0 R")
        for pn, page in enumerate(document):
            document.xref_set_key(page.xref, "StructParents", str(pn))
        document.xref_set_key(document.pdf_catalog(), "StructTreeRoot", f"{root} 0 R")
        document.xref_set_key(document.pdf_catalog(), "MarkInfo", "<< /Marked true >>")
        document.save(path)


def pdf(path, omit=""):
    structure = [(kind, [(0, 50, y, text)]) for kind, y, text in (
        ("H1", 70, "Synthetic heading"), ("P", 100, "Synthetic verification text."),
        ("H2", 130, "Versionsverlauf")) if text != omit]
    structure.append(("Table", [("TR", [("TH" if ri == 0 else "TD", [(0, 50 + ci * 95, 160 + ri * 30, text)])
                                      for ci, text in enumerate(row)])
                                for ri, row in enumerate((["Version", "Date", "Author", "Change", "Review"],
                                                          ["1.0", "2026", "Person", "Initial", "Done"]))]))
    tagged_pdf(path, structure)


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

    def test_heading_attribute_inventory_rejects_unclosed_repeated_markers(self):
        suffix = "{" + ("#" * 50) + "!"
        path = self.source / "policy.md"
        path.write_text(SOURCE + f"\n# Slow heading {suffix}\n", encoding="utf-8")
        inv = v.source_inventory(path, self.source)
        self.assertIn((1, f"Slow heading {suffix}"), inv["headings"])

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

    def pdf_matches(self, structure, body, page_count=1):
        path = self.source / "policy.md"
        history = SOURCE[SOURCE.index("## Versionsverlauf"):]
        path.write_text("---\ntitle: Synthetic policy\n---\n" + body + "\n" + history, encoding="utf-8")
        inventory = v.source_inventory(path, self.source)
        # The fixture exercises only the supplied body; inventory parsing still
        # establishes paragraph/heading/cell identity from actual Markdown.
        end = inventory["chunks"].index("Versionsverlauf")
        inventory["chunks"] = inventory["chunks"][:end]
        inventory["chunk_contexts"] = inventory["chunk_contexts"][:end]
        inventory["tables"] = inventory["tables"][:-1]
        target = self.root / "structure.pdf"
        tagged_pdf(target, structure, page_count)
        with pymupdf.open(target) as document:
            return v.pdf_source_matches(document, inventory)

    def test_pdf_wrapped_columns_are_cell_contiguous_and_row_major(self):
        structure = [("Table", [
            ("TR", [("TH", [(0, 50, 90, "Left")]), ("TH", [(0, 300, 90, "Right")])]),
            ("TR", [("TD", [(0, 50, 120, "alpha\nbeta")]), ("TD", [(0, 300, 120, "gamma\ndelta")])]),
            ("TR", [("TD", [(0, 50, 180, "omega")]), ("TD", [(0, 300, 180, "end")])]),
        ])]
        body = "| Left | Right |\n| --- | --- |\n| alpha beta | gamma delta |\n| omega | end |\n"
        self.assertEqual(self.pdf_matches(structure, body), [True] * 6)
        for wrong in (body.replace("alpha beta", "alpha MISSING beta"),
                      body.replace("alpha beta | gamma delta", "gamma delta | alpha beta"),
                      body.replace("omega | end", "end | omega")):
            with self.subTest(wrong=wrong):
                self.assertFalse(all(self.pdf_matches(structure, wrong)))

    def test_pdf_paragraph_continues_across_page_without_chrome(self):
        structure = [("P", [(0, 50, 740, "Start of paragraph"), (1, 50, 90, "and its ending.")])]
        self.assertEqual(self.pdf_matches(structure, "Start of paragraph and its ending.\n", 2), [True])
        self.assertEqual(self.pdf_matches(structure, "Start of paragraph LOST and its ending.\n", 2), [False])

    def test_pdf_painted_paragraph_survives_lossy_structured_xml(self):
        structure = [("P", [(0, 50, 740, "Start of paragraph"),
                             (1, 50, 90, "and its"), (1, 50, 105, "ending.")]),
                     ("P", [(1, 50, 140, "Separate ending.")])]
        original = pymupdf.Page.get_textpage

        def lossy(page, *args, **kwargs):
            textpage = original(page, *args, **kwargs)
            if page.number != 1:
                return textpage
            tree = ET.fromstring(textpage.extractXML())
            for node in tree.findall(".//struct[@std='P']")[0].iter("font"):
                node.clear()
            return MagicMock(extractXML=lambda: ET.tostring(tree, encoding="unicode"))

        # Paint and PDF marked-content ancestry remain real. Only the lossy
        # structured-text projection is substituted, never the expected source.
        with patch.object(pymupdf.Page, "get_textpage", lossy):
            self.assertEqual(self.pdf_matches(structure,
                             "Start of paragraph and its ending.\n\nSeparate ending.\n", 2), [True, True])
            for wrong in ("Start of paragraph LOST and its ending.",
                          "Start of paragraph and ending.",
                          "Start of paragraph Separate ending."):
                with self.subTest(wrong=wrong):
                    self.assertEqual(self.pdf_matches(structure, wrong + "\n", 2), [False])

    def test_pdf_painted_five_column_continuation_keeps_cell_ownership(self):
        headers = ["Version", "Date", "Author", "Change", "Review"]
        values = ["1.0", "2026", "Person", "First half", "Done"]
        structure = [("Table", [
            ("TR", [("TH", [(0, 50 + ci * 95, 90, text)]) for ci, text in enumerate(headers)]),
            ("TR", [("TD", [(0, 50 + ci * 95, 740, text)] +
                     ([(1, 50 + ci * 95, 90, "second half")] if ci == 3 else []))
                    for ci, text in enumerate(values)]),
        ])]
        body = ("| " + " | ".join(headers) + " |\n| --- | --- | --- | --- | --- |\n"
                "| 1.0 | 2026 | Person | First half second half | Done |\n")
        with patch.object(pymupdf.Page, "get_textpage") as extract:
            extract.return_value.extractXML.return_value = "<page/>"
            self.assertEqual(self.pdf_matches(structure, body, 2), [True] * 10)
            for wrong in (body.replace("second half", "second LOST half"),
                          body.replace("Person | First half second half", "Person second half | First half")):
                with self.subTest(wrong=wrong):
                    self.assertFalse(all(self.pdf_matches(structure, wrong, 2)))

    def test_pdf_ligature_continuations_preserve_bound_logical_text(self):
        target = self.root / "ligature.pdf"
        tagged_pdf(target, [("P", [(0, 50, 90, "ofce")]), ("P", [(0, 50, 120, "Next")])])
        with pymupdf.open(target) as document:
            font = document[0].get_fonts()[0][0]
            cmap = document.get_new_xref()
            document.update_object(cmap, "<< >>")
            document.update_stream(cmap, b"/CIDInit /ProcSet findresource begin 12 dict begin "
                                   b"begincmap /CMapType 2 def 1 begincodespacerange <00> <FF> "
                                   b"endcodespacerange 1 beginbfchar <66> <006600660069> "
                                   b"endbfchar endcmap CMapName currentdict /CMap defineresource pop end end")
            document.xref_set_key(font, "ToUnicode", f"{cmap} 0 R")
            # Reopen to discard font caches. The pinned PDF parser, not a mock
            # or the source inventory, supplies the continuation codepoints.
            with pymupdf.open(stream=document.tobytes(), filetype="pdf") as reopened:
                spans = reopened[0].get_texttrace()
                ligature = next(s for s in spans if any(c[1] < 0 for c in s["chars"]))
                continuations = [c for c in ligature["chars"] if c[1] < 0]
                self.assertEqual([c[0] for c in continuations], [ord("f"), ord("i")])
                self.assertTrue(all(pymupdf.Rect(c[3]).is_empty for c in continuations))
                self.assertEqual(v.pdf_paint_text([ligature]), "office")
                self.assertEqual(v.pdf_source_matches(reopened, {"chunks": ["office", "Next"]}), [True, True])
                for wrong in ("ofce", "ofice", "offfice", "office Next"):
                    with self.subTest(wrong=wrong):
                        self.assertEqual(v.pdf_source_matches(reopened, {"chunks": [wrong]}), [False])

    def test_pdf_ligature_empty_bounds_do_not_move_cell_columns(self):
        xml = ET.fromstring('<page><struct idx="0" std="Table">'
                            '<struct idx="0" std="TR">'
                            '<struct idx="0" std="TH"><paint index="0"/></struct>'
                            '<struct idx="1" std="TH"><paint index="1"/></struct></struct>'
                            '<struct idx="1" std="TR">'
                            '<struct idx="0" std="TD"><paint index="2"/></struct>'
                            '<struct idx="1" std="TD"><paint index="3"/></struct>'
                            '</struct></struct></page>')
        _, layers = self.checkmark_fixture()
        spans = [{**layers[2], "seqno": i,
                  "chars": [(ord(text), 100 + i, (x, y), (x, y - 8, x + 5, y + 2))]}
                 for i, (text, x, y) in enumerate((("L", 50, 90), ("R", 300, 90),
                                                  ("X", 50, 120), ("f", 300, 120)))]
        # A valid empty rectangle is not a painted column anchor, even if its
        # x coordinate differs from the positive component's rectangle.
        spans[3]["chars"].append((ord("i"), -1, (300, 120), (0, 112, 0, 122)))
        self.assertEqual(self.checkmark_matches(xml, spans, rows=[["L", "R"], ["X", "fi"]]), [True] * 4)
        for row in (["X", "f"], ["Xi", "f"], ["fi", "X"], ["X", "fii"]):
            with self.subTest(row=row):
                self.assertFalse(all(self.checkmark_matches(xml, spans, rows=[["L", "R"], row])))
        spans[3]["chars"].pop()
        self.assertEqual(self.checkmark_matches(xml, spans, rows=[["L", "R"], ["X", "fi"]]),
                         [True, True, True, False])

    def test_pdf_ligature_continuation_alone_has_no_unit_geometry(self):
        xml = ET.fromstring('<page><struct idx="0" std="P"><paint index="0"/></struct></page>')
        _, layers = self.checkmark_fixture()
        span = {**layers[2], "chars": [(ord("i"), -1, (50, 100), (50, 92, 50, 102))]}
        with self.assertRaisesRegex(v.CheckError, "pdf_text_structure_geometry"):
            self.checkmark_matches(xml, [span], text="i", kind="paragraph")

    def test_pdf_paint_rejects_invalid_codepoints_and_glyph_geometry(self):
        _, layers = self.checkmark_fixture()
        for code in (-1, 0x110000, 0xd800, 0xdfff, 102.5):
            for glyph, bounds in ((100, (50, 92, 55, 102)), (-1, (50, 92, 50, 102))):
                span = {**layers[2], "chars": [(code, glyph, (50, 100), bounds)]}
                with self.subTest(code=code, glyph=glyph), self.assertRaisesRegex(
                        v.CheckError, "pdf_source_paint_character_invalid"):
                    v.pdf_paint_text([span])
        for glyph, bounds in ((-1, (50, 92, 55, 102)), (0, (50, 92, 50, 102)),
                               (100, (50, 92, 50, 102)), (100, (50, 100, 55, 100))):
            span = {**layers[2], "chars": [(ord("i"), glyph, (50, 100), bounds)]}
            with self.subTest(glyph=glyph, bounds=bounds), self.assertRaisesRegex(
                    v.CheckError, "pdf_source_paint_character_invalid"):
                v.pdf_paint_text([span])

    def test_pdf_paint_rejects_malformed_geometry_for_both_component_kinds(self):
        _, layers = self.checkmark_fixture()
        malformed = [((50, 100), (55, 92, 50, 102)),
                     ((50, 100), (50, 102, 55, 92)),
                     ((50, 100), (50, 92, 55)),
                     ((50,), (50, 92, 55, 102)),
                     ((50, 100), (50, 92, "bad", 102)),
                     (("bad", 100), (50, 92, 55, 102)),
                     ((50, 100), tuple(pymupdf.INFINITE_RECT()))]
        for value in (float("nan"), float("inf"), -float("inf")):
            for index in range(4):
                bounds = [50.0, 92.0, 50.0, 102.0]
                bounds[index] = value
                malformed.append(((50, 100), tuple(bounds)))
            for index in range(2):
                origin = [50.0, 100.0]
                origin[index] = value
                malformed.append((tuple(origin), (50, 92, 50, 102)))
        for glyph in (-1, 100):
            for origin, bounds in malformed:
                span = {**layers[2], "chars": [(ord("i"), glyph, origin, bounds)]}
                with self.subTest(glyph=glyph, origin=origin, bounds=bounds), self.assertRaisesRegex(
                        v.CheckError, "pdf_source_paint_character_invalid"):
                    v.pdf_paint_text([span])

    def test_pdf_actualtext_metadata_cannot_replace_painted_source(self):
        target = self.root / "actualtext.pdf"
        tagged_pdf(target, [("P", [(0, 50, 90, "Painted words")])])
        with pymupdf.open(target) as document:
            for stream in document[0].get_contents():
                data = document.xref_stream(stream)
                document.update_stream(stream, data.replace(b"/MCID 0>>", b"/MCID 0 /ActualText (Invented words)>>"))
            # This is a real extractor disagreement, not a mocked text page.
            xml = document[0].get_textpage(flags=pymupdf.TEXTFLAGS_XML | pymupdf.TEXT_COLLECT_STRUCTURE).extractXML()
            extracted = "".join(c.get("c", "") for c in ET.fromstring(xml).iter("char"))
            self.assertIn("Invented words", extracted)
            self.assertNotIn("Painted words", extracted)
            for source, expected in (("Painted words", True), ("Invented words", False)):
                self.assertEqual(v.pdf_source_matches(document, {"chunks": [source]}), [expected])

    def test_pdf_paint_counter_tracks_paths_images_and_unmarked_text(self):
        target = self.root / "operations.pdf"
        tagged_pdf(target, [("P", [(0, 50, 90, "First")]), ("P", [(0, 50, 120, "Second")])])
        with pymupdf.open(target) as document:
            page = document[0]
            page.draw_rect(pymupdf.Rect(10, 10, 20, 20), color=(0, 0, 0), fill=(1, 0, 0))
            page.insert_image(pymupdf.Rect(25, 10, 35, 20), stream=PNG)
            page.insert_text((50, 105), "Unmarked decoy")
            streams = page.get_contents()
            second = next(xref for xref in streams if b"/MCID 1" in document.xref_stream(xref))
            document.xref_set_key(page.xref, "Contents", "[" + " ".join(
                f"{xref} 0 R" for xref in [*(r for r in streams if r != second), second]) + "]")
            for source in (["First", "Second"], ["First Unmarked decoy"]):
                with self.assertRaisesRegex(v.CheckError, "pdf_ambiguous_unmarked_paint"):
                    v.pdf_source_matches(document, {"chunks": source})
            # The ambiguous unmarked stream is not a recoverable source unit.
            decoy = streams[-1]
            document.update_stream(decoy, b"")
            for rotation in (0, 90):
                page.set_rotation(rotation)
                with self.subTest(rotation=rotation):
                    self.assertEqual(v.pdf_source_matches(document, {"chunks": ["First", "Second"]}), [True, True])
                    self.assertEqual(v.pdf_source_matches(document, {"chunks": ["First Unmarked decoy"]}), [False])
                    self.assertEqual(document[0].rotation, rotation)

    def test_pdf_same_local_indices_in_different_tables_stay_distinct(self):
        structure = [("Table", [("TR", [("TH", [(0, 50, y, "Column")])]),
                                 ("TR", [("TD", [(0, 50, y + 20, text)])])])
                     for y, text in ((90, "First"), (150, "Second"))]
        body = "| Column |\n| --- |\n| First |\n\n| Column |\n| --- |\n| Second |\n"
        self.assertEqual(self.pdf_matches(structure, body), [True] * 4)
        self.assertEqual(self.pdf_matches(structure, body.replace("Second", "First")), [True, True, True, False])

    def test_pdf_artifact_nested_in_source_mark_cannot_supply_source_text(self):
        target = self.root / "artifact.pdf"
        tagged_pdf(target, [("P", [(0, 50, 90, "Artifact decoy")])])
        with pymupdf.open(target) as document:
            for stream in document[0].get_contents():
                data = document.xref_stream(stream)
                if b"/MCID 0" in data:
                    document.update_stream(stream, data.replace(b"BDC", b"BDC /Artifact BMC", 1) + b"\nEMC")
            with self.assertRaisesRegex(v.CheckError, "pdf_ambiguous_unmarked_paint"):
                v.pdf_source_matches(document, {"chunks": ["Artifact decoy"]})

    def test_pdf_unlabelled_list_continuation_is_not_a_new_occurrence(self):
        structure = [("L", [
            ("LI", [("Lbl", [(0, 50, 740, "*")]), ("LBody", [(0, 70, 740, "First half")])]),
            ("LI", [("LBody", [(1, 70, 90, "second half.")])]),
        ])]
        self.assertEqual(self.pdf_matches(structure, "- First half second half.\n", 2), [True])
        structure[0][1][1][1].insert(0, ("Lbl", [(1, 50, 90, "*")]))
        self.assertEqual(self.pdf_matches(structure, "- First half second half.\n", 2), [False])

    def test_pdf_list_body_consumes_explicit_paragraphs_individually(self):
        structure = [("L", [("LI", [("Lbl", [(0, 50, 90, "1.")]), ("LBody", [
            ("P", [(0, 70, 90, "First paragraph.")]),
            ("P", [(0, 70, 120, "Second paragraph.")]),
        ])])])]
        body = "1. First paragraph.\n\n   Second paragraph.\n"
        self.assertEqual(self.pdf_matches(structure, body), [True, True])
        for wrong in (body.replace("Second", "LOST Second"),
                      body.replace("First paragraph.", "paragraph."),
                      body.replace("Second", "First"),
                      "1. Second paragraph.\n\n   First paragraph.\n"):
            with self.subTest(wrong=wrong):
                self.assertFalse(all(self.pdf_matches(structure, wrong)))
        self.assertEqual(self.pdf_matches(structure, "1. First paragraph. Second paragraph.\n"), [False])
        flattened = [("L", [("LI", [("LBody", [(0, 70, 90, "First paragraph. Second paragraph.")])])])]
        self.assertEqual(self.pdf_matches(flattened, body), [False, False])

    def test_pdf_list_body_explicit_paragraphs_keep_cross_page_identity(self):
        structure = [("L", [("LI", [("Lbl", [(0, 50, 740, "*")]), ("LBody", [
            ("P", [(0, 70, 740, "First half"), (1, 70, 90, "second half.")]),
            ("P", [(1, 70, 120, "Separate paragraph.")]),
        ])])])]
        body = "- First half second half.\n\n  Separate paragraph.\n"
        self.assertEqual(self.pdf_matches(structure, body, 2), [True, True])
        self.assertEqual(self.pdf_matches(structure, body + "\n  Separate paragraph.\n", 2),
                         [True, True, False])
        self.assertEqual(self.pdf_matches(structure, "- First half second half. Separate paragraph.\n", 2), [False])

    def test_pdf_ordered_list_prefix_in_p_and_lbody_is_exact_and_list_only(self):
        for role in ("P", "LBody"):
            for prefix in ("1.", "12)"):
                with self.subTest(role=role, prefix=prefix):
                    structure = [(role, [(0, 50, 90, prefix + " Expected body.")])]
                    self.assertEqual(self.pdf_matches(structure, "1. Expected body.\n"), [True])
                    self.assertEqual(self.pdf_matches(structure, "1. Wrong body.\n"), [False])
                    self.assertEqual(self.pdf_matches(structure, "Expected body.\n"), [False])
            for prefix in ("x1.", "1.2.", "(1)", "1..", "1", "1. Wrong"):
                with self.subTest(role=role, invalid_prefix=prefix):
                    structure = [(role, [(0, 50, 90, prefix + " Expected body.")])]
                    self.assertEqual(self.pdf_matches(structure, "1. Expected body.\n"), [False])
        structure = [("Code", [(0, 50, 90, "1. Expected body.")])]
        self.assertEqual(self.pdf_matches(structure, "1. Expected body.\n"), [False])

    def test_pdf_split_cell_uses_column_geometry_not_continuation_tag_index(self):
        structure = [("Table", [
            ("TR", [("TH", [(0, 50, 90, "Left")]), ("TH", [(0, 300, 90, "Right")])]),
            ("TR", [("TD", [(0, 50, 740, "First half")]), ("TD", [(0, 300, 740, "Neighbour")])]),
            ("TR", [("TD", []), ("TD", [(1, 50, 90, "second half.")])]),
            ("TR", [("TD", [(1, 50, 140, "Last")]), ("TD", [(1, 300, 140, "Row")])]),
        ])]
        body = "| Left | Right |\n| --- | --- |\n| First half second half. | Neighbour |\n| Last | Row |\n"
        self.assertEqual(self.pdf_matches(structure, body, 2), [True] * 6)
        self.assertFalse(all(self.pdf_matches(structure, body.replace("First half second", "First LOST half second"), 2)))
        structure[0][1][2][1][1] = ("TD", [(1, 300, 90, "second half.")])
        self.assertFalse(all(self.pdf_matches(structure, body, 2)))

    def test_pdf_next_page_sparse_row_is_not_assumed_to_be_a_continuation(self):
        structure = [("Table", [
            ("TR", [("TH", [(0, 50, 90, "Left")]), ("TH", [(0, 300, 90, "Right")])]),
            ("TR", [("TD", [(0, 50, 740, "First")]), ("TD", [(0, 300, 740, "Neighbour")])]),
            ("TR", [("TD", [(1, 50, 90, "Next")]), ("TD", [])]),
            ("TR", [("TD", [(1, 50, 140, "Last")]), ("TD", [(1, 300, 140, "Row")])]),
        ])]
        body = "| Left | Right |\n| --- | --- |\n| First | Neighbour |\n| Next | |\n| Last | Row |\n"
        self.assertEqual(self.pdf_matches(structure, body, 2), [True] * 7)
        for wrong in (body.replace("Next | |", "Next | MISSING |"),
                      body.replace("Next | |", "| Next |"), body.replace("Next", "Wrong")):
            with self.subTest(wrong=wrong):
                self.assertFalse(all(self.pdf_matches(structure, wrong, 2)))
        structure[0][1][2][1][1] = ("TD", [(1, 300, 90, "EXTRA")])
        self.assertFalse(all(self.pdf_matches(structure, body, 2)))
        # A continuation can also paint every column; populated-cell count
        # cannot decide between a separate row and a split source row.
        structure[0][1][2][1][1] = ("TD", [(1, 300, 90, "ending")])
        joined = body.replace("First | Neighbour |\n| Next | |", "First Next | Neighbour ending |")
        self.assertEqual(self.pdf_matches(structure, joined, 2), [True] * 6)
        self.assertEqual(self.pdf_matches(structure, joined.replace("Row", "Row \u2705"), 2),
                         [True, True, True, True, True, False])
        rows = [["Left", "Right"], ["FirstNext", "Neighbourending"], ["Last", "Row"]]
        ambiguous = [{"kind": "table", "rows": rows, "joined_rows": copy.deepcopy(rows)}]
        with patch.object(v, "pdf_reading_units", return_value=ambiguous):
            self.assertEqual(self.pdf_matches(structure, joined, 2), [False] * 6)

    def test_pdf_occurrences_order_and_paragraph_boundaries_are_strict(self):
        structure = [("P", [(0, 50, 90, "Repeat")]), ("P", [(0, 50, 120, "End")])]
        self.assertEqual(self.pdf_matches(structure, "Repeat\n\nRepeat\n\nEnd\n"), [True, False, True])
        self.assertFalse(all(self.pdf_matches(structure, "End\n\nRepeat\n")))
        self.assertEqual(self.pdf_matches(structure, "Repeat End\n"), [False])

    def test_pdf_missing_checkmark_is_still_a_deficit(self):
        structure = [("Table", [
            ("TR", [("TH", [(0, 50, 90, "Status")])]),
            ("TR", [("TD", [(0, 50, 120, "Done")])]),
        ])]
        self.assertEqual(self.pdf_matches(structure, "| Status |\n| --- |\n| Done \u2705 |\n"), [True, False])

    def checkmark_fixture(self):
        # Synthetic device-event tree plus independent paint traces. No system
        # font, Word, production PDF, or expected-source data is consulted.
        xml = ET.fromstring('<page><struct idx="0" std="Table"><struct idx="0" std="TR">'
                            '<struct idx="0" std="TD"><paint index="2"/>'
                            '<paint index="0"/><paint index="1"/>'
                            '</struct></struct></struct></page>')
        layers = [{"font": "SegoeUIEmoji", "size": 9.96, "type": 0, "opacity": 1.0,
                   "dir": (1.0, 0.0), "colorspace": 3, "color": color, "seqno": index + 1,
                   "chars": [(32, 100 + index, (90.0, 100.0), (90.0, 92.27, 103.68, 102.23))]}
                  for index, color in enumerate(((0, 210 / 255, 106 / 255), (244 / 255,) * 3))]
        layers.append({**layers[0], "font": "Helvetica", "color": (0, 0, 0), "seqno": 0,
                       "chars": [(ord(c), 200 + i, (50 + i * 5, 100),
                                  (50 + i * 5, 92, 55 + i * 5, 102)) for i, c in enumerate("Done")]})
        return xml, layers

    def checkmark_matches(self, xml, layers, text="Done \u2705", kind="cell", rows=None):
        # Device paint indices address get_texttrace's paint-ordered array.
        # Fixtures use stable IDs for mutations; project them to that array here.
        xml = copy.deepcopy(xml)
        order = sorted(range(len(layers)), key=lambda i: layers[i]["seqno"])
        positions = {old: new for new, old in enumerate(order)}
        for paint in xml.iter("paint"):
            paint.set("index", str(positions[int(paint.get("index"))]))
        layers = [layers[i] for i in order]
        page = MagicMock()
        document = MagicMock()
        document.xref_get_key.return_value = ("dict", "synthetic structure")
        document.__iter__.return_value = iter([page])
        inv = {"chunks": [text], "chunk_contexts": [("cell", 0, 0, 0) if kind == "cell" else (kind,)],
               "tables": [[[text]]] if kind == "cell" else []}
        if rows is not None:
            inv = {"chunks": [c for row in rows for c in row], "tables": [rows],
                   "chunk_contexts": [("cell", 0, ri, ci) for ri, row in enumerate(rows)
                                      for ci, _ in enumerate(row)]}
        with patch.object(v, "pdf_paint_structure", return_value=(xml, layers)):
            return v.pdf_source_matches(document, inv)

    def test_pdf_checkmark_two_layers_at_bound_cell_location(self):
        xml, layers = self.checkmark_fixture()
        self.assertEqual(self.checkmark_matches(xml, layers), [True])

    def test_pdf_checkmark_surrounding_text_without_paint_is_rejected(self):
        xml, layers = self.checkmark_fixture()
        cell = xml.find(".//struct[@std='TD']")
        for node in list(cell)[1:]:
            cell.remove(node)
        self.assertEqual(self.checkmark_matches(xml, layers), [False])

    def test_pdf_checkmark_bound_multispan_text_keeps_exact_position(self):
        for role in ("TD", "TH"):
            xml, layers = self.checkmark_fixture()
            cell = xml.find(".//struct[@std='TD']")
            cell.set("std", role)
            for text, y in (("APPROVED", 115), ("after", 130)):
                layers.append({**layers[2], "seqno": 1 if text == "APPROVED" else 4,
                               "chars": [(ord(c), 300 + i, (50 + i * 5, y),
                                          (50 + i * 5, y - 8, 55 + i * 5, y + 2))
                                         for i, c in enumerate(text)]})
            cell.insert(1, ET.Element("paint", index="3"))
            ET.SubElement(cell, "paint", index="4")
            layers[0]["seqno"], layers[1]["seqno"] = 2, 3
            with self.subTest(role=role):
                self.assertEqual(self.checkmark_matches(xml, layers, "Done APPROVED \u2705 after"), [True])
                for wrong in ("Done \u2705 APPROVED after", "APPROVED Done \u2705 after",
                              "Done APPROVE \u2705 after", "Done APPROVEDX \u2705 after",
                              "Done APPROVED \u2705 \u2705 after", "Done APPROVED after"):
                    self.assertEqual(self.checkmark_matches(xml, layers, wrong), [False])

    def test_pdf_real_cell_parent_and_child_paints_follow_paint_order(self):
        target = self.root / "mixed-owner.pdf"
        for role in ("TD", "TH"):
            structure = [("Table", [("TR", [(role, [
                (0, 50, 90, "Before ", "parent"),
                (0, 90, 90, "inside "),
                (0, 130, 90, "after", "parent"),
            ])])])]
            tagged_pdf(target, structure)
            with self.subTest(role=role), pymupdf.open(target) as document:
                tree, _ = v.pdf_paint_structure(document[0])
                cell = tree.find(f".//struct[@std='{role}']")
                assert cell is not None
                indices = [int(p.attrib["index"]) for p in cell.iter("paint")]
                self.assertNotEqual(indices, sorted(indices))
                for text, expected in (("Before inside after", True), ("inside Before after", False),
                                       ("Before after inside", False), ("Before after", False),
                                       ("Before inside after LOST", False)):
                    inventory = {"chunks": [text], "chunk_contexts": [("cell", 0, 0, 0)], "tables": [[[text]]]}
                    self.assertEqual(v.pdf_source_matches(document, inventory), [expected])

    def test_pdf_cell_parent_and_child_paints_follow_paint_order(self):
        for role in ("TD", "TH"):
            xml, layers = self.checkmark_fixture()
            cell = xml.find(".//struct[@std='TD']")
            assert cell is not None
            cell.set("std", role)
            for node in list(cell):
                cell.remove(node)
            # Child spans are created during the device walk, before direct
            # parent-owned paints are appended. XML order is not paint order.
            child = ET.SubElement(cell, "struct", idx="0", std="Span")
            ET.SubElement(child, "paint", index="0")
            ET.SubElement(child, "paint", index="1")
            ET.SubElement(cell, "paint", index="2")
            layers.append({**layers[2], "seqno": 3,
                           "chars": [(ord(c), 300 + i, (120 + i * 5, 100),
                                      (120 + i * 5, 92, 125 + i * 5, 102)) for i, c in enumerate("after")]})
            ET.SubElement(cell, "paint", index="3")
            with self.subTest(role=role):
                self.assertEqual(self.checkmark_matches(xml, layers, "Done \u2705 after"), [True])
                for wrong in ("\u2705 Done after", "Done after \u2705", "Done after", "Done \u2705 \u2705 after",
                              "Done LOST \u2705 after", "Done \u2705 afte"):
                    self.assertEqual(self.checkmark_matches(xml, layers, wrong), [False])

    def test_pdf_checkmark_no_borrowing_from_unmarked_or_foreign_paint(self):
        for owner in ("unmarked", "P", "TD"):
            xml, layers = self.checkmark_fixture()
            cell = xml.find(".//struct[@std='TD']")
            ordinary = cell.find("paint[@index='2']")
            cell.remove(ordinary)
            if owner == "unmarked":
                xml.append(ordinary)
            elif owner == "P":
                ET.SubElement(xml, "struct", idx="-1", std="P").append(ordinary)
            else:
                table = ET.SubElement(xml, "struct", idx="1", std="Table")
                row = ET.SubElement(table, "struct", idx="0", std="TR")
                ET.SubElement(row, "struct", idx="0", std="TD").append(ordinary)
            with self.subTest(owner=owner):
                self.assertEqual(self.checkmark_matches(xml, layers), [False])

    def test_pdf_checkmark_rejects_hidden_ordinary_text_and_layers(self):
        for index in (0, 1, 2):
            for attrs in ({"opacity": 0}, {"opacity": .5}, {"type": 3}, {"dir": (0, 1)}):
                xml, layers = self.checkmark_fixture()
                layers[index].update(attrs)
                with self.subTest(index=index, attrs=attrs), self.assertRaises(v.CheckError):
                    self.checkmark_matches(xml, layers)

    def test_pdf_checkmark_same_paint_cannot_supply_repeated_source_cell(self):
        xml, layers = self.checkmark_fixture()
        self.assertEqual(self.checkmark_matches(xml, layers, rows=[["Done \u2705"], ["Done \u2705"]]),
                         [False, False])
        table = xml.find(".//struct[@std='Table']")
        clone = copy.deepcopy(table[0])
        clone.set("idx", "1")
        table.append(clone)
        # Even an invalid duplicate ownership fixture cannot reuse the origin.
        self.assertEqual(self.checkmark_matches(xml, layers, rows=[["Done \u2705"], ["Done \u2705"]]),
                         [True, False])

    def test_pdf_checkmark_requires_both_exact_colour_layers(self):
        for change in (lambda s: s[0].update(chars=[]), lambda s: s[1].update(chars=[]),
                       lambda s: s[1].update(color=(1, 1, 1)),
                       lambda s: s[0].update(color=(0, 1, 0)),
                       lambda s: s[1].update(font="OtherFont"),
                       lambda s: s[1].update(seqno=4),
                       lambda s: s[1]["chars"].append(s[1]["chars"][0])):
            xml, layers = self.checkmark_fixture()
            change(layers)
            with self.subTest(layers=layers):
                self.assertEqual(self.checkmark_matches(xml, layers), [False])

    def test_pdf_checkmark_rejects_degenerate_wrong_size_or_displaced_layers(self):
        for origin, bounds in (((90, 100), (90, 92.27, 90, 102.23)),
                               ((90, 100), (90, 92.27, 93, 102.23)),
                               ((90, 100), (90, 100, 103.68, 100)),
                               ((110, 100), (110, 92.27, 123.68, 102.23)),
                               ((90, 130), (90, 122.27, 103.68, 132.23))):
            xml, layers = self.checkmark_fixture()
            layers[1]["chars"] = [(32, 101, origin, bounds)]
            with self.subTest(origin=origin, bounds=bounds):
                if pymupdf.Rect(bounds).is_empty:
                    with self.assertRaises(v.CheckError):
                        self.checkmark_matches(xml, layers)
                else:
                    self.assertEqual(self.checkmark_matches(xml, layers), [False])
        xml, layers = self.checkmark_fixture()
        layers[1]["size"] = 5
        self.assertEqual(self.checkmark_matches(xml, layers), [False])

    def test_pdf_checkmark_location_order_count_and_other_text_are_strict(self):
        for text in ("\u2705 Done", "Done \u2705 \u2705", "Done LOST \u2705", "Wrong \u2705", "Done \u2714", "Done"):
            xml, layers = self.checkmark_fixture()
            with self.subTest(text=text):
                self.assertEqual(self.checkmark_matches(xml, layers, text), [False])

    def test_pdf_checkmark_evidence_in_other_cell_or_paragraph_cannot_satisfy(self):
        for moved in ((0,), (1,), (0, 1)):
            xml, layers = self.checkmark_fixture()
            cell = xml.find(".//struct[@std='TD']")
            # Identical coordinates do not grant cell ownership.
            other = ET.SubElement(xml, "struct", idx="1", std="P")
            for index in moved:
                glyph = cell.find(f"paint[@index='{index}']")
                cell.remove(glyph)
                other.append(glyph)
            with self.subTest(moved=moved):
                self.assertEqual(self.checkmark_matches(xml, layers), [False])
        xml, layers = self.checkmark_fixture()
        xml.find(".//struct[@std='TD']").set("std", "P")
        self.assertEqual(self.checkmark_matches(xml, layers, kind="paragraph"), [False])

    def test_pdf_checkmark_requires_distinct_nonzero_glyphs_mapping_to_spaces(self):
        for code, glyph in ((ord("X"), 100), (32, 0), (32, 101)):
            xml, layers = self.checkmark_fixture()
            _, _, point, bounds = layers[0]["chars"][0]
            layers[0]["chars"] = [(code, glyph, point, bounds)]
            with self.subTest(code=code, glyph=glyph):
                self.assertEqual(self.checkmark_matches(xml, layers), [False])

    def test_pdf_cover_title_accepts_all_caps_but_rejects_wrong_or_later_title(self):
        renders = self.root / "renders"
        renders.mkdir()
        upper = self.root / "upper.pdf"
        with pymupdf.open() as document:
            page = document.new_page()
            page.insert_text((50, 70), "SYNTHETIC POLICY")
            page.insert_text((50, 800), "1.0 Internal")
            document.save(upper)
        v.check_pdf(upper, {"chunks": []}, META, renders, "file-001")

        wrong = self.root / "wrong.pdf"
        with pymupdf.open() as document:
            page = document.new_page()
            page.insert_text((50, 70), "SYNTHETIC OTHER")
            page.insert_text((50, 800), "1.0 Internal")
            document.save(wrong)
        with self.assertRaisesRegex(v.CheckError, "pdf_cover_title"):
            v.check_pdf(wrong, {"chunks": []}, META, renders, "file-002")

        later = self.root / "later.pdf"
        with pymupdf.open() as document:
            page = document.new_page()
            page.insert_text((50, 70), "Cover")
            page.insert_text((50, 800), "1.0 Internal")
            document.new_page().insert_text((50, 70), "SYNTHETIC POLICY")
            document.save(later)
        with self.assertRaisesRegex(v.CheckError, "pdf_cover_title"):
            v.check_pdf(later, {"chunks": []}, META, renders, "file-003")

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

    def test_controlled_failure_and_continuation_have_source_owned_dates(self):
        self.negative_fixture(false_success=False)

    def test_controlled_failure_rejects_date_error_as_wrong_cause(self):
        with self.assertRaisesRegex(v.CheckError, "controlled_failure_wrong_cause"):
            self.negative_fixture(false_success=False, error=
                                  "conversion template {DATE} requires last_modified as a real calendar date in YYYY-MM-DD format")

    def test_controlled_failure_cannot_claim_success_hash(self):
        with self.assertRaisesRegex(v.CheckError, "controlled_failure_false_success"):
            self.negative_fixture(false_success=True)

    def negative_fixture(self, false_success, error="Markdown image rejected: synthetic missing image"):
        output = self.root / "existing"
        output.mkdir()
        def fake_cli(_dms, _config, source, target):
            if target == output:
                return subprocess.CompletedProcess([], 1, b"", b"output root already exists; overwrite and resume are not supported")
            # Check the generated sources before mocking conversion: {DATE}
            # templates must reach the image error and the valid companion.
            for name in ("a-failure.md", "b-success.md"):
                fields, body = v.source_fields((source / name).read_text(encoding="utf-8"))
                self.assertIn("last_modified", fields, name)
                self.assertRegex(fields["last_modified"], r"^\d{4}-\d{2}-\d{2}$")
                self.assertEqual(date.fromisoformat(fields["last_modified"]).isoformat(), fields["last_modified"])
                self.assertEqual("![missing](missing.png)" in body, name == "a-failure.md")
            self.assertFalse((source / "missing.png").exists())
            target.mkdir()
            value = self.manifest()
            bad = copy.deepcopy(value["results"][0])
            bad.update(input="a-failure.md", input_sha256=v.file_sha(source / "a-failure.md"),
                       status="failed", error=error, word_version=None)
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
