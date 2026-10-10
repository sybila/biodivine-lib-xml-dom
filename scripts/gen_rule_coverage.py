#!/usr/bin/env python3
"""Generates `docs/design/rule-coverage.md` from the rule summaries and the classification below.

Run from the repository root:

    python3 scripts/gen_rule_coverage.py            # write docs/design/rule-coverage.md
    python3 scripts/gen_rule_coverage.py --check    # fail if the file is out of date

The rule set is read from `specification/rules/` (and cross-checked against the module list of
`src/xml_spec/rule_set.rs`), so a rule summary that is added, renamed or deleted cannot go
unnoticed. The *classification* — which layer of the library is responsible for a rule, and whether
that responsibility is actually met — is written by hand, because it is a claim about the code and
its tests rather than something derivable from the specification text.

The script fails, rather than silently producing a shorter table, when

* a rule summary has neither an explicit entry in `LAYERS` nor a section default in
  `DEFAULT_LAYERS`;
* a rule whose layer is B (parser), C (whole-document validation) or D (serializer) has no verdict in
  `VERDICTS` (those are the layers this library is expected to implement);
* `VERDICTS` contains an entry for a rule that no layer assigns to B, C or D;
* the rule set does not match the summary files on disk.

Layers: `A` = enforced at construction time by the validated `xml_spec` types; `B` = enforced while
parsing (well-formedness that is decidable locally); `C` = only detectable with a whole-document
view, therefore implemented by `Document::validate`; `D` = a serialization-side concern; `X` =
deliberately out of scope (DTD processing, validity checking, non-UTF-8 encodings); `N/A` = the rule
does not oblige this library (it prescribes processor behaviour that no API here exposes).
"""

import argparse
import difflib
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
RULES = ROOT / "specification" / "rules"
RULE_SET = ROOT / "src" / "xml_spec" / "rule_set.rs"
TARGET = ROOT / "docs" / "design" / "rule-coverage.md"

#: Status shown for the verdicts of layer-B/C/D rules.
STATUS = {
    "enforced": "enforced",
    "partial": "partial (the rest needs DTD processing)",
    "deferred": "deferred",
    "n/a": "n/a (the rule does not oblige this library)",
}

#: Status shown for the layers that do not carry a per-rule verdict.
LAYER_STATUS = {
    "A": "enforced by construction",
    "X": "out of scope",
    "N/A": "not applicable",
}

ORDER = [
    "A: local (type/construction)",
    "B: parser (well-formedness)",
    "C: whole-document validation",
    "D: serializer",
    "X: out of scope",
    "N/A: not applicable",
]

# Layer classification per specification section (rules without an explicit entry).
DEFAULT_LAYERS = {
    'attributes': 'X: out of scope',
    'entities': 'X: out of scope',
    'document-structure': 'X: out of scope',
    'elements-and-tags': 'X: out of scope',
    'validity': 'X: out of scope',
    'well-formedness': 'A: local (type/construction)',
    'namespace-basics': 'A: local (type/construction)',
    'namespace-usage': 'A: local (type/construction)',
}

# Explicit per-rule layer classification: rule id -> (layer, note).
LAYERS = {
    'well-formedness.escape-ampersand-and-lt': ('D: serializer', 'escaped on output; rejected on input'),
    'well-formedness.escape-gt-in-cdata-close': ('A: local (type/construction)', 'CData rejects `]]>`'),
    'well-formedness.char-ref-legal-char': ('B: parser (well-formedness)', 'character references must denote legal chars'),
    'well-formedness.comment-no-double-hyphen': ('A: local (type/construction)', 'Comment'),
    'well-formedness.comment-no-triple-hyphen': ('A: local (type/construction)', 'Comment'),
    'well-formedness.elements-nest-properly': ('B: parser (well-formedness)', 'start/end tag matching'),
    'well-formedness.single-root-element': ('B: parser (well-formedness)', 'plus structural check in validation'),
    'well-formedness.document-production': ('B: parser (well-formedness)', 'prolog/misc/root/epilog shape'),
    'well-formedness.no-peref-in-comments': ('B: parser (well-formedness)', 'no entity expansion in comments'),
    'well-formedness.no-peref-in-pis': ('B: parser (well-formedness)', 'no entity expansion in PIs'),
    'well-formedness.pi-pass-through': ('B: parser (well-formedness)', 'PI content kept verbatim'),
    'well-formedness.entities-must-be-well-formed': ('B: parser (well-formedness)', 'partially: predefined refs only'),
    'well-formedness.utf-8-utf-16-support': ('B: parser (well-formedness)', 'UTF-8 only, by project scope'),
    'well-formedness.whitespace-definition': ('B: parser (well-formedness)', 'S production for whitespace handling'),
    'well-formedness.xml-prefix-reserved': ('A: local (type/construction)', 'validate_namespace / PiTarget'),
    'well-formedness.legal-characters': ('A: local (type/construction)', 'Text / is_legal_char'),
    'well-formedness.name-starts-with-valid-char': ('A: local (type/construction)', 'NCName'),
    'well-formedness.name-valid-chars': ('A: local (type/construction)', 'NCName'),
    'well-formedness.pi-no-contains-close': ('A: local (type/construction)', 'PiData'),
    'well-formedness.pi-target-is-name': ('A: local (type/construction)', 'PiTarget'),
    'well-formedness.pi-target-not-xml': ('A: local (type/construction)', 'PiTarget'),
    'namespace-basics.empty-string-not-namespace-name': ('A: local (type/construction)', 'Namespace::new rejects empty URI'),
    'namespace-basics.uri-comparison-literal-case-sensitive': ('A: local (type/construction)', 'is_equal_ns compares bytes'),
    'namespace-basics.ns-decl-attribute-syntax': ('B: parser (well-formedness)', 'xmlns / xmlns:prefix recognition'),
    'namespace-basics.ns-decl-value-uri-or-empty': ('B: parser (well-formedness)', 'empty value only for default ns'),
    'namespace-basics.ncname-definition': ('A: local (type/construction)', 'NCName'),
    'namespace-basics.namespace-name-is-uri-reference': ('N/A: not applicable', 'no IRI syntax check required by spec'),
    'namespace-basics.reserved-xml-prefixes': ('A: local (type/construction)', 'validate_namespace'),
    'namespace-basics.no-other-prefix-to-xml-namespace': ('A: local (type/construction)', 'validate_namespace'),
    'namespace-basics.no-other-prefix-to-xmlns-namespace': ('A: local (type/construction)', 'validate_namespace'),
    'namespace-basics.xml-namespace-not-default': ('A: local (type/construction)', 'validate_namespace'),
    'namespace-basics.xmlns-namespace-not-default': ('A: local (type/construction)', 'validate_namespace'),
    'namespace-basics.xml-prefix-fixed-binding': ('A: local (type/construction)', 'validate_xml_prefix_binding'),
    'namespace-basics.xmlns-prefix-not-declared': ('A: local (type/construction)', 'validate_namespace'),
    'namespace-basics.xmlns-not-element-prefix': ('C: whole-document validation', 'prefix `xmlns` must not be used as an element prefix'),
    'namespace-usage.qname-format': ('A: local (type/construction)', 'split_qname'),
    'namespace-usage.zero-or-one-colon': ('A: local (type/construction)', 'split_qname'),
    'namespace-usage.prefix-and-localpart-ncname': ('A: local (type/construction)', 'split_qname'),
    'namespace-usage.must-be-well-formed': ('A: local (type/construction)', 'NCName / QName validation'),
    'namespace-usage.no-colon-typed-attributes': ('B: parser (well-formedness)', 'attribute values of ID type - DTD-free subset'),
    'namespace-usage.other-tokens-ncname': ('A: local (type/construction)', 'NCName for other name-like tokens'),
    'namespace-usage.attributes-unique-expanded-name': ('B: parser (well-formedness)', 'duplicate attribute detection'),
    'namespace-usage.prefix-declared': ('B: parser (well-formedness)', 'undefined prefix rejected while parsing'),
    'namespace-usage.prefix-declaration-scope': ('C: whole-document validation', 'declaration must be in scope for every use'),
    'namespace-usage.default-namespace-scope': ('C: whole-document validation', 'default namespace scope is a document property'),
    'namespace-usage.default-namespace-unprefixed-elements': ('B: parser (well-formedness)', 'element resolution'),
    'namespace-usage.default-namespace-not-attributes': ('A: local (type/construction)', 'resolve_attribute ignores default ns'),
    'namespace-usage.empty-default-namespace': ('B: parser (well-formedness)', 'xmlns="" removes the default namespace'),
    'namespace-usage.no-prefix-undeclaring': ('B: parser (well-formedness)', 'xmlns:p="" is rejected'),
    'namespace-usage.declarations-direct-or-internal-dtd': ('B: parser (well-formedness)', 'direct declarations only - no DTD processing'),
    'namespace-usage.dtd-not-namespace-aware': ('N/A: not applicable', 'no DTD processing'),
    'namespace-usage.processor-report-validity': ('N/A: not applicable', 'validating processor - no DTD processing'),
    'namespace-usage.processor-report-wellformedness': ('B: parser (well-formedness)', 'namespace well-formedness errors are reported'),
    'namespace-usage.uri-check-not-required': ('N/A: not applicable', 'spec explicitly does not require URI checks'),
    'document-structure.cdata-section-must-start-with-cdstart': ('B: parser (well-formedness)', 'CDStart recognition'),
    'document-structure.cdata-section-must-end-with-cdend': ('B: parser (well-formedness)', 'CDEnd recognition'),
    'document-structure.cdata-section-must-not-contain-cdend': ('A: local (type/construction)', 'CData'),
    'document-structure.cdata-sections-must-not-nest': ('B: parser (well-formedness)', 'CDATA has no markup inside'),
    'document-structure.processor-must-normalize-line-breaks': ('B: parser (well-formedness)', 'line-end normalisation'),
    'document-structure.processor-must-pass-all-non-markup-characters': ('B: parser (well-formedness)', 'text passthrough'),
    'document-structure.xml-lang-must-be-declared': ('C: whole-document validation', 'xml:lang scope is document-wide'),
    'document-structure.xml-lang-must-inherit-to-descendants': ('C: whole-document validation', 'inheritance is a tree property'),
    'document-structure.xml-lang-empty-must-override-ancestor': ('C: whole-document validation', 'inheritance is a tree property'),
    'document-structure.xml-lang-must-be-bcp47-or-empty': ('A: local (type/construction)', 'value is locally checkable'),
    'document-structure.xml-space-must-be-declared': ('C: whole-document validation', 'xml:space scope is document-wide'),
    'document-structure.xml-space-must-be-enumerated-default-preserve': ('A: local (type/construction)', 'value is locally checkable'),
    'document-structure.doctype-must-precede-first-element': ('X: out of scope', 'DTD'),
    'document-structure.external-subset-must-match-extsubset': ('X: out of scope', 'DTD'),
    'document-structure.internal-subset-must-precede-external-subset': ('X: out of scope', 'DTD'),
    'document-structure.pe-references-must-not-within-markup-decls-internal-subset': ('X: out of scope', 'DTD'),
    'document-structure.pe-replacement-text-in-declsep-must-match-extsubsetdecl': ('X: out of scope', 'DTD'),
    'document-structure.validating-processor-must-identify-whitespace-in-element-content': ('X: out of scope', 'DTD element types'),
    'elements-and-tags.start-tag-syntax': ('B: parser (well-formedness)', 'attribute syntax, duplicate attribute'),
    'elements-and-tags.unique-attribute-specification': ('B: parser (well-formedness)', 'no attribute with the same name'),
    'elements-and-tags.end-tag-must-match-start-tag': ('B: parser (well-formedness)', 'tag matching'),
    'elements-and-tags.every-start-tag-must-have-end-tag': ('B: parser (well-formedness)', 'tag matching'),
    'elements-and-tags.empty-element-representation': ('D: serializer', 'whose empty-element form (`<a/>` or `<a></a>`) to write is a serialization choice'),
    'elements-and-tags.no-less-than-in-attribute-values': ('B: parser (well-formedness)', 'attribute value must not contain `<`'),
    'elements-and-tags.no-external-entity-refs-in-attribute-values': ('B: parser (well-formedness)', 'no entity references in attributes'),
    'elements-and-tags.interoperability-shoulds': ('N/A: not applicable', 'SHOULD-level interoperability guidance'),
    'attributes.unique-name-in-tag': ('B: parser (well-formedness)', 'duplicate attribute in one start tag'),
    'attributes.no-lt-in-values': ('B: parser (well-formedness)', '`<` in attribute value'),
    'attributes.no-external-entity-refs': ('B: parser (well-formedness)', 'external entity in attribute value'),
    'attributes.no-undeclared-entity-refs': ('B: parser (well-formedness)', 'undeclared entity reference'),
    'attributes.values-must-be-normalized': ('B: parser (well-formedness)', 'attribute value normalisation'),
    'attributes.linebreaks-normalized-to-lf': ('B: parser (well-formedness)', 'line-end normalisation in attribute values'),
    'attributes.whitespace-normalized-to-space': ('B: parser (well-formedness)', 'whitespace normalisation in attribute values'),
    'attributes.char-ref-legal-char': ('B: parser (well-formedness)', 'character reference must denote a legal char'),
    'attributes.char-refs-expanded': ('B: parser (well-formedness)', 'character references are expanded'),
    'attributes.entity-refs-expanded': ('B: parser (well-formedness)', 'predefined entity references are expanded'),
    'attributes.only-in-start-tags': ('B: parser (well-formedness)', 'attributes only legal in start tags'),
    'attributes.must-be-declared': ('X: out of scope', 'requires an attribute-list declaration (DTD)'),
    'attributes.id-must-be-unique': ('C: whole-document validation', 'unique `xml:id` values across the document'),
    'attributes.id-must-be-name': ('C: whole-document validation', '`xml:id` must be an NCName'),
    'attributes.one-id-per-element-type': ('X: out of scope', 'DTD'),
    'entities.predefined-entities-recognized': ('B: parser (well-formedness)', 'amp/lt/gt/apos/quot expanded'),
    'entities.default-encoding-utf8': ('B: parser (well-formedness)', 'UTF-8 is the assumed encoding'),
    'entities.encoding-must-match-declaration': ('B: parser (well-formedness)', 'declared encoding must be UTF-8'),
    'entities.encoding-name-case-insensitive': ('B: parser (well-formedness)', 'encoding name comparison'),
    'entities.encoding-decl-required-non-utf': ('X: out of scope', 'non-UTF-8 encodings'),
    'entities.bom-encoding-detection': ('B: parser (well-formedness)', 'UTF-8 BOM handling'),
    'entities.utf8-bom-optional': ('B: parser (well-formedness)', 'UTF-8 BOM handling'),
    'entities.utf-utf16-support': ('B: parser (well-formedness)', 'UTF-8 half only, by project scope'),
    'entities.utf16-bom-required': ('X: out of scope', 'UTF-16 not supported'),
    'entities.utf-excludes-related': ('X: out of scope', 'non-UTF-8 encodings'),
    'entities.illegal-byte-sequence-fatal': ('B: parser (well-formedness)', 'invalid UTF-8 is a fatal error'),
    'entities.unsupported-encoding-fatal': ('B: parser (well-formedness)', 'unsupported encoding is a fatal error'),
    'entities.no-encoding-legal-utf-required': ('B: parser (well-formedness)', 'no encoding declaration, valid UTF-8'),
    'entities.feff-requires-bom': ('X: out of scope', 'non-UTF-8 encodings'),
    'entities.only-referenced-entities-well-formed': ('B: parser (well-formedness)', 'partially: external declaration subset is ignored'),
    'entities.document-entity-well-formed': ('B: parser (well-formedness)', 'partially: single well-formed root'),
    'entities.charref-legal-character': ('B: parser (well-formedness)', 'character reference must denote a legal char'),
    'entities.nonvalidator-notify-skip': ('X: out of scope', 'DTD'),
}

# Verdicts for the rules the library is expected to act on: rule id -> (verdict, evidence).
VERDICTS = {
    'attributes.char-ref-legal-char': ('enforced', '`parse_value` + `Text` validation; an illegal expansion is `InvalidText`'),
    'attributes.char-refs-expanded': ('enforced', '`Attributes::normalized_value_with`; `attribute_value_normalisation_in_both_directions`'),
    'attributes.entity-refs-expanded': ('enforced', 'same; the five predefined entities are the only resolver'),
    'attributes.linebreaks-normalized-to-lf': ('enforced', 'same; `\\r\\n`/`\\r` become a space like any other literal whitespace'),
    'attributes.no-external-entity-refs': ('enforced', 'every non-predefined reference is `UndeclaredEntityReference`'),
    'attributes.no-lt-in-values': ('enforced', '`parse` rejects `<` in the raw attribute value; `less_than_is_rejected_in_attribute_values`'),
    'attributes.no-undeclared-entity-refs': ('enforced', '`undeclared_entity_references_are_typed_errors_not_panics`'),
    'attributes.only-in-start-tags': ('enforced', 'the tokenizer only produces attributes for start/empty-element tags'),
    'attributes.unique-name-in-tag': ('enforced', '`attribute_error` maps `AttrError::Duplicated` to `DuplicateAttribute`'),
    'attributes.values-must-be-normalized': ('enforced', '`attribute_value_normalisation_in_both_directions`'),
    'attributes.whitespace-normalized-to-space': ('enforced', 'same'),
    'document-structure.cdata-section-must-end-with-cdend': ('enforced', 'the tokenizer only emits a CDATA event for a closed section'),
    'document-structure.cdata-section-must-start-with-cdstart': ('enforced', 'same'),
    'document-structure.cdata-sections-must-not-nest': ('enforced', 'the tokenizer treats CDATA content as opaque'),
    'document-structure.processor-must-normalize-line-breaks': ('enforced', '`BytesText::xml10_content` + `xml_spec::normalize_line_ends`; `line_ends_are_normalised_in_text`'),
    'document-structure.processor-must-pass-all-non-markup-characters': ('enforced', 'text is stored verbatim; `text_is_escaped_so_that_it_round_trips`'),
    'elements-and-tags.end-tag-must-match-start-tag': ('enforced', 'tokenizer end-name check; `tags_must_nest_and_close`'),
    'elements-and-tags.every-start-tag-must-have-end-tag': ('enforced', 'the parser keeps its own element stack; `tags_must_nest_and_close`'),
    'elements-and-tags.no-external-entity-refs-in-attribute-values': ('enforced', 'as `attributes.no-external-entity-refs`'),
    'elements-and-tags.no-less-than-in-attribute-values': ('enforced', 'as `attributes.no-lt-in-values`'),
    'elements-and-tags.start-tag-syntax': ('enforced', 'attribute syntax errors come from the tokenizer, duplicates from this crate'),
    'elements-and-tags.unique-attribute-specification': ('enforced', 'raw duplicates and duplicates by expanded name are both rejected'),
    'entities.bom-encoding-detection': ('enforced', '`parse_bytes` strips a leading UTF-8 BOM; `an_optional_utf8_byte_order_mark_is_accepted`'),
    'entities.charref-legal-character': ('enforced', '`resolve_reference` checks the `Char` production; `illegal_character_references_are_rejected`'),
    'entities.default-encoding-utf8': ('enforced', 'UTF-8 is the only supported encoding'),
    'entities.document-entity-well-formed': ('partial', 'the document *is* checked to be well-formed (tags, single root); the DTD parts of the rule are out of scope'),
    'entities.encoding-must-match-declaration': ('enforced', '`unsupported_declarations_are_rejected`'),
    'entities.encoding-name-case-insensitive': ('enforced', '`xml_spec::declaration::encoding_is_utf8`; `the_declaration_is_interpreted_and_preserved`'),
    'entities.illegal-byte-sequence-fatal': ('enforced', '`parse_bytes` validates UTF-8 first; `invalid_utf8_is_a_typed_error`'),
    'entities.no-encoding-legal-utf-required': ('enforced', 'a declaration without `encoding` is accepted and validated as UTF-8'),
    'entities.only-referenced-entities-well-formed': ('partial', 'references are resolved (or rejected) correctly; entity *declarations* need DTD processing, which is out of scope'),
    'entities.predefined-entities-recognized': ('enforced', '`predefined_entities_are_expanded`'),
    'entities.unsupported-encoding-fatal': ('enforced', '`UnsupportedEncoding`; `unsupported_declarations_are_rejected`'),
    'entities.utf-utf16-support': ('deferred', "UTF-16 is out of the project's scope (AGENTS.md: XML 1.0 + UTF-8); a UTF-16 input is rejected as invalid UTF-8"),
    'entities.utf8-bom-optional': ('enforced', 'as `entities.bom-encoding-detection`'),
    'namespace-basics.ns-decl-attribute-syntax': ('enforced', '`Parser::namespace_declarations`'),
    'namespace-basics.ns-decl-value-uri-or-empty': ('enforced', 'an empty value is only accepted for the default namespace; `namespaces_are_resolved_and_reported`'),
    'namespace-usage.attributes-unique-expanded-name': ('enforced', 'duplicates by expanded name are rejected'),
    'namespace-usage.declarations-direct-or-internal-dtd': ('partial', 'only direct declarations exist here; declarations in an internal DTD subset are out of scope'),
    'namespace-usage.default-namespace-unprefixed-elements': ('enforced', 'unprefixed element names take the default namespace; `namespaces_are_resolved_and_reported`'),
    'namespace-usage.empty-default-namespace': ('enforced', '`an_empty_default_declaration_removes_the_default_namespace`'),
    'namespace-usage.no-colon-typed-attributes': ('enforced', 'attribute names are validated as `NCName` parts of a `QName`'),
    'namespace-usage.no-prefix-undeclaring': ('enforced', '`xmlns:p=""` is `InvalidNamespace`; `namespaces_are_resolved_and_reported`'),
    'namespace-usage.prefix-declared': ('enforced', '`undeclared namespace prefix`; `namespaces_are_resolved_and_reported`'),
    'namespace-usage.processor-report-wellformedness': ('enforced', 'namespace well-formedness failures are reported as typed errors, never silently ignored'),
    'well-formedness.char-ref-legal-char': ('enforced', 'as `entities.charref-legal-character`'),
    'well-formedness.document-production': ('enforced', 'single root plus the documented top-level policy; `there_must_be_exactly_one_root_element`, `top_level_content_policy`'),
    'well-formedness.elements-nest-properly': ('enforced', '`tags_must_nest_and_close`'),
    'well-formedness.entities-must-be-well-formed': ('partial', 'references must be well formed and predefined; entity declarations are out of scope'),
    'well-formedness.no-peref-in-comments': ('partial', 'references inside comments are never expanded (this crate stores comment text verbatim); parameter entities do not exist without DTD processing'),
    'well-formedness.no-peref-in-pis': ('partial', 'same, for processing-instruction content'),
    'well-formedness.pi-pass-through': ('enforced', '`processing_instruction_content_is_kept_verbatim`'),
    'well-formedness.single-root-element': ('enforced', '`MultipleRootElements`; `there_must_be_exactly_one_root_element`'),
    'well-formedness.utf-8-utf-16-support': ('deferred', 'UTF-16 is out of scope; UTF-8 is fully supported (see `entities.utf-utf16-support`)'),
    'well-formedness.whitespace-definition': ('enforced', 'the `S` production is used for the declaration separator and for recognising ignorable top-level whitespace'),
    'attributes.id-must-be-name': ('enforced', '`check_xml_attribute_values` rejects an `xml:id` that is not an `NCName`; `xml_id_values_must_be_names_and_unique`'),
    'attributes.id-must-be-unique': ('enforced', '`check_xml_id_uniqueness` walks the attached tree; `xml_id_values_must_be_names_and_unique`'),
    'document-structure.xml-lang-empty-must-override-ancestor': ('n/a', 'prescribes how a processor resolves the language of an element; this crate exposes no API that consumes `xml:lang`, so there is nothing to check'),
    'document-structure.xml-lang-must-be-declared': ('deferred', '"MUST be declared" is an attribute-list (DTD) constraint; without DTD processing there is no declaration to check. The checkable half, the value shape, is enforced via `rule.document-structure.xml-lang-must-be-bcp47-or-empty`'),
    'document-structure.xml-lang-must-inherit-to-descendants': ('n/a', 'as `xml-lang-empty-must-override-ancestor`: processor behaviour for a feature this crate does not model'),
    'document-structure.xml-space-must-be-declared': ('deferred', 'as `xml-lang-must-be-declared`; the value shape is enforced via `rule.document-structure.xml-space-must-be-enumerated-default-preserve`'),
    'namespace-basics.xmlns-not-element-prefix': ('enforced', '`Namespace::prefixed`/`Namespace::new` refuse the reserved prefix at construction (layer A); `ScopeViolation::ReservedPrefix` re-checks it during validation, though no public constructor can reach that state'),
    'namespace-usage.default-namespace-scope': ('enforced', '`check_element_name`/`check_attribute_name`; `default_namespace_scope_must_match_the_name`, `attributes_are_not_affected_by_the_default_namespace`'),
    'namespace-usage.prefix-declaration-scope': ('enforced', '`NamespaceScope::from_declarations` plus `check_element_name`; `a_prefix_bound_to_a_different_uri_is_reported`, `editing_is_silent_and_validation_is_what_reports`'),
    'elements-and-tags.empty-element-representation': ('enforced', '`EmptyElementStyle`; `empty_elements_and_write_options`'),
    'well-formedness.escape-ampersand-and-lt': ('enforced', '`escape_text`/`escape_attribute`; `text_is_escaped_so_that_it_round_trips`'),
}


def rule_files() -> dict[str, str]:
    """Every rule summary as `identifier -> file name`.

    The identifier is the file name without the `rule.` prefix and the `.md` suffix
    (`attributes.id-must-be-unique`), which is the spelling the layer and verdict tables use.
    """
    files = {}
    for path in sorted(RULES.glob("rule.*.md")):
        identifier = path.name[len("rule.") : -len(".md")]
        files[identifier] = path.name
    if not files:
        sys.exit(f"no rule summaries found in {RULES}")
    return files


def rule_set_modules() -> set[str]:
    """The module names of the generated rule set, as a cross-check on the file list."""
    if not RULE_SET.is_file():
        sys.exit(f"{RULE_SET.relative_to(ROOT)} is missing: run scripts/gen_rule_set.py first")
    text = RULE_SET.read_text(encoding="utf-8")
    return set(re.findall(r"^pub mod ([a-z0-9_]+) \{\}$", text, re.M))


def classify(identifier: str) -> tuple[str, str]:
    """The `(layer, note)` of a rule, from the explicit table or its section default."""
    if identifier in LAYERS:
        return LAYERS[identifier]
    section = identifier.split(".", 1)[0]
    if section in DEFAULT_LAYERS:
        return DEFAULT_LAYERS[section], "default for this specification section"
    sys.exit(f"{identifier} has no layer: add it to LAYERS or {section!r} to DEFAULT_LAYERS")


def rows(files: dict[str, str]) -> list[tuple[str, str, str, str, str]]:
    """One `(section, rule, layer, status, evidence)` row per rule, in file order."""
    table = []
    for identifier in files:
        section = identifier.split(".", 1)[0]
        layer, note = classify(identifier)
        kind = layer.split(":", 1)[0]
        if kind in ("B", "C", "D"):
            if identifier not in VERDICTS:
                sys.exit(f"{identifier} is layer {kind} but has no verdict in VERDICTS")
            verdict, evidence = VERDICTS[identifier]
            status = STATUS[verdict]
            evidence = f"{note}; {evidence}"
        else:
            if identifier in VERDICTS:
                sys.exit(f"VERDICTS has an entry for {identifier}, which is not layer B, C or D")
            status = LAYER_STATUS[kind]
            evidence = note
        table.append((section, identifier, layer, status, evidence))

    expected_modules = {re.sub(r"[.-]", "_", name[: -len(".md")]) for name in files.values()}
    if expected_modules != rule_set_modules():
        missing = sorted(expected_modules - rule_set_modules())
        extra = sorted(rule_set_modules() - expected_modules)
        sys.exit(f"the generated rule set is out of sync (missing: {missing}, extra: {extra})")
    return table


def render(table: list[tuple[str, str, str, str, str]]) -> str:
    """The coverage document."""
    counts: dict[str, int] = {}
    for _, _, layer, _, _ in table:
        counts[layer] = counts.get(layer, 0) + 1
    verdicts: dict[str, int] = {}
    for identifier, (verdict, _) in VERDICTS.items():
        verdicts[verdict] = verdicts.get(verdict, 0) + 1

    lines = [
        "# Rule coverage",
        "",
        "Which specification rules this library acts on, in which layer, and how complete that is.",
        "The file is generated by `scripts/gen_rule_coverage.py`; the classification it renders is",
        "hand-written, because it is a claim about the code and its tests rather than something that",
        "can be derived from the specification text.",
        "",
        "| layer | rule summaries |",
        "| --- | --- |",
    ]
    for layer in ORDER:
        lines.append(f"| {layer} | {counts.get(layer, 0)} |")
    lines += [
        "",
        "Every layer-B, layer-C and layer-D rule carries a verdict: "
        + ", ".join(f"{verdicts.get(name, 0)} {name}" for name in ("enforced", "partial", "deferred", "n/a"))
        + ".",
        "`enforced` means the code path named in the table is covered by the test named with it;",
        "`partial` means the expressible part is implemented and the remainder needs DTD processing;",
        "`deferred` means it is deliberately not implemented. Layer `X` rules need DTD processing,",
        "validity checking or non-UTF-8 encodings, all of which are out of scope; layer `N/A` rules",
        "prescribe processor behaviour that no API of this library exposes.",
        "",
    ]
    for section in sorted({row[0] for row in table}):
        section_rows = [row for row in table if row[0] == section]
        lines += [
            f"## `{section}`",
            "",
            "| rule | layer | status | where it happens / why not |",
            "| --- | --- | --- | --- |",
        ]
        for _, identifier, layer, status, evidence in section_rows:
            lines.append(f"| `rule.{identifier}` | {layer} | {status} | {evidence} |")
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail instead of writing when out of date")
    arguments = parser.parse_args()

    generated = render(rows(rule_files()))

    if arguments.check:
        current = TARGET.read_text(encoding="utf-8") if TARGET.is_file() else ""
        if current != generated:
            print(
                f"{TARGET.relative_to(ROOT)} is out of date: run scripts/gen_rule_coverage.py",
                file=sys.stderr,
            )
            for line in list(
                difflib.unified_diff(
                    current.splitlines(), generated.splitlines(), "on disk", "generated", lineterm=""
                )
            )[:40]:
                print(line, file=sys.stderr)
            return 1
        print(f"{TARGET.relative_to(ROOT)}: up to date")
        return 0

    TARGET.write_text(generated, encoding="utf-8")
    print(f"{TARGET.relative_to(ROOT)}: {len(rule_files())} rules written")
    return 0


if __name__ == "__main__":
    sys.exit(main())
