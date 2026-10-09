#!/usr/bin/env python3
"""Generates `rule-inventory.md` from `specification/rules/*.md`.

Run from the repository root:

    python3 docs/design/evidence/make_rule_inventory.py

The classification is a *planning* artefact for G3/G4: for every rule file shipped in
`specification/rules/` it records whether this library intends to enforce the rule, and at
which layer. The classification is manual (see DEFAULTS / OVERRIDES below) but the inventory
itself is generated, so it can never drift out of sync with the rule files on disk.
"""

import os
import re
from collections import Counter

RULES_DIR = "specification/rules"
OUT = "docs/design/evidence/rule-inventory.md"

# Enforcement layers.
LOCAL = "A: local (type/construction)"
PARSER = "B: parser (well-formedness)"
VALID = "C: whole-document validation"
SER = "D: serializer"
OUT_OF_SCOPE = "X: out of scope"
N_A = "N/A: not applicable"

SCOPE_ORDER = [LOCAL, PARSER, VALID, SER, OUT_OF_SCOPE, N_A]

# Per-section default.
DEFAULTS = {
    "attributes": OUT_OF_SCOPE,
    "entities": OUT_OF_SCOPE,
    "document-structure": OUT_OF_SCOPE,
    "elements-and-tags": OUT_OF_SCOPE,
    "validity": OUT_OF_SCOPE,
    "well-formedness": LOCAL,
    "namespace-basics": LOCAL,
    "namespace-usage": LOCAL,
}

# Explicit per-rule classification: "<section>.<rule>" -> (layer, note)
OVERRIDES = {
    # --- well-formedness -------------------------------------------------------------
    "well-formedness.escape-ampersand-and-lt": (SER, "escaped on output; rejected on input"),
    "well-formedness.escape-gt-in-cdata-close": (LOCAL, "CData rejects `]]>`"),
    "well-formedness.char-ref-legal-char": (PARSER, "character references must denote legal chars"),
    "well-formedness.comment-no-double-hyphen": (LOCAL, "Comment"),
    "well-formedness.comment-no-triple-hyphen": (LOCAL, "Comment"),
    "well-formedness.elements-nest-properly": (PARSER, "start/end tag matching"),
    "well-formedness.single-root-element": (PARSER, "plus structural check in validation"),
    "well-formedness.document-production": (PARSER, "prolog/misc/root/epilog shape"),
    "well-formedness.no-peref-in-comments": (PARSER, "no entity expansion in comments"),
    "well-formedness.no-peref-in-pis": (PARSER, "no entity expansion in PIs"),
    "well-formedness.pi-pass-through": (PARSER, "PI content kept verbatim"),
    "well-formedness.entities-must-be-well-formed": (PARSER, "partially: predefined refs only"),
    "well-formedness.utf-8-utf-16-support": (PARSER, "UTF-8 only, by project scope"),
    "well-formedness.whitespace-definition": (PARSER, "S production for whitespace handling"),
    "well-formedness.xml-prefix-reserved": (LOCAL, "validate_namespace / PiTarget"),
    "well-formedness.legal-characters": (LOCAL, "Text / is_legal_char"),
    "well-formedness.name-starts-with-valid-char": (LOCAL, "NCName"),
    "well-formedness.name-valid-chars": (LOCAL, "NCName"),
    "well-formedness.pi-no-contains-close": (LOCAL, "PiData"),
    "well-formedness.pi-target-is-name": (LOCAL, "PiTarget"),
    "well-formedness.pi-target-not-xml": (LOCAL, "PiTarget"),
    # --- namespace-basics ------------------------------------------------------------
    "namespace-basics.empty-string-not-namespace-name": (LOCAL, "Namespace::new rejects empty URI"),
    "namespace-basics.uri-comparison-literal-case-sensitive": (LOCAL, "is_equal_ns compares bytes"),
    "namespace-basics.ns-decl-attribute-syntax": (PARSER, "xmlns / xmlns:prefix recognition"),
    "namespace-basics.ns-decl-value-uri-or-empty": (PARSER, "empty value only for default ns"),
    "namespace-basics.ncname-definition": (LOCAL, "NCName"),
    "namespace-basics.namespace-name-is-uri-reference": (N_A, "no IRI syntax check required by spec"),
    "namespace-basics.reserved-xml-prefixes": (LOCAL, "validate_namespace"),
    "namespace-basics.no-other-prefix-to-xml-namespace": (LOCAL, "validate_namespace"),
    "namespace-basics.no-other-prefix-to-xmlns-namespace": (LOCAL, "validate_namespace"),
    "namespace-basics.xml-namespace-not-default": (LOCAL, "validate_namespace"),
    "namespace-basics.xmlns-namespace-not-default": (LOCAL, "validate_namespace"),
    "namespace-basics.xml-prefix-fixed-binding": (LOCAL, "validate_xml_prefix_binding"),
    "namespace-basics.xmlns-prefix-not-declared": (LOCAL, "validate_namespace"),
    "namespace-basics.xmlns-not-element-prefix": (VALID, "prefix `xmlns` must not be used as an element prefix"),
    # --- namespace-usage -------------------------------------------------------------
    "namespace-usage.qname-format": (LOCAL, "split_qname"),
    "namespace-usage.zero-or-one-colon": (LOCAL, "split_qname"),
    "namespace-usage.prefix-and-localpart-ncname": (LOCAL, "split_qname"),
    "namespace-usage.must-be-well-formed": (LOCAL, "NCName / QName validation"),
    "namespace-usage.no-colon-typed-attributes": (PARSER, "attribute values of ID type - DTD-free subset"),
    "namespace-usage.other-tokens-ncname": (LOCAL, "NCName for other name-like tokens"),
    "namespace-usage.attributes-unique-expanded-name": (PARSER, "duplicate attribute detection"),
    "namespace-usage.prefix-declared": (PARSER, "undefined prefix rejected while parsing"),
    "namespace-usage.prefix-declaration-scope": (VALID, "declaration must be in scope for every use"),
    "namespace-usage.default-namespace-scope": (VALID, "default namespace scope is a document property"),
    "namespace-usage.default-namespace-unprefixed-elements": (PARSER, "element resolution"),
    "namespace-usage.default-namespace-not-attributes": (LOCAL, "resolve_attribute ignores default ns"),
    "namespace-usage.empty-default-namespace": (PARSER, "xmlns=\"\" removes the default namespace"),
    "namespace-usage.no-prefix-undeclaring": (PARSER, "xmlns:p=\"\" is rejected"),
    "namespace-usage.declarations-direct-or-internal-dtd": (PARSER, "direct declarations only - no DTD processing"),
    "namespace-usage.dtd-not-namespace-aware": (N_A, "no DTD processing"),
    "namespace-usage.processor-report-validity": (N_A, "validating processor - no DTD processing"),
    "namespace-usage.processor-report-wellformedness": (PARSER, "namespace well-formedness errors are reported"),
    "namespace-usage.uri-check-not-required": (N_A, "spec explicitly does not require URI checks"),
    # --- document-structure ----------------------------------------------------------
    "document-structure.cdata-section-must-start-with-cdstart": (PARSER, "CDStart recognition"),
    "document-structure.cdata-section-must-end-with-cdend": (PARSER, "CDEnd recognition"),
    "document-structure.cdata-section-must-not-contain-cdend": (LOCAL, "CData"),
    "document-structure.cdata-sections-must-not-nest": (PARSER, "CDATA has no markup inside"),
    "document-structure.processor-must-normalize-line-breaks": (PARSER, "line-end normalisation"),
    "document-structure.processor-must-pass-all-non-markup-characters": (PARSER, "text passthrough"),
    "document-structure.xml-lang-must-be-declared": (VALID, "xml:lang scope is document-wide"),
    "document-structure.xml-lang-must-inherit-to-descendants": (VALID, "inheritance is a tree property"),
    "document-structure.xml-lang-empty-must-override-ancestor": (VALID, "inheritance is a tree property"),
    "document-structure.xml-lang-must-be-bcp47-or-empty": (LOCAL, "value is locally checkable") ,
    "document-structure.xml-space-must-be-declared": (VALID, "xml:space scope is document-wide"),
    "document-structure.xml-space-must-be-enumerated-default-preserve": (LOCAL, "value is locally checkable"),
    "document-structure.doctype-must-precede-first-element": (OUT_OF_SCOPE, "DTD"),
    "document-structure.external-subset-must-match-extsubset": (OUT_OF_SCOPE, "DTD"),
    "document-structure.internal-subset-must-precede-external-subset": (OUT_OF_SCOPE, "DTD"),
    "document-structure.pe-references-must-not-within-markup-decls-internal-subset": (OUT_OF_SCOPE, "DTD"),
    "document-structure.pe-replacement-text-in-declsep-must-match-extsubsetdecl": (OUT_OF_SCOPE, "DTD"),
    "document-structure.validating-processor-must-identify-whitespace-in-element-content": (OUT_OF_SCOPE, "DTD element types"),
    # --- elements-and-tags -----------------------------------------------------------
    "elements-and-tags.start-tag-syntax": (PARSER, "attribute syntax, duplicate attribute"),
    "elements-and-tags.unique-attribute-specification": (PARSER, "no attribute with the same name"),
    "elements-and-tags.end-tag-must-match-start-tag": (PARSER, "tag matching"),
    "elements-and-tags.every-start-tag-must-have-end-tag": (PARSER, "tag matching"),
    "elements-and-tags.empty-element-representation": (SER, "<a/> vs <a></a> is a serialization choice"),
    "elements-and-tags.no-less-than-in-attribute-values": (PARSER, "attribute value must not contain `<`"),
    "elements-and-tags.no-external-entity-refs-in-attribute-values": (PARSER, "no entity references in attributes"),
    "elements-and-tags.interoperability-shoulds": (N_A, "SHOULD-level interoperability guidance"),
    # --- attributes ------------------------------------------------------------------
    "attributes.unique-name-in-tag": (PARSER, "duplicate attribute in one start tag"),
    "attributes.no-lt-in-values": (PARSER, "`<` in attribute value"),
    "attributes.no-external-entity-refs": (PARSER, "external entity in attribute value"),
    "attributes.no-undeclared-entity-refs": (PARSER, "undeclared entity reference"),
    "attributes.values-must-be-normalized": (PARSER, "attribute value normalisation"),
    "attributes.linebreaks-normalized-to-lf": (PARSER, "line-end normalisation in attribute values"),
    "attributes.whitespace-normalized-to-space": (PARSER, "whitespace normalisation in attribute values"),
    "attributes.char-ref-legal-char": (PARSER, "character reference must denote a legal char"),
    "attributes.char-refs-expanded": (PARSER, "character references are expanded"),
    "attributes.entity-refs-expanded": (PARSER, "predefined entity references are expanded"),
    "attributes.only-in-start-tags": (PARSER, "attributes only legal in start tags"),
    "attributes.must-be-declared": (OUT_OF_SCOPE, "requires an attribute-list declaration (DTD)"),
    "attributes.id-must-be-unique": (VALID, "unique `xml:id` values across the document"),
    "attributes.id-must-be-name": (VALID, "`xml:id` must be an NCName"),
    "attributes.one-id-per-element-type": (OUT_OF_SCOPE, "DTD"),
    # --- entities --------------------------------------------------------------------
    "entities.predefined-entities-recognized": (PARSER, "amp/lt/gt/apos/quot expanded"),
    "entities.default-encoding-utf8": (PARSER, "UTF-8 is the assumed encoding"),
    "entities.encoding-must-match-declaration": (PARSER, "declared encoding must be UTF-8"),
    "entities.encoding-name-case-insensitive": (PARSER, "encoding name comparison"),
    "entities.encoding-decl-required-non-utf": (OUT_OF_SCOPE, "non-UTF-8 encodings"),
    "entities.bom-encoding-detection": (PARSER, "UTF-8 BOM handling"),
    "entities.utf8-bom-optional": (PARSER, "UTF-8 BOM handling"),
    "entities.utf-utf16-support": (PARSER, "UTF-8 half only, by project scope"),
    "entities.utf16-bom-required": (OUT_OF_SCOPE, "UTF-16 not supported"),
    "entities.utf-excludes-related": (OUT_OF_SCOPE, "non-UTF-8 encodings"),
    "entities.illegal-byte-sequence-fatal": (PARSER, "invalid UTF-8 is a fatal error"),
    "entities.unsupported-encoding-fatal": (PARSER, "unsupported encoding is a fatal error"),
    "entities.no-encoding-legal-utf-required": (PARSER, "no encoding declaration, valid UTF-8"),
    "entities.feff-requires-bom": (OUT_OF_SCOPE, "non-UTF-8 encodings"),
    "entities.only-referenced-entities-well-formed": (PARSER, "partially: external declaration subset is ignored"),
    "entities.document-entity-well-formed": (PARSER, "partially: single well-formed root"),
    "entities.charref-legal-character": (PARSER, "character reference must denote a legal char"),
    "entities.nonvalidator-notify-skip": (OUT_OF_SCOPE, "DTD"),
}


def main() -> None:
    files = sorted(os.listdir(RULES_DIR))
    rows = []
    for fname in files:
        m = re.match(r"rule\.(.*)\.md$", fname)
        if not m:
            continue
        key = m.group(1)
        section = key.split(".")[0]
        rule = key.split(".", 1)[1]
        layer, note = OVERRIDES.get(key, (DEFAULTS[section], "default for this specification section"))
        rows.append((section, rule, layer, note, fname))

    counts = Counter(r[2] for r in rows)
    lines = []
    lines.append("# Rule inventory — `specification/rules/` vs. enforcement layer")
    lines.append("")
    lines.append(f"Generated by `python3 {os.path.basename(__file__)}` from {len(rows)} rule files.")
    lines.append("Regenerate after adding or removing a rule file; the table is never edited by hand.")
    lines.append("")
    lines.append("Layers:")
    lines.append("")
    for scope in SCOPE_ORDER:
        lines.append(f"- **{scope}** — {counts.get(scope, 0)} rule files")
    lines.append("")
    lines.append(
        "`A` = enforced at construction time by the validated `xml_spec` types; "
        "`B` = enforced while parsing (document-level well-formedness that is locally decidable); "
        "`C` = only detectable with a whole-document view, therefore implemented by "
        "`Document::validate`; `D` = a serialization-side concern; "
        "`X` = deliberately out of scope (DTD / validity / non-UTF-8 encodings); "
        "`N/A` = the rule does not oblige this library."
    )
    lines.append("")

    for section in sorted({r[0] for r in rows}):
        section_rows = [r for r in rows if r[0] == section]
        lines.append(f"## `{section}` ({len(section_rows)} rules)")
        lines.append("")
        lines.append("| rule | layer | note |")
        lines.append("| --- | --- | --- |")
        for _, rule, layer, note, _ in section_rows:
            lines.append(f"| `{rule}` | {layer} | {note} |")
        lines.append("")

    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"wrote {OUT}: {len(rows)} rules, {dict(counts)}")


if __name__ == "__main__":
    main()
