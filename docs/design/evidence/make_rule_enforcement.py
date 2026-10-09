#!/usr/bin/env python3
"""Generates `rule-enforcement.md`: what G3/G4 did with each layer-B, layer-C and layer-D rule.

Run from the repository root:

    python3 docs/design/evidence/make_rule_enforcement.py

The G1 review produced `rule-inventory.md`, which assigns every rule file to an enforcement layer
but does not say whether the layer has actually been *implemented*. This script closes that gap for
the implemented layers: the 55 rules the parser must enforce (layer B), the 9 that need a
whole-document view (layer C) and the 2 that are the serializer's concern (layer D).

`ENFORCEMENT` is written by hand (it is a claim about the code), but the *set of rules* is read from
`rule-inventory.md`, so a rule can never be silently dropped from the table: the script fails if a
layer-B/D rule has no verdict.
"""

import re
import sys

INVENTORY = "docs/design/evidence/rule-inventory.md"
OUT = "docs/design/evidence/rule-enforcement.md"

# rule id -> (verdict, where / why)
#   "enforced"  - the rule is implemented; `where` names the code and the test
#   "partial"   - implemented for the part that is expressible without DTD processing
#   "deferred"  - deliberately not implemented; `where` gives the one-line reason
#   "n/a"       - the rule does not oblige this library (it prescribes processor behaviour that no
#                 API of this crate exposes)
ENFORCEMENT = {
    # --- attributes --------------------------------------------------------------------
    "attributes.char-ref-legal-char": ("enforced", "`parse_value` + `Text` validation; an illegal expansion is `InvalidText`"),
    "attributes.char-refs-expanded": ("enforced", "`Attributes::normalized_value_with`; `attribute_value_normalisation_in_both_directions`"),
    "attributes.entity-refs-expanded": ("enforced", "same; the five predefined entities are the only resolver"),
    "attributes.linebreaks-normalized-to-lf": ("enforced", "same; `\\r\\n`/`\\r` become a space like any other literal whitespace"),
    "attributes.no-external-entity-refs": ("enforced", "every non-predefined reference is `UndeclaredEntityReference`"),
    "attributes.no-lt-in-values": ("enforced", "`parse` rejects `<` in the raw attribute value; `less_than_is_rejected_in_attribute_values`"),
    "attributes.no-undeclared-entity-refs": ("enforced", "`undeclared_entity_references_are_typed_errors_not_panics`"),
    "attributes.only-in-start-tags": ("enforced", "the tokenizer only produces attributes for start/empty-element tags"),
    "attributes.unique-name-in-tag": ("enforced", "`attribute_error` maps `AttrError::Duplicated` to `DuplicateAttribute`"),
    "attributes.values-must-be-normalized": ("enforced", "`attribute_value_normalisation_in_both_directions`"),
    "attributes.whitespace-normalized-to-space": ("enforced", "same"),
    # --- document structure ------------------------------------------------------------
    "document-structure.cdata-section-must-end-with-cdend": ("enforced", "the tokenizer only emits a CDATA event for a closed section"),
    "document-structure.cdata-section-must-start-with-cdstart": ("enforced", "same"),
    "document-structure.cdata-sections-must-not-nest": ("enforced", "the tokenizer treats CDATA content as opaque"),
    "document-structure.processor-must-normalize-line-breaks": ("enforced", "`BytesText::xml10_content` + `xml_spec::normalize_line_ends`; `line_ends_are_normalised_in_text`"),
    "document-structure.processor-must-pass-all-non-markup-characters": ("enforced", "text is stored verbatim; `text_is_escaped_so_that_it_round_trips`"),
    # --- elements and tags -------------------------------------------------------------
    "elements-and-tags.end-tag-must-match-start-tag": ("enforced", "tokenizer end-name check; `tags_must_nest_and_close`"),
    "elements-and-tags.every-start-tag-must-have-end-tag": ("enforced", "the parser keeps its own element stack; `tags_must_nest_and_close`"),
    "elements-and-tags.no-external-entity-refs-in-attribute-values": ("enforced", "as `attributes.no-external-entity-refs`"),
    "elements-and-tags.no-less-than-in-attribute-values": ("enforced", "as `attributes.no-lt-in-values`"),
    "elements-and-tags.start-tag-syntax": ("enforced", "attribute syntax errors come from the tokenizer, duplicates from this crate"),
    "elements-and-tags.unique-attribute-specification": ("enforced", "raw duplicates and duplicates by expanded name are both rejected"),
    # --- entities ----------------------------------------------------------------------
    "entities.bom-encoding-detection": ("enforced", "`parse_bytes` strips a leading UTF-8 BOM; `an_optional_utf8_byte_order_mark_is_accepted`"),
    "entities.charref-legal-character": ("enforced", "`resolve_reference` checks the `Char` production; `illegal_character_references_are_rejected`"),
    "entities.default-encoding-utf8": ("enforced", "UTF-8 is the only supported encoding"),
    "entities.document-entity-well-formed": ("partial", "the document *is* checked to be well-formed (tags, single root); the DTD parts of the rule are out of scope"),
    "entities.encoding-must-match-declaration": ("enforced", "`unsupported_declarations_are_rejected`"),
    "entities.encoding-name-case-insensitive": ("enforced", "`xml_spec::declaration::encoding_is_utf8`; `the_declaration_is_interpreted_and_preserved`"),
    "entities.illegal-byte-sequence-fatal": ("enforced", "`parse_bytes` validates UTF-8 first; `invalid_utf8_is_a_typed_error`"),
    "entities.no-encoding-legal-utf-required": ("enforced", "a declaration without `encoding` is accepted and validated as UTF-8"),
    "entities.only-referenced-entities-well-formed": ("partial", "references are resolved (or rejected) correctly; entity *declarations* need DTD processing, which is out of scope"),
    "entities.predefined-entities-recognized": ("enforced", "`predefined_entities_are_expanded`"),
    "entities.unsupported-encoding-fatal": ("enforced", "`UnsupportedEncoding`; `unsupported_declarations_are_rejected`"),
    "entities.utf-utf16-support": ("deferred", "UTF-16 is out of the project's scope (AGENTS.md: XML 1.0 + UTF-8); a UTF-16 input is rejected as invalid UTF-8"),
    "entities.utf8-bom-optional": ("enforced", "as `entities.bom-encoding-detection`"),
    # --- namespace basics / usage ------------------------------------------------------
    "namespace-basics.ns-decl-attribute-syntax": ("enforced", "`Parser::namespace_declarations`"),
    "namespace-basics.ns-decl-value-uri-or-empty": ("enforced", "an empty value is only accepted for the default namespace; `namespaces_are_resolved_and_reported`"),
    "namespace-usage.attributes-unique-expanded-name": ("enforced", "duplicates by expanded name are rejected"),
    "namespace-usage.declarations-direct-or-internal-dtd": ("partial", "only direct declarations exist here; declarations in an internal DTD subset are out of scope"),
    "namespace-usage.default-namespace-unprefixed-elements": ("enforced", "unprefixed element names take the default namespace; `namespaces_are_resolved_and_reported`"),
    "namespace-usage.empty-default-namespace": ("enforced", "`an_empty_default_declaration_removes_the_default_namespace`"),
    "namespace-usage.no-colon-typed-attributes": ("enforced", "attribute names are validated as `NCName` parts of a `QName`"),
    "namespace-usage.no-prefix-undeclaring": ("enforced", "`xmlns:p=\"\"` is `InvalidNamespace`; `namespaces_are_resolved_and_reported`"),
    "namespace-usage.prefix-declared": ("enforced", "`undeclared namespace prefix`; `namespaces_are_resolved_and_reported`"),
    "namespace-usage.processor-report-wellformedness": ("enforced", "namespace well-formedness failures are reported as typed errors, never silently ignored"),
    # --- well-formedness ---------------------------------------------------------------
    "well-formedness.char-ref-legal-char": ("enforced", "as `entities.charref-legal-character`"),
    "well-formedness.document-production": ("enforced", "single root plus the documented top-level policy; `there_must_be_exactly_one_root_element`, `top_level_content_policy`"),
    "well-formedness.elements-nest-properly": ("enforced", "`tags_must_nest_and_close`"),
    "well-formedness.entities-must-be-well-formed": ("partial", "references must be well formed and predefined; entity declarations are out of scope"),
    "well-formedness.no-peref-in-comments": ("partial", "references inside comments are never expanded (this crate stores comment text verbatim); parameter entities do not exist without DTD processing"),
    "well-formedness.no-peref-in-pis": ("partial", "same, for processing-instruction content"),
    "well-formedness.pi-pass-through": ("enforced", "`processing_instruction_content_is_kept_verbatim`"),
    "well-formedness.single-root-element": ("enforced", "`MultipleRootElements`; `there_must_be_exactly_one_root_element`"),
    "well-formedness.utf-8-utf-16-support": ("deferred", "UTF-16 is out of scope; UTF-8 is fully supported (see `entities.utf-utf16-support`)"),
    "well-formedness.whitespace-definition": ("enforced", "the `S` production is used for the declaration separator and for recognising ignorable top-level whitespace"),
    # --- validation layer (goal G4) -----------------------------------------------------
    "attributes.id-must-be-name": ("enforced", "`check_xml_attribute_values` rejects an `xml:id` that is not an `NCName`; `xml_id_values_must_be_names_and_unique`"),
    "attributes.id-must-be-unique": ("enforced", "`check_xml_id_uniqueness` walks the attached tree; `xml_id_values_must_be_names_and_unique`"),
    "document-structure.xml-lang-empty-must-override-ancestor": ("n/a", "prescribes how a processor resolves the language of an element; this crate exposes no API that consumes `xml:lang`, so there is nothing to check"),
    "document-structure.xml-lang-must-be-declared": ("deferred", "\"MUST be declared\" is an attribute-list (DTD) constraint; without DTD processing there is no declaration to check. The checkable half, the value shape, is enforced via `rule.document-structure.xml-lang-must-be-bcp47-or-empty`"),
    "document-structure.xml-lang-must-inherit-to-descendants": ("n/a", "as `xml-lang-empty-must-override-ancestor`: processor behaviour for a feature this crate does not model"),
    "document-structure.xml-space-must-be-declared": ("deferred", "as `xml-lang-must-be-declared`; the value shape is enforced via `rule.document-structure.xml-space-must-be-enumerated-default-preserve`"),
    "namespace-basics.xmlns-not-element-prefix": ("enforced", "`Namespace::prefixed`/`Namespace::new` refuse the reserved prefix at construction (layer A); `ScopeViolation::ReservedPrefix` re-checks it during validation, though no public constructor can reach that state"),
    "namespace-usage.default-namespace-scope": ("enforced", "`check_element_name`/`check_attribute_name`; `default_namespace_scope_must_match_the_name`, `attributes_are_not_affected_by_the_default_namespace`"),
    "namespace-usage.prefix-declaration-scope": ("enforced", "`NamespaceScope::from_declarations` plus `check_element_name`; `a_prefix_bound_to_a_different_uri_is_reported`, `editing_is_silent_and_validation_is_what_reports`"),
    # --- serializer layer (goal G3) -----------------------------------------------------
    "elements-and-tags.empty-element-representation": ("enforced", "`EmptyElementStyle`; `empty_elements_and_write_options`"),
    "well-formedness.escape-ampersand-and-lt": ("enforced", "`escape_text`/`escape_attribute`; `text_is_escaped_so_that_it_round_trips`"),
}


def main() -> None:
    inventory = open(INVENTORY, encoding="utf-8").read()
    # The inventory groups rows under `## `section`` headings, so the full rule id is
    # `section.rule` - the same spelling the rule file uses.
    owned = []
    section = None
    for line in inventory.splitlines():
        heading = re.match(r"^## `([a-z-]+)`", line)
        if heading:
            section = heading.group(1)
            continue
        row = re.match(r"^\| `([^`]+)` \| (B|C|D): [^|]*\|", line)
        if row and section:
            owned.append((f"{section}.{row.group(1)}", row.group(2)))
    if not owned:
        sys.exit(f"no layer-B/D rules found in {INVENTORY}")

    missing = sorted({rule for rule, _ in owned} - set(ENFORCEMENT))
    if missing:
        sys.exit("no verdict for: " + ", ".join(missing))
    extra = sorted(set(ENFORCEMENT) - {rule for rule, _ in owned})
    if extra:
        sys.exit("verdict for a rule that is not layer B/D: " + ", ".join(extra))

    counts = {"enforced": 0, "partial": 0, "deferred": 0, "n/a": 0}
    lines = [
        "# Rule enforcement — layer B (parser), layer C (validation) and layer D (serializer)",
        "",
        f"Generated by `python3 {__file__.split('/')[-1]}` from `{INVENTORY}` plus an explicit",
        "per-rule verdict. The *set* of rules is read from the inventory, so a rule cannot be",
        "dropped from this table silently: the script fails if a layer-B/D rule has no verdict.",
        "",
        "Verdicts:",
        "",
        "- **enforced** — implemented; the note names the code path and the test that pins it.",
        "- **partial** — the expressible part is implemented; the rest needs DTD processing, which",
        "  AGENTS.md puts out of scope (no `DOCTYPE` validation).",
        "- **deferred** — deliberately not implemented; the note gives the reason.",
        "- **n/a** — the rule does not oblige this library (it prescribes processor behaviour that no",
        "  API of this crate exposes).",
        "",
        "| rule | layer | verdict | code path / reason |",
        "| --- | --- | --- | --- |",
    ]
    for rule, layer in owned:
        verdict, note = ENFORCEMENT[rule]
        counts[verdict] += 1
        lines.append(f"| `rule.{rule}.md` | {layer} | {verdict} | {note} |")

    lines += [
        "",
        "## Summary",
        "",
        f"- {counts['enforced']} enforced",
        f"- {counts['partial']} partial (the remainder needs DTD processing)",
        f"- {counts['deferred']} deferred (UTF-16 or DTD validity, both out of the project's scope)",
        f"- {counts['n/a']} not applicable",
        f"- {len(owned)} total layer-B/C/D rules",
        "",
        "Everything not listed here belongs to layer A (validated types, in `xml_spec`), layer C",
        "(whole-document validation) or layer X (out of scope); see `rule-inventory.md`.",
        "",
    ]
    with open(OUT, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
    print(f"wrote {OUT}: {len(owned)} rules, {counts}")


if __name__ == "__main__":
    main()
