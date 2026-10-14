# /// script
# requires-python = ">=3.11"
# dependencies = ["markdown-it-py==4.0.0", "mdit-py-plugins==0.5.0", "PyMuPDF==1.26.5"]
# ///
"""Independent smoke verifier; see Test-MarkdownBatchExport.ps1 for operation.

Only synthetic tests may run on Linux. Production entry refuses non-Windows.
No DMS modules are imported. Raw subprocess diagnostics and parser exceptions
never reach stdout or evidence. XML checks are package/relationship health,
not full ECMA XSD validation; PDF text/geometry is not visual sign-off.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import posixpath
import re
import stat
import subprocess
import sys
import unicodedata
import xml.etree.ElementTree as ET
import zipfile
from collections import Counter
from pathlib import Path, PurePosixPath
from urllib.parse import unquote, urlsplit
from uuid import UUID

import pymupdf
from markdown_it import MarkdownIt
from mdit_py_plugins.footnote import footnote_plugin

# Parser diagnostics can include document paths; only our fixed codes are public.
pymupdf.TOOLS.mupdf_display_errors(False)
pymupdf.TOOLS.mupdf_display_warnings(False)

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
A = "http://schemas.openxmlformats.org/drawingml/2006/main"
WP = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
REL = "http://schemas.openxmlformats.org/package/2006/relationships"
CT = "http://schemas.openxmlformats.org/package/2006/content-types"
NS = {"w": W, "r": R, "a": A, "wp": WP}
MANIFEST_KEYS = {"conversion_only", "dry_run", "without_toc", "renderer_version",
                 "configuration_schema", "template", "results"}
RESULT_KEYS = {"input", "input_sha256", "assets", "metadata", "docx", "pdf",
               "word_version", "status", "error", "retained_diagnostics"}
TOKEN = re.compile(r"\{[A-Z][A-Z0-9_ ]*\}")
FIELD_ERROR = re.compile(r"Error!|Fehler!|No table of contents entries|Keine Inhaltsverzeichniseintr", re.IGNORECASE)
NUMBERED = re.compile(r"^\s*\d+(?:\.\d+)*[.)]?\s+\S")


class CheckError(Exception):
    """Only fixed machine codes, never source/path/exception text."""


def require(ok, code):
    if not ok:
        raise CheckError(code)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def compact(text):
    return "".join(unicodedata.normalize("NFKC", text).replace("\u00ad", "").split())


def fidelity_text(text):
    # Compare visible text, not typographic spelling: pulldown-cmark enables
    # smart punctuation, whereas code spans and literal HTML stay unchanged.
    # Do not enable markdown-it's broader (c)/(tm)/etc replacement rules.
    return compact(text.translate(str.maketrans({"‘": "'", "’": "'", "“": '"', "”": '"',
                                                "–": "--", "—": "---"})))


def linked(path):
    info = path.lstat()
    return stat.S_ISLNK(info.st_mode) or bool(getattr(info, "st_file_attributes", 0) & 0x400)


def ordinary(path):
    require(path.is_file() and not linked(path), "non_regular_file")
    return path


def canonical(path):
    # Windows config roots may use verbatim paths while argv uses Win32 paths.
    # Normalize only after filesystem resolution, before containment comparisons.
    resolved = path.resolve(strict=True)
    text = str(resolved)
    if text.startswith("\\\\?\\UNC\\"):
        return Path("\\\\" + text[8:])
    if text.startswith("\\\\?\\"):
        return Path(text[4:])
    return resolved


def snapshot(root):
    """Hash bytes and enumerate directories, including hidden .dms state.

    Keep paths only in memory. Refuse links/reparse points, never follow them.
    """
    require(root.exists() and not linked(root), "snapshot_root_unavailable")
    if root.is_file():
        return {".": (root.stat().st_size, file_sha(root))}
    result = {".": "directory"}
    def unreadable(_error):
        raise CheckError("snapshot_unreadable_subtree")

    for directory, dirs, files in os.walk(root, followlinks=False, onerror=unreadable):
        for name in sorted(dirs + files):
            path = Path(directory) / name
            require(not linked(path), "snapshot_reparse_point")
            key = path.relative_to(root).as_posix()
            if path.is_dir():
                result[key] = "directory"
            else:
                ordinary(path)
                result[key] = (path.stat().st_size, file_sha(path))
    return result


def snapshot_digest(value):
    return sha(json.dumps(value, sort_keys=True, ensure_ascii=True).encode())


def overlaps(left, right):
    return left == right or left in right.parents or right in left.parents


def fresh(path):
    require(not os.path.lexists(path), "output_already_exists")
    require(path.parent.is_dir(), "output_parent_missing_create_it_explicitly")
    return canonical(path.parent) / path.name


def relative_file(root, name):
    require(isinstance(name, str) and "\\" not in name and ":" not in name,
            "unsafe_manifest_path")
    parts = PurePosixPath(name)
    require(name and not parts.is_absolute() and all(p not in (".", "..", "") for p in name.split("/")),
            "unsafe_manifest_path")
    path = root.joinpath(*parts.parts)
    for parent in [path, *path.parents]:
        if parent == root:
            break
        require(not linked(parent), "output_reparse_point")
    require(canonical(path).is_relative_to(root), "artifact_outside_root")
    return ordinary(path)


def load_json(path):
    def unique(pairs):
        obj = {}
        for key, value in pairs:
            require(key not in obj, "duplicate_json_key")
            obj[key] = value
        return obj
    return json.loads(path.read_text(encoding="utf-8-sig"), object_pairs_hook=unique)


def source_fields(text):
    # The product uses flat literal scalars, NOT YAML typing/unescaping.
    lines = text.lstrip("\ufeff").splitlines(keepends=True)
    require(lines and lines[0].rstrip("\r\n") == "---", "frontmatter_missing")
    fields = {}
    for index, raw in enumerate(lines[1:], 1):
        line = raw.rstrip("\r\n")
        if line in ("---", "..."):
            return fields, "".join(lines[index + 1:])
        if not line.strip():
            continue
        require(not line[0].isspace() and ":" in line, "frontmatter_not_flat")
        key, value = (x.strip() for x in line.split(":", 1))
        require(key not in fields and key and not any(x.isspace() for x in key), "frontmatter_key")
        require(value and value[0] not in "[{|>&*", "frontmatter_scalar")
        if value[0] in "\"'" or value[-1] in "\"'":
            require(len(value) >= 2 and value[0] == value[-1], "frontmatter_quote")
            value = value[1:-1]
        require(value, "frontmatter_empty")
        fields[key] = value
    raise CheckError("frontmatter_unclosed")


def select_policies(source, manifest_path, expected_count):
    """Reconcile an operator-reviewed JSON filename array with CLI discovery.

    The CLI cannot select a subset: refuse extras rather than convert an
    unreviewed README or silently stage/copy a different input tree.
    """
    names = load_json(ordinary(manifest_path))
    require(isinstance(names, list) and all(isinstance(n, str) and n and
            not any(c in n for c in "/\\:") and PurePosixPath(n).suffix.lower() == ".md"
            for n in names), "policy_selection_manifest_invalid")
    require(len(names) == expected_count and len({n.casefold() for n in names}) == len(names),
            "selected_count_or_case_collision")
    discovered = sorted(p.name for p in source.iterdir() if p.is_file() and p.suffix.lower() == ".md")
    require(discovered == sorted(names), "policy_selection_not_reconciled")
    for name in names:
        relative_file(source, name)
    return sorted(names)


def inline_text(tokens):
    parts, links = [], []
    for token in tokens:
        if token.type in ("text", "code_inline", "html_inline"):
            parts.append(token.content)
        elif token.type in ("softbreak", "hardbreak"):
            parts.append(" ")
        elif token.type == "link_open":
            links.append(token.attrGet("href"))
        elif token.type == "link_close":
            parts.append(" (" + links.pop() + ")")
        elif token.type == "footnote_ref":
            parts.append("[^" + token.meta["label"] + "]")
    return "".join(parts)


def source_inventory(path, root):
    data = path.read_bytes()
    fields, body = source_fields(data.decode("utf-8-sig"))
    tokens = MarkdownIt("commonmark").enable(["table", "strikethrough"]).use(footnote_plugin).parse(body)
    # Heading attributes and task markers are syntax, not body text. Operate
    # on parsed inline text only so fenced/code/HTML literals remain intact.
    for index, token in enumerate(tokens):
        if token.type != "inline" or not token.children:
            continue
        children = token.children
        if tokens[index - 1].type == "heading_open" and children[-1].type == "text":
            children[-1].content = re.sub(r"\s+\{(?:[#.][^\s{}]+\s*)+\}$", "", children[-1].content)
        if (index >= 2 and tokens[index - 1].type == "paragraph_open"
                and tokens[index - 2].type == "list_item_open" and children[0].type == "text"):
            children[0].content = re.sub(r"^\[X\](?=\s)", "[x]", children[0].content)
    images, chunks, headings, tables, history = [], [], [], [], []
    current_table = None
    row = None
    history_next = False
    for index, token in enumerate(tokens):
        if token.type == "heading_open":
            text = inline_text(tokens[index + 1].children or [])
            headings.append((int(token.tag[1:]), text))
            history_next = compact(text).casefold().endswith("versionsverlauf")
        if token.type == "table_open":
            current_table = []
        elif token.type == "tr_open":
            row = []
        elif token.type == "tr_close":
            current_table.append(row)
        elif token.type == "table_close":
            tables.append(current_table)
            if history_next:
                history.append(current_table)
                history_next = False
            current_table = None
        elif token.type == "inline":
            children = token.children or []
            text = inline_text(children)
            if text.strip():
                chunks.append(text)
            if current_table is not None and tokens[index - 1].type in ("td_open", "th_open"):
                row.append(text)
            for child in children:
                require(child.type != "html_inline" or not re.search(r"<img\b", child.content, re.IGNORECASE),
                        "html_image_not_supported")
                if child.type != "image":
                    continue
                url = urlsplit(child.attrGet("src"))
                require(not url.scheme and not url.netloc and not url.query and not url.fragment,
                        "unsupported_source_image_url")
                name = unquote(url.path)
                require(not Path(name).is_absolute(), "absolute_source_image")
                asset = canonical(path.parent / name)
                require(asset.is_relative_to(root), "source_image_escape")
                ordinary(asset)
                images.append((asset.relative_to(root).as_posix(), file_sha(asset)))
        elif token.type in ("html_block", "html_inline"):
            require(not re.search(r"<img\b", token.content, re.IGNORECASE), "html_image_not_supported")
            if token.content.strip():
                chunks.append(token.content)
        elif token.type in ("fence", "code_block") and token.content.strip():
            chunks.append(token.content)
    require(len(history) == 1 and history[0] and all(len(r) == 5 for r in history[0]),
            "five_column_versionsverlauf_missing")
    return {"sha256": sha(data), "fields": fields, "images": images, "chunks": chunks,
            "headings": headings, "tables": tables, "history": history[0]}


def xml_text(node):
    return "".join(n.text or "" for n in node.iter(f"{{{W}}}t"))


def package(path):
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        require(len(names) == len(set(names)), "docx_duplicate_parts")
        require(archive.testzip() is None, "docx_zip_crc")
        require(all(not n.startswith("/") and ".." not in PurePosixPath(n).parts for n in names),
                "docx_unsafe_part")
        parts = {n: archive.read(n) for n in names}
    xml = {}
    for name, data in parts.items():
        if name.endswith((".xml", ".rels")):
            require(b"<!DOCTYPE" not in data.upper() and b"<!ENTITY" not in data.upper(), "xml_dtd")
            xml[name] = ET.fromstring(data)
    for name, tag in {"word/document.xml": f"{{{W}}}document", "[Content_Types].xml": f"{{{CT}}}Types",
                      "_rels/.rels": f"{{{REL}}}Relationships"}.items():
        require(name in xml and xml[name].tag == tag, "docx_required_root")
    require(xml["word/document.xml"].find("w:body", NS) is not None, "docx_body_missing")
    content_types = xml["[Content_Types].xml"]
    defaults = {n.get("Extension", "").lower(): n.get("ContentType") for n in content_types.findall(f"{{{CT}}}Default")}
    overrides = {n.get("PartName", "").lstrip("/"): n.get("ContentType") for n in content_types.findall(f"{{{CT}}}Override")}
    for name in parts:
        if name == "[Content_Types].xml" or name.endswith("/"):
            continue
        require(overrides.get(name) or defaults.get(name.rsplit(".", 1)[-1].lower()), "docx_missing_content_type")
    require(overrides.get("word/document.xml") == "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
            "docx_document_content_type")
    relationships = {}
    for name, node in xml.items():
        if not name.endswith(".rels"):
            continue
        base = "" if name == "_rels/.rels" else posixpath.dirname(posixpath.dirname(name))
        mapping = {}
        for rel in node:
            require(rel.tag == f"{{{REL}}}Relationship" and rel.get("Id") not in mapping,
                    "docx_relationship_invalid")
            if rel.get("TargetMode") == "External":
                require(not rel.get("Type", "").endswith("/image"), "docx_external_image")
                mapping[rel.get("Id")] = None
                continue
            target = posixpath.normpath(posixpath.join(base, unquote(rel.get("Target", "")))).lstrip("/")
            require(target in parts, "docx_relationship_missing_target")
            mapping[rel.get("Id")] = target
        relationships[name] = mapping
    drawing_hashes = Counter()
    body_drawings = Counter()
    for name, node in xml.items():
        if not name.startswith("word/") or not name.endswith(".xml"):
            continue
        rel_name = posixpath.join(posixpath.dirname(name), "_rels", posixpath.basename(name) + ".rels")
        for blip in node.findall(".//a:blip", NS):
            require(blip.get(f"{{{R}}}link") is None, "docx_linked_image")
            target = relationships.get(rel_name, {}).get(blip.get(f"{{{R}}}embed"))
            require(target is not None, "docx_image_relationship_missing")
            digest = sha(parts[target])
            drawing_hashes[digest] += 1
            if name == "word/document.xml":
                body_drawings[digest] += 1
    return {"parts": parts, "xml": xml, "drawings": drawing_hashes, "body_drawings": body_drawings,
            "relationships": relationships}


def footer_profile(template):
    """Controlled footer stories, including inherited section references.

    Headers may be logos/static text. A distinct cover story may omit either
    controlled field; subsequent PDF pages must carry both. More complex
    pagination still needs the mandatory rendered-page review.
    """
    xml = template["xml"]
    rels = template["relationships"].get("word/_rels/document.xml.rels", {})
    inherited, stories, cover_fields = {}, {}, set()
    even = xml.get("word/settings.xml")
    even = None if even is None else even.find("w:evenAndOddHeaders", NS)
    even = even is not None and even.get(f"{{{W}}}val", "true") not in ("0", "false", "off")
    for index, section in enumerate(xml["word/document.xml"].findall(".//w:sectPr", NS)):
        for ref in section.findall("w:footerReference", NS):
            target = rels.get(ref.get(f"{{{R}}}id"))
            require(target in xml, "template_footer_relationship_missing")
            inherited[ref.get(f"{{{W}}}type")] = target
        title = section.find("w:titlePg", NS)
        first = title is not None and title.get(f"{{{W}}}val", "true") not in ("0", "false", "off")
        active = {"default"} | ({"first"} if first else set()) | ({"even"} if even else set())
        for kind in active:
            target = inherited.get(kind)
            node = xml.get(target)
            fields = set()
            if node is not None:
                text = xml_text(node)
                instructions = " ".join(n.text or "" for n in node.findall(".//w:instrText", NS))
                instructions += " " + " ".join(n.get(f"{{{W}}}instr", "") for n in node.findall(".//w:fldSimple", NS))
                for key in ("VERSION", "CONFIDENTIALITY"):
                    if "{" + key + "}" in text or re.search(r"\bDMS_" + key + r"\b", instructions):
                        fields.add(key)
                if fields:
                    stories[target] = fields
            if index == 0 and kind == ("first" if first else "default"):
                cover_fields = fields
    require(set().union(*stories.values()) == {"VERSION", "CONFIDENTIALITY"}, "template_controlled_footer_missing")
    return {"stories": stories, "cover_required": bool(cover_fields), "cover_fields": cover_fields}


def metadata_for(inventory, config):
    fields = inventory["fields"]
    item = config["confidentiality_types"][fields["confidentiality"]]
    require(item["enabled"] is True and item["id"] == fields["confidentiality"], "classification_disabled")
    require(re.fullmatch(r"\d+\.\d+", fields["version"], re.ASCII) is not None, "source_version")
    return {"title": fields["title"], "version": fields["version"],
            "document_number": fields.get("document_number") if fields.get("document_number", "").strip() else None,
            "confidentiality": {"type_id": item["id"], "label": item["label"]}}


def check_docx(doc, template, inventory, metadata):
    xml = doc["xml"]
    body = xml["word/document.xml"]
    visible = "\n".join(xml_text(node) for node in xml.values())
    require(not TOKEN.search(visible) and not FIELD_ERROR.search(visible), "docx_unresolved_field")
    values = {"DMS_TITLE": metadata["title"], "DMS_VERSION": metadata["version"],
              "DMS_DOCUMENT_NUMBER": metadata["document_number"] or "",
              "DMS_CONFIDENTIALITY": metadata["confidentiality"]["label"]}
    custom = xml.get("docProps/custom.xml")
    require(custom is not None, "docx_custom_properties_missing")
    actual = {p.get("name"): "".join(p.itertext()) for p in custom}
    require(all(actual.get(k) == v for k, v in values.items()), "docx_controlled_properties")
    profile = footer_profile(template)
    for name, fields in profile["stories"].items():
        require(name in xml and all(compact(values["DMS_" + key]) in compact(xml_text(xml[name]))
                                   for key in fields), "docx_controlled_footer")
    body_text = fidelity_text(xml_text(body))
    require(all(fidelity_text(t) in body_text for t in inventory["chunks"]), "docx_source_text_missing")
    tables = []
    for table in body.findall(".//w:tbl", NS):
        rows = [[fidelity_text(xml_text(cell)) for cell in row.findall("w:tc", NS)]
                for row in table.findall("w:tr", NS)]
        tables.append(rows)
    require(all([[fidelity_text(c) for c in row] for row in expected] in tables for expected in inventory["tables"]),
            "docx_table_cell_text_or_order")
    source_drawings = Counter(digest for _, digest in inventory["images"])
    require(doc["drawings"] == template["drawings"] + source_drawings, "docx_source_template_drawing_accounting")
    require(doc["body_drawings"] == template["body_drawings"] + source_drawings, "docx_source_drawing_location")
    template_media = Counter(sha(data) for name, data in template["parts"].items() if name.startswith("word/media/"))
    media = Counter(sha(data) for name, data in doc["parts"].items() if name.startswith("word/media/"))
    require(not (template_media - media), "docx_template_media_changed")
    sections = body.findall(".//w:sectPr", NS)
    require(sections, "docx_section_missing")
    size, margin = sections[-1].find("w:pgSz", NS), sections[-1].find("w:pgMar", NS)
    require(size is not None and margin is not None, "docx_page_geometry_missing")
    width = int(size.get(f"{{{W}}}w")) - sum(int(margin.get(f"{{{W}}}{key}", "0")) for key in ("left", "right", "gutter"))
    require(width > 0, "docx_page_geometry_invalid")
    for extent in body.findall(".//wp:extent", NS):
        require(0 < int(extent.get("cx")) <= width * 635 and int(extent.get("cy")) > 0,
                "docx_drawing_outside_printable_width")
    for table in body.findall(".//w:tbl", NS):
        rows = [[fidelity_text(xml_text(c)) for c in r.findall("w:tc", NS)] for r in table.findall("w:tr", NS)]
        if rows not in [[[fidelity_text(c) for c in r] for r in t] for t in inventory["tables"]]:
            continue
        layout = table.find("w:tblPr/w:tblLayout", NS)
        tw = table.find("w:tblPr/w:tblW", NS)
        grid = table.findall("w:tblGrid/w:gridCol", NS)
        require(layout is not None and layout.get(f"{{{W}}}type") == "fixed" and tw is not None
                and tw.get(f"{{{W}}}type") == "dxa" and int(tw.get(f"{{{W}}}w")) == width
                and sum(int(c.get(f"{{{W}}}w")) for c in grid) == width, "docx_table_width")
    toc = [s for s in body.findall(".//w:sdt", NS)
           if any(n.get(f"{{{W}}}val") == "Table of Contents" for n in s.findall(".//w:docPartGallery", NS))]
    require(toc and any("TOC" in "".join(n.itertext()) for s in toc for n in s.findall(".//w:instrText", NS)),
            "docx_toc_field_missing")
    require(any(re.search(r"\d+\s*$", xml_text(p)) for s in toc for p in s.findall(".//w:p", NS)),
            "docx_toc_cached_page_numbers_missing")
    # A numbered source heading plus effective Word numbering is duplication,
    # even though the automatic prefix is absent from w:t XML text.
    styles = xml.get("word/styles.xml")
    style_map = {} if styles is None else {s.get(f"{{{W}}}styleId"): s for s in styles.findall("w:style", NS)}
    def numbered(node, seen=None):
        seen = set() if seen is None else seen
        num = node.find("w:pPr/w:numPr/w:numId", NS)
        if num is not None:
            return num.get(f"{{{W}}}val") != "0"
        link = node.find("w:pPr/w:pStyle", NS) if node.tag == f"{{{W}}}p" else node.find("w:basedOn", NS)
        key = None if link is None else link.get(f"{{{W}}}val")
        if key in style_map and key not in seen:
            return numbered(style_map[key], seen | {key})
        return False
    numbered_sources = {fidelity_text(text) for _, text in inventory["headings"] if NUMBERED.match(text)}
    require(not any(fidelity_text(xml_text(p)) in numbered_sources and numbered(p) for p in body.findall(".//w:p", NS)),
            "duplicate_heading_numbering")
    return {"template_drawings": sum(template["drawings"].values()),
            "source_references": len(inventory["images"]), "source_drawings": sum(source_drawings.values()),
            "source_unique_assets": len(set(inventory["images"])),
            "history_rows": len(inventory["history"]), "history_columns": 5,
            "tables": len(inventory["tables"]), "heading_convention": "no_source_plus_word_double_numbering"}


def check_pdf(path, inventory, metadata, render_dir, file_id, profile=None):
    require(path.read_bytes().rstrip().endswith(b"%%EOF"), "pdf_incomplete")
    with pymupdf.open(path) as pdf:
        require(not pdf.is_encrypted and not pdf.is_repaired and pdf.page_count > 0, "pdf_invalid_or_repaired")
        pages, footer_pages = [], []
        for number, page in enumerate(pdf, 1):
            text = page.get_text("text", sort=True)
            require(text.strip() and not TOKEN.search(text) and not FIELD_ERROR.search(text), "pdf_empty_or_unresolved_field")
            # Text remains private; expose only page indices and character counts.
            for word in page.get_text("words"):
                require((page.rect + (-1, -1, 1, 1)).contains(pymupdf.Rect(word[:4])), "pdf_text_outside_page")
            for image in page.get_image_info():
                require((page.rect + (-1, -1, 1, 1)).contains(pymupdf.Rect(image["bbox"])), "pdf_image_outside_page")
            chrome = page.get_text(clip=pymupdf.Rect(0, page.rect.height * .75, page.rect.width, page.rect.height))
            values = {"VERSION": metadata["version"], "CONFIDENTIALITY": metadata["confidentiality"]["label"]}
            fields = set(values)
            if number == 1 and profile is not None:
                fields = profile.get("cover_fields", set(values) if profile["cover_required"] else set())
            if fields:
                require(all(compact(values[key]) in compact(chrome) for key in fields), "pdf_footer_controlled_chrome")
                footer_pages.append(number)
            if number == 1:
                require(compact(metadata["title"]) in compact(text), "pdf_cover_title")
            page.get_pixmap(dpi=120, alpha=False).save(render_dir / f"{file_id}-page-{number:03}.png")
            pages.append(text)
        all_text = fidelity_text("\n".join(pages))
        require(all(fidelity_text(chunk) in all_text for chunk in inventory["chunks"]), "pdf_source_text_missing")
        return {"pdf_pages": len(pages), "pdf_text_characters": [len(t) for t in pages],
                "rendered_pages": list(range(1, len(pages) + 1)), "footer_checked_pages": footer_pages,
                "visual_review": "required"}


def manifest_check(manifest, selected, config):
    require(set(manifest) == MANIFEST_KEYS, "manifest_schema_or_release_evidence")
    require(manifest["conversion_only"] is True and manifest["dry_run"] is False
            and manifest["without_toc"] is False, "manifest_not_conversion_only")
    actual, expected = manifest["template"], config["markdown_template"]
    require(set(actual) == {"id", "relative_path", "sha256", "contract_version"}
            and UUID(actual["id"]) == UUID(expected["id"])
            and all(actual[key] == expected[key] for key in ("relative_path", "sha256", "contract_version"))
            and manifest["configuration_schema"] == config["schema_version"],
            "manifest_configuration_template_mismatch")
    require(re.fullmatch(r"dms-core/\d+\.\d+\.\d+", manifest["renderer_version"]) is not None, "manifest_renderer")
    results = manifest["results"]
    require(len(results) == len(selected) and Counter(r["input"] for r in results) == Counter(selected), "manifest_selection")
    require(all(set(r) == RESULT_KEYS for r in results), "result_schema_or_release_evidence")
    require(all(r["status"] in ("succeeded", "failed") for r in results), "result_status_invalid")
    return {r["input"]: r for r in results}


def run_cli(dms, config, source, output):
    # Native executable receives argv, no shell expansion; stderr includes source
    # names and must never be relayed or persisted in sanitized evidence.
    return subprocess.run([str(dms), "--json", "convert", "markdown", "--workspace-config", str(config),
                           "--input-root", str(source), "--output-root", str(output)],
                          capture_output=True, check=False)


def check_run(output, process, selected, inventories, config, template, evidence_root):
    require(output.is_dir() and not linked(output), "cli_no_output_manifest")
    manifest = load_json(relative_file(output, "manifest.json"))
    require(json.loads(process.stdout) == manifest, "cli_manifest_stdout_mismatch")
    results = manifest_check(manifest, selected, config)
    evidence, allowed = [], {"manifest.json"}
    for index, name in enumerate(selected, 1):
        item = results[name]
        inv = inventories[name]
        receipt = {"file_id": f"file-{index:03}", "source_path_sha256": sha(name.encode()),
                   "input_sha256": inv["sha256"], "cli_status": item["status"]}
        evidence.append(receipt)
        try:
            require(item["status"] == "succeeded", "cli_document_failed")
            require(item["error"] is None and item["retained_diagnostics"] == [] and item["word_version"], "success_with_error_or_diagnostics")
            require(re.fullmatch(r"[0-9]+(?:\.[0-9]+)*", item["word_version"]) is not None, "word_version_invalid")
            receipt["word_version"] = item["word_version"]
            require(item["input_sha256"] == inv["sha256"], "manifest_input_digest")
            require(Counter((a["relative_path"], a["sha256"]) for a in item["assets"]) == Counter(set(inv["images"])),
                    "manifest_asset_digests")
            receipt["asset_sha256"] = sorted({digest for _, digest in inv["images"]})
            metadata = metadata_for(inv, config)
            require(item["metadata"] == metadata, "manifest_controlled_metadata")
            paths = {}
            for extension in ("docx", "pdf"):
                artifact = item[extension]
                expected = str(PurePosixPath(name).with_suffix("." + extension))
                require(isinstance(artifact, dict) and set(artifact) == {"relative_path", "sha256"}
                        and artifact["relative_path"] == expected, "manifest_artifact_path")
                allowed.add(expected)
                paths[extension] = relative_file(output, expected)
                digest = file_sha(paths[extension])
                require(digest == artifact["sha256"], "manifest_final_artifact_digest")
                receipt[extension + "_sha256"] = digest
            receipt.update(check_docx(package(paths["docx"]), template, inv, metadata))
            receipt.update(check_pdf(paths["pdf"], inv, metadata, evidence_root, receipt["file_id"], footer_profile(template)))
            receipt["verified"] = True
        except CheckError as error:
            receipt.update(verified=False, failure=str(error))
        except Exception:  # noqa: BLE001 -- sanitize parser/native exception text
            receipt.update(verified=False, failure="artifact_parse_or_io_failure")
    children = list(output.iterdir())
    files = {p.relative_to(output).as_posix() for p in children if p.is_file()}
    return {"cli_exit": process.returncode, "selected": len(selected),
            "succeeded": sum(r["status"] == "succeeded" for r in results.values()),
            "failed": sum(r["status"] == "failed" for r in results.values()),
            "verified": sum(e["verified"] for e in evidence), "files": evidence,
            "renderer_version": manifest["renderer_version"],
            "no_unexpected_artifacts": files == allowed and all(p.is_file() and not linked(p) for p in children)}


def negative_probes(dms, config_path, config, source, output, evidence_root):
    before = snapshot(output)
    refused = run_cli(dms, config_path, source, output)
    require(refused.returncode != 0 and b"output root already exists; overwrite and resume are not supported" in refused.stderr
            and snapshot(output) == before, "existing_output_not_refused_unchanged")
    # Missing local image is a document-level error after valid metadata; no real
    # policy is copied or modified. A companion valid source proves continuation.
    fixture = evidence_root / "synthetic-input"
    fixture.mkdir()
    classification = next(k for k, v in config["confidentiality_types"].items() if v["enabled"])
    front = f"---\ntitle: Synthetic smoke policy\nversion: 1.0\nconfidentiality: {classification}\n---\n"
    (fixture / "a-failure.md").write_text(front + "# Synthetic failure\n![missing](missing.png)\n", encoding="utf-8")
    (fixture / "b-success.md").write_text(front + "# Synthetic continuation\nSynthetic verification text.\n", encoding="utf-8")
    fixture_before = snapshot(fixture)
    target = evidence_root / "synthetic-output"
    process = run_cli(dms, config_path, fixture, target)
    require(process.returncode != 0, "controlled_failure_exit_zero")
    manifest = load_json(target / "manifest.json")
    require(json.loads(process.stdout) == manifest, "failure_stdout_manifest_mismatch")
    results = manifest_check(manifest, ["a-failure.md", "b-success.md"], config)
    bad, good = results["a-failure.md"], results["b-success.md"]
    require(bad["status"] == "failed" and bad["error"] and bad["docx"] is None and bad["pdf"] is None
            and bad["word_version"] is None and not bad["retained_diagnostics"], "controlled_failure_false_success")
    require(bad["input_sha256"] == file_sha(fixture / "a-failure.md")
            and bad["error"].startswith("Markdown image rejected:"), "controlled_failure_wrong_cause")
    require(not (target / "a-failure.docx").exists() and not (target / "a-failure.pdf").exists(), "controlled_failure_partial_pair")
    require(good["status"] == "succeeded" and good["error"] is None and good["word_version"]
            and not good["retained_diagnostics"], "controlled_failure_no_continuation")
    for kind in ("docx", "pdf"):
        artifact = good[kind]
        require(artifact["relative_path"] == "b-success." + kind
                and file_sha(relative_file(target, artifact["relative_path"])) == artifact["sha256"], "continuation_pair_digest")
    package(target / "b-success.docx")
    with pymupdf.open(target / "b-success.pdf") as pdf:
        require(not pdf.is_repaired and pdf.page_count > 0 and "Syntheticverificationtext." in compact("".join(p.get_text() for p in pdf)),
                "continuation_pdf_invalid")
    require(snapshot(fixture) == fixture_before, "synthetic_input_changed")
    return {"rerun_exit": refused.returncode, "rerun_unchanged": True,
            "controlled_failure_exit": process.returncode, "controlled_failed": 1, "continued_success": 1}


def execute(args):
    require(sys.platform == "win32", "native_windows_required_no_wsl_fallback")
    config_path = canonical(ordinary(Path(args.workspace_config)))
    source = canonical(Path(args.input_root))
    output = fresh(Path(args.output_root).absolute())
    repeat = fresh(output.with_name(output.name + "-repeat"))
    evidence_root = fresh(output.with_name(output.name + "-evidence"))
    dms = canonical(ordinary(Path(args.dms)))
    policy_manifest = canonical(ordinary(Path(args.policy_manifest)))
    config = load_json(config_path)
    edit, publish = (canonical(Path(config[k])) for k in ("edit_root", "publish_root"))
    require(config_path == edit / ".dms" / "workspace.json", "config_edit_root_mismatch")
    template_path = relative_file(edit, config["markdown_template"]["relative_path"])
    require(file_sha(template_path) == config["markdown_template"]["sha256"], "template_digest_mismatch")
    protected = {"input": source, "edit": edit, "publish": publish, "configuration": config_path,
                 "template": template_path, "policy_manifest": policy_manifest}
    require(not overlaps(source, edit) and not overlaps(source, publish) and not overlaps(edit, publish), "protected_roots_overlap")
    for destination in (output, repeat, evidence_root):
        require(all(not overlaps(destination, root) for root in protected.values()), "destination_overlaps_protected_tree")
    selected = select_policies(source, policy_manifest, args.expected_count)
    before = {key: snapshot(root) for key, root in protected.items()}
    template = package(template_path)
    evidence = {"conversion_only": True, "g4_complete": False, "visual_review": "required",
                "selected": len(selected), "snapshots": {k: snapshot_digest(v) for k, v in before.items()},
                "configuration_sha256": file_sha(config_path), "template_sha256": file_sha(template_path),
                "policy_manifest_sha256": file_sha(policy_manifest),
                "selected_set_sha256": sha(json.dumps(selected, separators=(",", ":"), ensure_ascii=True).encode()),
                "selection": [{"file_id": f"file-{i:03}", "source_path_sha256": sha(name.encode()),
                               "input_sha256": file_sha(source / name)} for i, name in enumerate(selected, 1)],
                "cli_sha256": file_sha(dms), "runs": [], "automated_pass": False}
    evidence_root.mkdir()
    try:
        inventories = {name: source_inventory(source / name, source) for name in selected}
        evidence["source_image_references"] = sum(len(v["images"]) for v in inventories.values())
        evidence["policies_with_images"] = sum(bool(v["images"]) for v in inventories.values())
        for index, destination in enumerate((output, repeat), 1):
            renders = evidence_root / f"run-{index}-pages"
            renders.mkdir()
            process = run_cli(dms, config_path, source, destination)
            result = check_run(destination, process, selected, inventories, config, template, renders)
            evidence["runs"].append(result)
            require(result["cli_exit"] == 0 and result["verified"] == len(selected)
                    and result["no_unexpected_artifacts"], "conversion_or_independent_verification_failed")
        evidence["negative_probes"] = negative_probes(dms, config_path, config, source, output, evidence_root)
        evidence["automated_pass"] = True
    except CheckError as error:
        evidence["failure"] = str(error)
    except Exception:  # noqa: BLE001 -- sanitize parser/native exception text
        evidence["failure"] = "verification_parse_or_io_failure"
    finally:
        try:
            after = {key: snapshot(root) for key, root in protected.items()}
            evidence["unchanged"] = {k: before[k] == after[k] for k in before}
            require(all(evidence["unchanged"].values()), "protected_tree_changed_no_release_safety_proof")
            evidence["no_dms_release_evidence"] = all(r["no_unexpected_artifacts"] for r in evidence["runs"])
        except Exception:  # noqa: BLE001 -- unreadable snapshot must fail closed
            evidence["automated_pass"] = False
            evidence["failure"] = "protected_tree_changed_or_unreadable"
        with (evidence_root / "evidence.json").open("x", encoding="utf-8") as stream:
            json.dump(evidence, stream, indent=2, ensure_ascii=True)
    # This object contains only generated identifiers/counts/hashes/static codes.
    print(json.dumps(evidence, ensure_ascii=True))
    return 0 if evidence["automated_pass"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("workspace-config", "input-root", "output-root", "dms", "policy-manifest"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--expected-count", required=True, type=int)
    args = parser.parse_args()
    try:
        require(args.expected_count > 0, "expected_count_must_be_positive")
        return execute(args)
    except CheckError as error:
        print(json.dumps({"automated_pass": False, "failure": str(error)}))
    except Exception:  # noqa: BLE001 -- never print config/path/parser contents
        print(json.dumps({"automated_pass": False, "failure": "preflight_parse_or_io_failure"}))
    return 1


if __name__ == "__main__":
    sys.exit(main())
