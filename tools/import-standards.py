#!/usr/bin/env python3
"""Index pinned OMG sources without copying their prose or OCL implementations.

Requires pypdf 6.10.0 and Poppler pdftotext 26.01.0 only when regenerating.
Normal CI validates the checked-in catalogue without these tools or downloads.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET
from zipfile import ZipFile

import pypdf

ROOT = Path(__file__).resolve().parents[1]
XMI = "{http://www.omg.org/spec/XMI/20161101}"
SECTION = re.compile(r"^((?:[1-9]\d*|A)(?:\.\d+)*)\s+(.+)$")
KINDS = {
    "Class": "metaclass", "Enumeration": "enumeration",
    "EnumerationLiteral": "enumeration-literal", "Property": "property",
    "Operation": "operation", "Parameter": "parameter",
    "Constraint": "constraint", "Association": "association",
    "Generalization": "generalization", "Package": "package",
    "PackageImport": "package-import",
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def normalize(text):
    return " ".join(text.split())


def facet(standard, section):
    if section.startswith("A"):
        return "annex"
    if section == "2":
        return "conformance"
    if section == "7" or section.startswith("7."):
        return "language-description"
    if standard == "sysml" and section.startswith("8.2.3"):
        return "graphical-notation"
    for prefix, name in [("8.2", "textual-notation"), ("8.3", "abstract-syntax"),
                         ("8.4", "semantics"), ("9", "libraries"), ("10", "interchange")]:
        if section == prefix or section.startswith(prefix + "."):
            return name
    return "context"


def clause_id(standard, section):
    return f"{standard.upper()}-CLAUSE-{section}"


def pdf_index(standard, pdf, manifest):
    reader = pypdf.PdfReader(pdf)
    assert len(reader.pages) == manifest["pages"]
    headings = []
    unnumbered = []

    def visit(items):
        for item in items:
            if isinstance(item, list):
                visit(item)
            else:
                match = SECTION.fullmatch(normalize(item.title))
                if match:
                    headings.append({"section": match[1], "title": match[2],
                                     "page": reader.get_destination_page_number(item) + 1})
                elif headings:
                    unnumbered.append({"title": normalize(item.title),
                                       "page": reader.get_destination_page_number(item) + 1,
                                       "section": headings[-1]["section"]})
    visit(reader.outline)
    by_section = {h["section"]: h for h in headings}
    assert len(by_section) == len(headings), "Duplicate clause bookmarks"
    layout = subprocess.check_output(["pdftotext", "-layout", str(pdf), "-"]).decode("utf8")
    pages = layout.split("\f")
    if not pages[-1].strip():
        pages.pop()
    assert len(pages) == len(reader.pages)

    # Independently reconcile all numbered entries in the printed TOC. Bookmark
    # extraction alone must not silently drop a printed clause.
    toc = set()
    for page in pages[:headings[0]["page"] - 1]:
        for line in page.splitlines():
            match = re.match(r"\s*((?:[1-9]\d*|A)(?:\.\d+)*)\s+.+\.{2,}\s*\d+\s*$", line)
            if match:
                toc.add(match[1])
    assert toc == set(by_section), f"TOC/bookmark mismatch: {toc ^ set(by_section)}"

    items = []
    for i, h in enumerate(headings):
        parent = h["section"].rsplit(".", 1)[0] if "." in h["section"] else None
        assert parent is None or parent in by_section
        following = next((n for n in headings[i + 1:]
                          if not n["section"].startswith(h["section"] + ".")), None)
        items.append({
            "id": clause_id(standard, h["section"]), "standard": standard,
            "kind": "clause", "facet": facet(standard, h["section"]),
            "title": h["title"], "section": h["section"],
            "parent": clause_id(standard, parent) if parent else None,
            "source": standard + "-pdf", "page": h["page"],
            "endPage": following["page"] if following else len(pages),
            "decomposition": "pending",
        })

    current = "1"
    grammar = []
    passages = []
    seen_headings = set()
    blocks = []
    block = []
    block_section = "1"
    block_page = headings[0]["page"]

    def flush():
        nonlocal block
        if block:
            blocks.append((block_section, block_page, normalize(" ".join(block))))
            block = []

    for page_number, page in enumerate(pages, 1):
        if page_number < headings[0]["page"]:
            continue
        for line_number, line in enumerate(page.splitlines(), 1):
            stripped = normalize(line)
            m = SECTION.fullmatch(stripped)
            if m and m[1] in by_section and by_section[m[1]]["page"] == page_number:
                # Exact numbering and start page disambiguate headings from references.
                flush()
                current = m[1]
                seen_headings.add(current)
                continue
            if not stripped:
                flush()
                continue
            if re.search(r"(?:Systems|Kernel) Modeling Language (?:v2\.0|v1\.0)", stripped):
                continue
            if not block:
                block_section, block_page = current, page_number
            block.append(stripped)
            in_grammar = (standard == "sysml" and current.startswith("8.2.2.")
                          and current != "8.2.2.1" and not current.startswith("8.2.2.1.")) or (
                              standard == "kerml" and current.startswith("8.2.")
                              and current != "8.2.1")
            if not in_grammar:
                continue
            header = re.fullmatch(r"\s*([A-Z][A-Za-z0-9_]*)(?:\s*:\s*([A-Z][A-Za-z0-9_]*))?\s*=.*", line)
            # Twelve printed headers omit '='. Preserve that discrepancy explicitly;
            # do not silently repair or omit the production in the inventory.
            exceptions = {("sysml", "PackageMember", 199): "", ("sysml", "DefinitionDeclaration", 201): "",
                          ("sysml", "UsagePrefix", 202): "", ("sysml", "StateSendActionUsage", 217): "",
                          ("kerml", "ImportDeclaration", 107): "",
                          ("sysml", "ConnectorEndMember", 208): ":",
                          ("sysml", "InterfaceEnd", 210): ":",
                          ("sysml", "EntryTransitionMember", 217): ":",
                          ("sysml", "MetadataBodyUsage", 224): ":",
                          ("kerml", "SpecificType", 113): ":",
                          ("kerml", "BasicFeaturePrefix", 115): ":",
                          ("kerml", "PrefixMetadataFeature", 131): ":"}
            missing = re.fullmatch(r"\s*([A-Z][A-Za-z0-9_]*)\s*:\s*([A-Z][A-Za-z0-9_]*)\s*(:?)\s*", line)
            if missing:
                assert exceptions.get((standard, missing[1], page_number)) == missing[3], (
                    f"Unreviewed grammar header exception: {standard}, page {page_number}, {stripped}"
                )
                header = missing
            if header:
                item = {"id": f"{standard.upper()}-GRAMMAR-{header[1]}",
                        "standard": standard, "kind": "grammar-production", "facet": "textual-notation",
                        "title": header[1], "section": current,
                        "parent": clause_id(standard, current), "source": standard + "-pdf",
                        "page": page_number, "line": line_number}
                if missing:
                    item["extractionNote"] = ("Printed production header ends with a colon instead of an equals sign."
                                              if missing[3] else "Printed production header omits the equals sign.")
                grammar.append(item)
        flush()

    assert seen_headings == set(by_section), f"Unlocated clause headings: {set(by_section) - seen_headings}"
    assert len({g["id"] for g in grammar}) == len(grammar), "Duplicate grammar production"
    # Passage locators are review candidates, not an automatic normative-requirement
    # count. Declarative rules without a modal remain covered by clause review.
    for section, page, text in blocks:
        modals = sorted(set(re.findall(r"\b(?:shall|must|should|required)\b", text, re.I)))
        if not modals:
            continue
        digest = sha(text.encode())
        passages.append({"id": f"{standard.upper()}-PROSE-{section}-{digest[:16]}",
                         "standard": standard, "kind": "prose-candidate", "facet": facet(standard, section),
                         "title": f"Modal passage in {section}, page {page}", "section": section,
                         "parent": clause_id(standard, section), "source": standard + "-pdf",
                         "page": page, "textSha256": digest, "modalTerms": modals,
                         "decomposition": "pending"})
    # Identical repeated paragraphs in one clause have one candidate with all pages.
    unique = {}
    for passage in passages:
        if passage["id"] in unique:
            unique[passage["id"]].setdefault("alsoOnPages", []).append(passage["page"])
        else:
            unique[passage["id"]] = passage
    heading_occurrences = Counter()
    for heading in unnumbered:
        section = heading["section"]
        key = f"{standard.upper()}-HEADING-{section}-{sha(heading['title'].encode())[:12]}"
        heading_occurrences[key] += 1
        items.append({"id": f"{key}-{heading_occurrences[key]}",
                      "standard": standard, "kind": "unnumbered-heading", "facet": facet(standard, section),
                      "title": heading["title"], "section": section, "parent": clause_id(standard, section),
                      "source": standard + "-pdf", "page": heading["page"], "decomposition": "pending"})
    items.extend(grammar)
    items.extend(unique.values())
    return items, {"pdfPages": len(pages), "outlineClauses": len(headings),
                   "tocClauses": len(toc), "locatedHeadings": len(seen_headings),
                   "unnumberedHeadings": len(unnumbered),
                   "grammarProductions": len(grammar), "modalPassageCandidates": len(unique)}


def xmi_index(standard, path, clauses):
    root = ET.parse(path).getroot()
    parents = {child: parent for parent in root.iter() for child in parent}
    names = {re.sub(r"\s+", "", c["title"]): c for c in clauses
             if c["kind"] == "clause" and c["section"].startswith("8.3.")}
    result = []
    ids = set()
    for element in root.iter():
        xmi_id = element.get(XMI + "id")
        xml_type = element.get(XMI + "type", "").removeprefix("uml:")
        # Root UML package has a tag rather than xmi:type.
        if element.tag.endswith("}Package"):
            xml_type = "Package"
        if not xmi_id or xml_type not in KINDS:
            continue
        assert xmi_id not in ids
        ids.add(xmi_id)
        owner = parents.get(element)
        while owner is not None and not owner.get(XMI + "id"):
            owner = parents.get(owner)
        classifier = element
        while classifier is not None and classifier.get(XMI + "type") not in ("uml:Class", "uml:Enumeration"):
            classifier = parents.get(classifier)
        clause = names.get(classifier.get("name")) if classifier is not None else None
        section = clause["section"] if clause else "8.3"
        item = {"id": f"{standard.upper()}-XMI-{xmi_id}", "standard": standard,
                "kind": KINDS[xml_type], "facet": "abstract-syntax",
                "title": element.get("name") or xmi_id, "section": section,
                "source": standard + "-xmi", "fragment": xmi_id,
                "parent": clause_id(standard, section)}
        if owner is not None and owner.get(XMI + "id"):
            item["ownerFragment"] = owner.get(XMI + "id")
        if clause:
            item["pdfPage"] = clause["page"]
        if xml_type == "Constraint":
            name = element.get("name", "")
            item["ruleKind"] = next((p for p in ["validate", "derive", "check"] if name.startswith(p)), "operation-condition")
        # Retain structural facts and reference identities, never copied prose/OCL.
        facts = {k: v for k, v in element.attrib.items()
                 if k in ["isAbstract", "isDerived", "isReadOnly", "isOrdered", "isUnique", "aggregation", "direction"]}
        for tag in ["type", "lowerValue", "upperValue", "general", "subsettedProperty", "redefinedProperty", "memberEnd"]:
            values = []
            for child in element.findall(tag):
                value = child.get(XMI + "idref") or child.get("href") or child.get("value")
                if value is None and tag in ["lowerValue", "upperValue"]:
                    value = "0"  # UML integer literal default; explicit child exists.
                if value is not None:
                    values.append(value)
            if values:
                facts[tag] = values
        if facts:
            item["facts"] = facts
        result.append(item)
    counts = dict(sorted(Counter(x["kind"] for x in result).items()))
    assert counts["metaclass"] == (93 if standard == "sysml" else 82)
    assert all("pdfPage" in x for x in result if x["kind"] in ["metaclass", "enumeration"]), "Unmapped classifier"
    return result, counts


def companion_index(source_id, source, path):
    """Inventory every normative companion and its named definition/model files.

    An archive model file or schema definition is a review container, not an
    assertion that each declaration/property has already been decomposed.
    """
    standard = source["standard"]
    section = "9" if path.suffix == ".kpar" else ("10" if standard == "kerml" else "2")
    base = {"standard": standard, "source": source_id, "section": section,
            "parent": clause_id(standard, section),
            "facet": "libraries" if path.suffix == ".kpar" else "interchange"}
    result = [{**base, "id": source_id.upper(), "kind": "normative-artifact",
               "title": source["cacheFile"], "decomposition": "pending"}]
    if path.suffix == ".kpar":
        with ZipFile(path) as archive:
            files = sorted(i.filename for i in archive.infolist() if not i.is_dir())
            assert len(files) == len(set(files)), "Duplicate archive paths"
            for file in files:
                if not file.endswith((".sysml", ".kerml", ".json")):
                    # OS packaging metadata is inventoried in the artifact audit,
                    # but is not a modeling-language requirement.
                    continue
                data = archive.read(file)
                result.append({**base, "id": f"{source_id.upper()}-FILE-{file}",
                               "kind": "library-model" if file.endswith((".sysml", ".kerml")) else "archive-metadata",
                               "title": file, "archivePath": file,
                               "textSha256": sha(data), "decomposition": "pending"})
            audit = {"archiveFiles": len(files), "indexedFiles": len(result) - 1,
                     "packagingMetadataExcluded": [f for f in files if not f.endswith((".sysml", ".kerml", ".json"))]}
    else:
        schema = json.loads(path.read_text(encoding="utf8"))
        for name, definition in sorted(schema["$defs"].items()):
            pointer = "/$defs/" + name.replace("~", "~0").replace("/", "~1")
            result.append({**base, "id": f"{source_id.upper()}-DEF-{name}",
                           "kind": "schema-definition", "title": name,
                           "fragment": pointer, "decomposition": "pending",
                           "facts": {"properties": sorted(definition.get("properties", {})),
                                     "required": definition.get("required", []),
                                     "definitionSha256": sha(json.dumps(definition, sort_keys=True, separators=(",", ":")).encode())}})
        audit = {"schemaDefinitions": len(schema["$defs"])}
    return result, audit


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, default=ROOT / "target/standards")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    manifest = json.loads((ROOT / "standards/sources.json").read_text(encoding="utf8"))
    assert pypdf.__version__ == manifest["extraction"]["pypdf"], "Use pinned pypdf"
    version = subprocess.run(["pdftotext", "-v"], capture_output=True, text=True, check=True).stderr
    assert f"pdftotext version {manifest['extraction']['pdftotext']}" in version, "Use pinned Poppler"
    for source in manifest["sources"].values():
        data = (args.source_dir / source["cacheFile"]).read_bytes()
        assert len(data) == source["bytes"] and sha(data) == source["sha256"], f"Changed source: {source['cacheFile']}"
    items, audits = [], {}
    for standard in ["sysml", "kerml"]:
        pdf_items, audit = pdf_index(standard, args.source_dir / f"{standard}.pdf", manifest["sources"][standard + "-pdf"])
        xmi_items, xmi_counts = xmi_index(standard, args.source_dir / f"{standard}.xmi", pdf_items)
        items.extend(pdf_items + xmi_items)
        audits[standard] = {**audit, "xmi": xmi_counts}
    audits["companions"] = {}
    for source_id, source in manifest["sources"].items():
        path = args.source_dir / source["cacheFile"]
        if path.suffix in [".kpar", ".json"]:
            companion_items, audit = companion_index(source_id, source, path)
            items.extend(companion_items)
            audits["companions"][source_id] = audit
    assert len({x["id"] for x in items}) == len(items)
    catalogue = {"version": 1, "sourcesSha256": sha((ROOT / "standards/sources.json").read_bytes()),
                 "audit": audits, "items": items}
    output = json.dumps(catalogue, ensure_ascii=False, indent=2) + "\n"
    destination = ROOT / "standards/catalogue.json"
    if args.check:
        assert destination.read_text(encoding="utf8") == output, "Standards catalogue drift"
    else:
        destination.write_text(output, encoding="utf8", newline="\n")
    print(json.dumps(audits, indent=2))
    print(f"Indexed {len(items)} source units; prose decomposition remains pending.")


if __name__ == "__main__":
    main()
