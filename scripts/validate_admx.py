#!/usr/bin/env python3
"""Validate DMS Desktop ADMX/ADML administrative templates.

Stdlib only. Parses both XML documents and proves the ADMX target namespace,
Computer policy class, exact registry key/value names, and required ADML string
and presentation references. Rejects Windows ADMX dependencies and unpaired or
unused policy elements.
"""

from __future__ import annotations

import argparse
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

EXPECTED_NAMESPACE = "Videoclinic.Policies.DMSDesktop"
EXPECTED_PREFIX = "dmsdesktop"
EXPECTED_POLICY_NAME = "EntraIdentifiers"
EXPECTED_POLICY_CLASS = "Machine"
EXPECTED_KEY = r"SOFTWARE\Policies\Videoclinic\DMS"
EXPECTED_VALUE_NAMES = ("EntraClientId", "EntraTenantId")
STRING_REF = re.compile(r"\$\(string\.([^)]+)\)")
PRESENTATION_REF = re.compile(r"\$\(presentation\.([^)]+)\)")


def _local(tag: str) -> str:
    if tag.startswith("{") and "}" in tag:
        return tag.split("}", 1)[1]
    return tag


def _children(element: ET.Element, name: str) -> list[ET.Element]:
    return [child for child in list(element) if _local(child.tag) == name]


def _child(element: ET.Element, name: str) -> ET.Element | None:
    matches = _children(element, name)
    if not matches:
        return None
    if len(matches) > 1:
        raise ValueError(f"multiple <{name}> elements under <{_local(element.tag)}>")
    return matches[0]


def _require_child(element: ET.Element, name: str) -> ET.Element:
    child = _child(element, name)
    if child is None:
        raise ValueError(f"missing <{name}> under <{_local(element.tag)}>")
    return child


def _load(path: Path) -> ET.Element:
    try:
        tree = ET.parse(path)
    except ET.ParseError as exc:
        raise ValueError(f"{path}: XML parse error: {exc}") from exc
    return tree.getroot()


def _collect_refs(element: ET.Element, pattern: re.Pattern[str]) -> set[str]:
    found: set[str] = set()
    for value in element.attrib.values():
        found.update(pattern.findall(value))
    if element.text:
        found.update(pattern.findall(element.text))
    for child in list(element):
        found.update(_collect_refs(child, pattern))
    return found


def validate(admx_path: Path, adml_path: Path) -> None:
    if adml_path.parent.name != "en-US" or adml_path.name != "DMSDesktop.adml":
        raise ValueError(
            f"ADML must be en-US/DMSDesktop.adml (Intune custom-template import "
            f"accepts one en-US language file); got {adml_path}"
        )

    admx = _load(admx_path)
    adml = _load(adml_path)
    if _local(admx.tag) != "policyDefinitions":
        raise ValueError(f"{admx_path}: root must be <policyDefinitions>")
    if _local(adml.tag) != "policyDefinitionResources":
        raise ValueError(f"{adml_path}: root must be <policyDefinitionResources>")

    namespaces = _require_child(admx, "policyNamespaces")
    target = _require_child(namespaces, "target")
    if target.attrib.get("prefix") != EXPECTED_PREFIX:
        raise ValueError(
            f"ADMX target prefix must be {EXPECTED_PREFIX!r}; got {target.attrib.get('prefix')!r}"
        )
    if target.attrib.get("namespace") != EXPECTED_NAMESPACE:
        raise ValueError(
            f"ADMX target namespace must be {EXPECTED_NAMESPACE!r}; "
            f"got {target.attrib.get('namespace')!r}"
        )
    using = _children(namespaces, "using")
    if using:
        names = [node.attrib.get("namespace", "") for node in using]
        raise ValueError(
            "ADMX must not depend on another template "
            f"(found <using> namespaces: {names})"
        )

    policies_root = _require_child(admx, "policies")
    policies = _children(policies_root, "policy")
    if len(policies) != 1:
        raise ValueError(f"ADMX must define exactly one policy; found {len(policies)}")
    policy = policies[0]
    if policy.attrib.get("name") != EXPECTED_POLICY_NAME:
        raise ValueError(
            f"policy name must be {EXPECTED_POLICY_NAME!r}; got {policy.attrib.get('name')!r}"
        )
    if policy.attrib.get("class") != EXPECTED_POLICY_CLASS:
        raise ValueError(
            f"policy class must be {EXPECTED_POLICY_CLASS!r} (Computer Configuration); "
            f"got {policy.attrib.get('class')!r}"
        )
    if policy.attrib.get("key") != EXPECTED_KEY:
        raise ValueError(
            f"policy key must be {EXPECTED_KEY!r}; got {policy.attrib.get('key')!r}"
        )

    elements_root = _require_child(policy, "elements")
    texts = _children(elements_root, "text")
    if len(texts) != 2:
        raise ValueError(f"policy must have exactly two <text> elements; found {len(texts)}")
    value_names = []
    element_ids = []
    for text in texts:
        element_id = text.attrib.get("id")
        value_name = text.attrib.get("valueName")
        if not element_id or not value_name:
            raise ValueError("each <text> element needs id and valueName")
        if text.attrib.get("required") != "true":
            raise ValueError(f"text element {element_id!r} must be required=\"true\"")
        if element_id != value_name:
            raise ValueError(
                f"text id {element_id!r} must match valueName {value_name!r}"
            )
        element_ids.append(element_id)
        value_names.append(value_name)
    if tuple(value_names) != EXPECTED_VALUE_NAMES:
        raise ValueError(
            f"value names must be {EXPECTED_VALUE_NAMES} in order; got {tuple(value_names)}"
        )

    extra_elements = [
        _local(child.tag) for child in list(elements_root) if _local(child.tag) != "text"
    ]
    if extra_elements:
        raise ValueError(f"policy has unpaired non-text elements: {extra_elements}")

    admx_strings = _collect_refs(admx, STRING_REF)
    admx_presentations = _collect_refs(admx, PRESENTATION_REF)
    if not admx_presentations:
        raise ValueError("policy must reference a presentation")

    resources = _require_child(adml, "resources")
    string_table = _require_child(resources, "stringTable")
    presentation_table = _require_child(resources, "presentationTable")
    adml_strings = {
        node.attrib["id"]: (node.text or "")
        for node in _children(string_table, "string")
        if node.attrib.get("id")
    }
    presentations = {
        node.attrib["id"]: node
        for node in _children(presentation_table, "presentation")
        if node.attrib.get("id")
    }

    missing_strings = sorted(admx_strings - set(adml_strings))
    unused_strings = sorted(set(adml_strings) - admx_strings)
    if missing_strings:
        raise ValueError(f"ADML missing string ids referenced by ADMX: {missing_strings}")
    if unused_strings:
        raise ValueError(f"ADML has unused string ids: {unused_strings}")
    empty_strings = sorted(sid for sid, text in adml_strings.items() if not text.strip())
    if empty_strings:
        raise ValueError(f"ADML string ids have empty text: {empty_strings}")

    missing_presentations = sorted(admx_presentations - set(presentations))
    unused_presentations = sorted(set(presentations) - admx_presentations)
    if missing_presentations:
        raise ValueError(
            f"ADML missing presentation ids referenced by ADMX: {missing_presentations}"
        )
    if unused_presentations:
        raise ValueError(f"ADML has unused presentation ids: {unused_presentations}")

    for presentation_id, presentation in presentations.items():
        ref_ids = [
            child.attrib.get("refId")
            for child in list(presentation)
            if child.attrib.get("refId")
        ]
        if tuple(ref_ids) != tuple(element_ids):
            raise ValueError(
                f"presentation {presentation_id!r} refId order must match policy "
                f"elements {tuple(element_ids)}; got {tuple(ref_ids)}"
            )
        extra_widgets = [
            _local(child.tag)
            for child in list(presentation)
            if not child.attrib.get("refId")
        ]
        if extra_widgets:
            raise ValueError(
                f"presentation {presentation_id!r} has unpaired widgets: {extra_widgets}"
            )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("admx", type=Path)
    parser.add_argument("adml", type=Path)
    args = parser.parse_args(argv)
    try:
        validate(args.admx, args.adml)
    except (OSError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    print(f"OK: {args.admx} + {args.adml}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
