//! Parser and serializer behaviour: well-formedness, entities, namespaces, escaping, options and
//! round-trip fidelity.
//!
//! Rules that this crate deliberately does *not* act on (DTD validity, non-UTF-8 encodings) are
//! listed in the rule coverage document; everything asserted here is annotated with the rule it
//! implements, and `rules_referenced_by_the_parser_and_serializer_exist` checks that each named
//! rule still has a summary.

mod common;

use biodivine_lib_xml_dom::xml_spec::rules;
use biodivine_lib_xml_dom::{
    DeclarationStyle, Document, EmptyElementStyle, Namespace, NodeContent, QualifiedName,
    WriteOptions, XmlError,
};

use common::element;

/// Parses and returns the text content of the root element's first text child.
fn root_text(xml: &str) -> String {
    let document = Document::parse_string(xml).unwrap();
    let root = document.root().unwrap();
    root.children()
        .iter()
        .filter_map(|node| node.text())
        .map(|text| text.as_str().to_string())
        .collect()
}

/// Asserts that parsing fails and returns the error.
fn parse_error(xml: &str) -> XmlError {
    Document::parse_string(xml).expect_err("the input was expected to be rejected")
}

// ---------------------------------------------------------------------------------------------
// Entities and character references
// ---------------------------------------------------------------------------------------------

#[test]
fn predefined_entities_are_expanded() {
    // rule: rule.entities.predefined-entities-recognized
    rules::assert_rule_exists("rule.entities.predefined-entities-recognized");
    assert_eq!(
        root_text(r#"<a>&amp; &lt; &gt; &apos; &quot;</a>"#),
        "& < > ' \""
    );
    // The reference may be split across text chunks.
    assert_eq!(root_text("<a>AT&amp;T</a>"), "AT&T");
    assert_eq!(root_text("<a>&amp;&amp;&amp;</a>"), "&&&");
}

#[test]
fn character_references_are_expanded() {
    // rule: rule.entities.charref-legal-character
    // rule: rule.well-formedness.char-ref-legal-char
    rules::assert_rule_exists("rule.entities.charref-legal-character");
    rules::assert_rule_exists("rule.well-formedness.char-ref-legal-char");
    assert_eq!(root_text("<a>&#65;&#x42;&#x4a;</a>"), "ABJ");
    assert_eq!(root_text("<a>&#x9;&#xA;&#xD;</a>"), "\t\n\r");
}

#[test]
fn illegal_character_references_are_rejected() {
    // rule: rule.entities.charref-legal-character
    assert!(matches!(
        parse_error("<a>&#x0;</a>"),
        XmlError::InvalidCharacterReference(_)
    ));
    assert!(matches!(
        parse_error("<a>&#x1;</a>"),
        XmlError::InvalidCharacterReference(_)
    ));
    assert!(matches!(
        parse_error("<a>&#xD800;</a>"),
        XmlError::InvalidCharacterReference(_)
    ));
    assert!(matches!(
        parse_error("<a>&#x110000;</a>"),
        XmlError::InvalidCharacterReference(_)
    ));
}

#[test]
fn undeclared_entity_references_are_typed_errors_not_panics() {
    // rule: rule.attributes.no-undeclared-entity-refs
    // rule: rule.entities.entity-declared-wfc
    rules::assert_rule_exists("rule.attributes.no-undeclared-entity-refs");
    for input in [
        "<a>&undefined;</a>",
        "<a>&custom;</a>",
        "<a b=\"&undefined;\"/>",
    ] {
        assert!(
            matches!(parse_error(input), XmlError::UndeclaredEntityReference(_)),
            "input: {input}"
        );
    }
}

#[test]
fn line_ends_are_normalised_in_text() {
    // rule: rule.document-structure.processor-must-normalize-line-breaks
    rules::assert_rule_exists("rule.document-structure.processor-must-normalize-line-breaks");
    assert_eq!(root_text("<a>x\r\ny</a>"), "x\ny");
    assert_eq!(root_text("<a>x\ry</a>"), "x\ny");
    assert_eq!(root_text("<a>x\r\r\ny</a>"), "x\n\ny");
}

#[test]
fn attribute_value_normalisation_in_both_directions() {
    // rule: rule.attributes.values-must-be-normalized
    // rule: rule.attributes.whitespace-normalized-to-space
    // rule: rule.attributes.linebreaks-normalized-to-lf
    // rule: rule.attributes.char-refs-expanded
    // rule: rule.attributes.entity-refs-expanded
    rules::assert_rule_exists("rule.attributes.values-must-be-normalized");
    rules::assert_rule_exists("rule.attributes.whitespace-normalized-to-space");
    rules::assert_rule_exists("rule.attributes.linebreaks-normalized-to-lf");

    // A *literal* whitespace character (tab, line feed, carriage return) becomes a space.
    let document = Document::parse_string("<a b=\"x\ty\"/>").unwrap();
    assert_eq!(attribute(&document, "b"), "x y");
    let document = Document::parse_string("<a b=\"x\ny\"/>").unwrap();
    assert_eq!(attribute(&document, "b"), "x y");
    let document = Document::parse_string("<a b=\"x\r\ny\"/>").unwrap();
    assert_eq!(attribute(&document, "b"), "x y");

    // A *character reference* is not affected by normalisation, so it survives.
    let document = Document::parse_string("<a b=\"x&#x9;y\"/>").unwrap();
    assert_eq!(attribute(&document, "b"), "x\ty");
    let document = Document::parse_string("<a b=\"x&#xA;y\"/>").unwrap();
    assert_eq!(attribute(&document, "b"), "x\ny");
    let document = Document::parse_string("<a b=\"x&#xD;y\"/>").unwrap();
    assert_eq!(attribute(&document, "b"), "x\ry");

    // And the serializer writes those characters back as character references, so a value
    // containing whitespace round-trips byte-exactly.
    for value in ["x\ty", "x\ny", "x\ry", "x\ty\nz\rw"] {
        let document = Document::empty();
        let root = element(&document, "a");
        document.set_root(root.clone());
        root.set_attribute(QualifiedName::without_namespace("b").unwrap(), value);
        let serialized = document.write_string().unwrap();
        assert!(
            !serialized.contains('\t') || value.contains('\t'),
            "a literal tab must not appear in an attribute value: {serialized}"
        );
        let reparsed = Document::parse_string(&serialized).unwrap();
        assert_eq!(attribute(&reparsed, "b"), value, "output: {serialized}");
    }

    // Markup characters are expanded on parse and re-escaped on write.
    let document = Document::parse_string("<a b=\"&lt;&amp;&quot;\"/>").unwrap();
    assert_eq!(attribute(&document, "b"), "<&\"");
    assert_eq!(
        document.write_string().unwrap(),
        "<a b=\"&lt;&amp;&quot;\"/>"
    );
}

/// The value of the no-namespace attribute `name` of the root element.
fn attribute(document: &Document, name: &str) -> String {
    document
        .root()
        .unwrap()
        .attribute(&QualifiedName::without_namespace(name).unwrap())
        .expect("attribute must be present")
        .to_string()
}

// ---------------------------------------------------------------------------------------------
// The XML declaration and encodings
// ---------------------------------------------------------------------------------------------

#[test]
fn the_declaration_is_interpreted_and_preserved() {
    // rule: rule.entities.encoding-must-match-declaration
    // rule: rule.entities.encoding-name-case-insensitive
    // rule: rule.entities.default-encoding-utf8
    // rule: rule.entities.no-encoding-legal-utf-required
    for encoding in ["UTF-8", "utf-8", "Utf8"] {
        let xml = format!(r#"<?xml version="1.0" encoding="{encoding}"?><a/>"#);
        let document = Document::parse_string(&xml).unwrap();
        assert!(document.xml_declaration().is_some(), "encoding {encoding}");
    }

    let document =
        Document::parse_string(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a/>"#)
            .unwrap();
    let declaration = document.xml_declaration().unwrap();
    assert_eq!(declaration.version(), "1.0");
    assert_eq!(declaration.encoding(), Some("UTF-8"));
    assert_eq!(declaration.standalone(), Some(true));
    assert_eq!(
        document.write_string().unwrap(),
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a/>"#
    );

    // A declaration without an encoding is legal.
    let document = Document::parse_string(r#"<?xml version="1.0"?><a/>"#).unwrap();
    assert_eq!(document.xml_declaration().unwrap().encoding(), None);
    assert_eq!(
        document.write_string().unwrap(),
        r#"<?xml version="1.0"?><a/>"#
    );
}

#[test]
fn unsupported_declarations_are_rejected() {
    assert!(matches!(
        parse_error(r#"<?xml version="1.1"?><a/>"#),
        XmlError::UnsupportedXmlVersion(_)
    ));
    assert!(matches!(
        parse_error(r#"<?xml version="1.0" encoding="ISO-8859-1"?><a/>"#),
        XmlError::UnsupportedEncoding(_)
    ));
    assert!(matches!(
        parse_error(r#"<?xml version="1.0" encoding="UTF-16"?><a/>"#),
        XmlError::UnsupportedEncoding(_)
    ));
    assert!(matches!(
        parse_error(r#"<?xml version="1.0" standalone="maybe"?><a/>"#),
        XmlError::MalformedXml(_)
    ));
}

#[test]
fn a_declaration_must_be_the_very_first_thing() {
    // rule: rule.well-formedness.pi-target-not-xml
    rules::assert_rule_exists("rule.well-formedness.pi-target-not-xml");

    assert!(matches!(
        parse_error(r#"<a><?xml version="1.0"?></a>"#),
        XmlError::InvalidProcessingInstruction(_)
    ));
    assert!(matches!(
        parse_error(r#" <?xml version="1.0"?><a/>"#),
        XmlError::InvalidProcessingInstruction(_)
    ));
    // `<?xml-stylesheet?>` is an ordinary processing instruction, not a declaration.
    let document = Document::parse_string(r#"<?xml-stylesheet href="a.css"?><a/>"#).unwrap();
    assert!(document.xml_declaration().is_none());
}

#[test]
fn an_optional_utf8_byte_order_mark_is_accepted() {
    // rule: rule.entities.utf8-bom-optional
    rules::assert_rule_exists("rule.entities.utf8-bom-optional");
    let with_bom = Document::parse_bytes(b"\xEF\xBB\xBF<a/>").unwrap();
    assert_eq!(with_bom.write_string().unwrap(), "<a/>");

    let with_bom_and_declaration =
        Document::parse_bytes(b"\xEF\xBB\xBF<?xml version=\"1.0\"?><a/>").unwrap();
    assert!(with_bom_and_declaration.xml_declaration().is_some());
}

#[test]
fn invalid_utf8_is_a_typed_error() {
    // rule: rule.entities.illegal-byte-sequence-fatal
    rules::assert_rule_exists("rule.entities.illegal-byte-sequence-fatal");
    assert!(matches!(
        Document::parse_bytes(b"<a>\xFF\xFE</a>"),
        Err(XmlError::InvalidUtf8(_))
    ));
    assert!(matches!(
        Document::parse_bytes(&[0x3C, 0x61, 0x3E, 0xC3, 0x28]),
        Err(XmlError::InvalidUtf8(_))
    ));
}

// ---------------------------------------------------------------------------------------------
// Well-formedness
// ---------------------------------------------------------------------------------------------

#[test]
fn there_must_be_exactly_one_root_element() {
    // rule: rule.well-formedness.single-root-element
    rules::assert_rule_exists("rule.well-formedness.single-root-element");
    assert!(matches!(
        parse_error("<a/><b/>"),
        XmlError::MultipleRootElements
    ));
    assert!(matches!(
        parse_error("<a></a><b></b>"),
        XmlError::MultipleRootElements
    ));
    assert!(matches!(parse_error(""), XmlError::MissingRoot));
    assert!(matches!(
        parse_error("<?xml version=\"1.0\"?>"),
        XmlError::MissingRoot
    ));
    assert!(matches!(parse_error("   "), XmlError::MissingRoot));
}

#[test]
fn top_level_content_policy() {
    // rule: rule.well-formedness.document-production
    rules::assert_rule_exists("rule.well-formedness.document-production");

    // Whitespace around the root is allowed and is not represented.
    assert_eq!(
        Document::parse_string("  <a/>  ")
            .unwrap()
            .write_string()
            .unwrap(),
        "<a/>"
    );
    // Whitespace *inside* the root is content and is preserved verbatim (this library does not
    // reformat documents); only the whitespace around the root is dropped.
    assert_eq!(
        Document::parse_string("\n<a>\n  <b/>\n</a>\n")
            .unwrap()
            .write_string()
            .unwrap(),
        "<a>\n  <b/>\n</a>"
    );

    // Comments and processing instructions outside the root are accepted and discarded: the data
    // model has exactly one root and nothing else at the top level. This is the documented default
    // behaviour (see the crate documentation).
    assert_eq!(
        Document::parse_string("<!-- c --><?pi d?><a/><!-- e -->")
            .unwrap()
            .write_string()
            .unwrap(),
        "<a/>"
    );
    assert!(
        Document::parse_string("<!--c--><a/>")
            .unwrap()
            .root()
            .is_some()
    );

    // Non-whitespace text and a second element are errors.
    assert!(matches!(
        parse_error("text<a/>"),
        XmlError::ContentOutsideRoot
    ));
    assert!(matches!(
        parse_error("<a/>text"),
        XmlError::ContentOutsideRoot
    ));
    assert!(matches!(
        parse_error("<a></a><![CDATA[x]]>"),
        XmlError::ContentOutsideRoot
    ));
}

#[test]
fn tags_must_nest_and_close() {
    // rule: rule.elements-and-tags.end-tag-must-match-start-tag
    // rule: rule.well-formedness.elements-nest-properly
    // rule: rule.elements-and-tags.every-start-tag-must-have-end-tag
    rules::assert_rule_exists("rule.elements-and-tags.end-tag-must-match-start-tag");
    rules::assert_rule_exists("rule.well-formedness.elements-nest-properly");
    rules::assert_rule_exists("rule.elements-and-tags.every-start-tag-must-have-end-tag");

    assert!(matches!(parse_error("<a>"), XmlError::MalformedXml(_)));
    assert!(matches!(
        parse_error("<a><b></a>"),
        XmlError::MalformedXml(_)
    ));
    assert!(matches!(parse_error("<a></b>"), XmlError::MalformedXml(_)));
    assert!(matches!(parse_error("</a>"), XmlError::MalformedXml(_)));
    assert!(matches!(
        parse_error("<a><b></b>"),
        XmlError::MalformedXml(_)
    ));
}

#[test]
fn namespaces_are_resolved_and_reported() {
    let document = Document::parse_string(
        r#"<h:root xmlns:h="http://h" xmlns="http://d"><child h:attr="v"/></h:root>"#,
    )
    .unwrap();
    let root = document.root().unwrap();
    assert_eq!(root.qualified_name().to_string(), "h:root");
    let child = root.child_elements()[0].clone();
    assert_eq!(child.local_name(), "child");
    assert_eq!(child.namespace().unwrap().uri(), "http://d");
    assert_eq!(
        child
            .attribute(
                &QualifiedName::with_namespace(
                    "attr",
                    &Namespace::prefixed("http://h", "h").unwrap()
                )
                .unwrap()
            )
            .unwrap()
            .as_ref(),
        "v"
    );

    // rule: rule.namespace-usage.prefix-declared
    // rule: rule.namespace-usage.no-prefix-undeclaring
    // rule: rule.namespace-usage.attributes-unique-expanded-name
    rules::assert_rule_exists("rule.namespace-usage.prefix-declared");
    rules::assert_rule_exists("rule.namespace-usage.no-prefix-undeclaring");
    rules::assert_rule_exists("rule.namespace-usage.attributes-unique-expanded-name");

    assert!(matches!(
        parse_error("<p:a/>"),
        XmlError::UndeclaredPrefix(_)
    ));
    assert!(matches!(
        parse_error(r#"<a xmlns:p=""/>"#),
        XmlError::InvalidNamespace(_)
    ));
    assert!(matches!(
        parse_error(r#"<a xmlns:p="http://p" p:x="1" p:x="2"/>"#),
        XmlError::DuplicateAttribute(_)
    ));
    assert!(matches!(
        parse_error(r#"<a x="1" x="2"/>"#),
        XmlError::DuplicateAttribute(_)
    ));
    // A reserved prefix is refused.
    assert!(matches!(
        parse_error(r#"<a xmlns:xmlns="http://x"/>"#),
        XmlError::ReservedPrefix(_)
    ));
}

#[test]
fn an_empty_default_declaration_removes_the_default_namespace() {
    // rule: rule.namespace-usage.empty-default-namespace
    rules::assert_rule_exists("rule.namespace-usage.empty-default-namespace");
    let document =
        Document::parse_string(r#"<root xmlns="http://d"><inner xmlns=""><leaf/></inner></root>"#)
            .unwrap();
    let root = document.root().unwrap();
    assert_eq!(root.namespace().unwrap().uri(), "http://d");
    let inner = root.child_elements()[0].clone();
    assert!(inner.namespace().is_none());
    assert_eq!(inner.namespace_declarations().get(&None), Some(&None));
    let leaf = inner.child_elements()[0].clone();
    assert!(leaf.namespace().is_none());

    // The empty declaration survives a round trip.
    let serialized = document.write_string().unwrap();
    assert!(serialized.contains(r#"xmlns="""#), "{serialized}");
    let reparsed = Document::parse_string(&serialized).unwrap();
    assert!(
        reparsed.root().unwrap().child_elements()[0]
            .namespace()
            .is_none()
    );
}

#[test]
fn xml_lang_and_xml_space_round_trip_with_the_implicit_xml_prefix() {
    // rule: rule.namespace-basics.xml-prefix-fixed-binding
    // rule: rule.document-structure.xml-lang-must-be-bcp47-or-empty
    // rule: rule.document-structure.xml-space-must-be-enumerated-default-preserve
    rules::assert_rule_exists("rule.namespace-basics.xml-prefix-fixed-binding");
    rules::assert_rule_exists("rule.document-structure.xml-lang-must-be-bcp47-or-empty");
    rules::assert_rule_exists(
        "rule.document-structure.xml-space-must-be-enumerated-default-preserve",
    );

    let xml = r#"<a xml:lang="en-GB" xml:space="preserve"><b/></a>"#;
    let document = Document::parse_string(xml).unwrap();
    let root = document.root().unwrap();

    // The `xml` prefix is bound without any declaration (Namespaces 1.0 §3), so `xml:lang` is an
    // attribute in the reserved XML namespace and the element stores no declaration for it.
    let xml_lang = QualifiedName::with_namespace(
        "lang",
        &Namespace::prefixed("http://www.w3.org/XML/1998/namespace", "xml").unwrap(),
    )
    .unwrap();
    let xml_space = QualifiedName::with_namespace(
        "space",
        &Namespace::prefixed("http://www.w3.org/XML/1998/namespace", "xml").unwrap(),
    )
    .unwrap();
    assert!(root.namespace_declarations().is_empty());
    assert_eq!(root.attribute(&xml_lang).unwrap().as_ref(), "en-GB");
    assert_eq!(root.attribute(&xml_space).unwrap().as_ref(), "preserve");
    // An unprefixed attribute called `lang` would be a *different* name with no namespace.
    assert!(
        root.attribute(&QualifiedName::without_namespace("lang").unwrap())
            .is_none()
    );
    assert!(
        root.resolve_attribute_name("xml:lang")
            .unwrap()
            .namespace()
            .is_some(),
        "`xml` must resolve without a declaration"
    );

    // The value predicates G4 will use accept these values ...
    assert!(biodivine_lib_xml_dom::xml_spec::is_valid_language_tag(
        "en-GB"
    ));
    assert!(biodivine_lib_xml_dom::xml_spec::is_valid_xml_space_value(
        "preserve"
    ));

    // ... and the document round-trips byte for byte.
    assert_eq!(document.write_string().unwrap(), xml);
    let reparsed = Document::parse_string(&document.write_string().unwrap()).unwrap();
    assert_eq!(reparsed.write_string().unwrap(), xml);

    // A value that breaks the `xml:space` enumeration is *validity*, not well-formedness: the
    // parser stores it (no DTD means no attribute types) and `is_valid_xml_space_value` rejects
    // it, so the whole-document validation pass (G4) is what reports it.
    let loose = Document::parse_string(r#"<a xml:space="preserved"/>"#).unwrap();
    assert_eq!(
        loose
            .root()
            .unwrap()
            .attribute(&xml_space)
            .unwrap()
            .as_ref(),
        "preserved"
    );
    assert!(!biodivine_lib_xml_dom::xml_spec::is_valid_xml_space_value(
        "preserved"
    ));
    assert!(!biodivine_lib_xml_dom::xml_spec::is_valid_language_tag(
        "en_US_very_long_invalid"
    ));
}

#[test]
fn less_than_is_rejected_in_attribute_values() {
    // rule: rule.attributes.no-lt-in-values
    // rule: rule.elements-and-tags.no-less-than-in-attribute-values
    rules::assert_rule_exists("rule.attributes.no-lt-in-values");
    assert!(matches!(
        parse_error(r#"<a b="x<y"/>"#),
        XmlError::MalformedXml(_)
    ));
    // ... but an escaped one is fine.
    assert_eq!(
        attribute(&Document::parse_string(r#"<a b="x&lt;y"/>"#).unwrap(), "b"),
        "x<y"
    );
}

#[test]
fn comments_must_be_well_formed() {
    // rule: rule.well-formedness.comment-no-double-hyphen
    rules::assert_rule_exists("rule.well-formedness.comment-no-double-hyphen");
    assert!(matches!(
        parse_error("<a><!-- bad -- comment --></a>"),
        XmlError::InvalidComment(_) | XmlError::MalformedXml(_)
    ));
    // A comment whose content actually *ends* with a hyphen: the first `-->` closes the section,
    // leaving `x-` as the content, which the specification forbids.
    assert!(matches!(
        parse_error("<a><!--x---></a>"),
        XmlError::InvalidComment(_) | XmlError::MalformedXml(_)
    ));
    assert!(Document::parse_string("<a><!-- fine --></a>").is_ok());
    assert!(Document::parse_string("<a><!-- ends with - --></a>").is_ok());
}

#[test]
fn cdata_and_processing_instructions_must_be_well_formed() {
    // rule: rule.document-structure.cdata-section-must-not-contain-cdend
    // rule: rule.well-formedness.pi-target-is-name
    // rule: rule.well-formedness.pi-no-contains-close
    rules::assert_rule_exists("rule.document-structure.cdata-section-must-not-contain-cdend");
    rules::assert_rule_exists("rule.well-formedness.pi-target-is-name");
    rules::assert_rule_exists("rule.well-formedness.pi-no-contains-close");

    // `]]>` simply closes the section, so this is *valid*: the rest becomes text.
    let document = Document::parse_string("<a><![CDATA[x]]>y</a>").unwrap();
    let kinds: Vec<NodeContent> = document
        .root()
        .unwrap()
        .children()
        .iter()
        .map(|node| node.content())
        .collect();
    assert_eq!(kinds.len(), 2, "CDATA followed by text");

    assert!(matches!(
        parse_error("<a><?1bad d?></a>"),
        XmlError::InvalidProcessingInstruction(_) | XmlError::MalformedXml(_)
    ));
}

#[test]
fn parser_never_panics_on_odd_input() {
    // Rule-free smoke test: a selection of inputs that historically broke parsers.
    let inputs = [
        "",
        "<",
        "<a",
        "<a ",
        "<a/",
        "<a b",
        "<a b=",
        "<a b='",
        "<!",
        "<!--",
        "<!-- --",
        "<![CDATA[",
        "<![CDATA[]]]",
        "<a>&",
        "<a>&#",
        "<a>&#x",
        "<a>&;",
        "<a b='&",
        "<?",
        "<?x",
        "<a></",
        "<a></a",
        "]]>",
        "\u{0}",
        "<a>\u{0}</a>",
        "<a>\u{FFFF}</a>",
        "<:a/>",
        "<a:/>",
        "<a b:c:d='1'/>",
        "<a xmlns='http://'/>",
        "<a xmlns:x='http://x' x:a='1' a='2'/>",
    ];
    for input in inputs {
        let _ = Document::parse_string(input);
        let _ = Document::parse_bytes(input.as_bytes());
    }
}

// ---------------------------------------------------------------------------------------------
// The serializer
// ---------------------------------------------------------------------------------------------

#[test]
fn prefixes_are_preserved() {
    // This was REVIEW D2: the pre-rewrite serializer dropped the element prefix.
    let xml = r#"<html:html xmlns:html="http://www.w3.org/1999/xhtml"><html:body>hi</html:body></html:html>"#;
    let document = Document::parse_string(xml).unwrap();
    assert_eq!(document.write_string().unwrap(), xml);

    let document = Document::parse_string(xml).unwrap();
    assert!(document.root().unwrap().to_string().contains("<html:body>"));
}

#[test]
fn namespace_declarations_are_never_invented_or_removed() {
    let document = Document::empty();
    let ex = Namespace::prefixed("http://example.com", "ex").unwrap();
    let root = document.create_element(QualifiedName::with_namespace("item", &ex).unwrap());
    document.set_root(root.clone());

    // The name carries the namespace, but nothing declares the prefix: the serializer writes what
    // is there and does not "fix" the document.
    let serialized = document.write_string().unwrap();
    assert_eq!(serialized, "<ex:item/>");
    assert!(Document::parse_string(&serialized).is_err());

    // After declaring it, the document is consistent again.
    root.declare_namespace(ex);
    assert_eq!(
        document.write_string().unwrap(),
        r#"<ex:item xmlns:ex="http://example.com"/>"#
    );
}

#[test]
fn empty_elements_and_write_options() {
    let document = Document::parse_string("<a/>").unwrap();
    let default = WriteOptions::default();
    assert_eq!(default.declaration, DeclarationStyle::IfPresent);
    assert_eq!(default.empty_elements, EmptyElementStyle::SelfClosing);
    assert_eq!(document.write_string().unwrap(), "<a/>");
    assert_eq!(
        document
            .write_string_with(&WriteOptions {
                declaration: DeclarationStyle::Never,
                empty_elements: EmptyElementStyle::ExplicitEndTag,
            })
            .unwrap(),
        "<a></a>"
    );

    // A nested empty element inside a non-empty one.
    let document = Document::parse_string("<a><b/><c>text</c></a>").unwrap();
    assert_eq!(document.write_string().unwrap(), "<a><b/><c>text</c></a>");
    assert_eq!(
        document
            .write_string_with(&WriteOptions {
                declaration: DeclarationStyle::Never,
                empty_elements: EmptyElementStyle::ExplicitEndTag,
            })
            .unwrap(),
        "<a><b></b><c>text</c></a>"
    );
}

#[test]
fn declaration_style_is_honoured() {
    let with_declaration =
        Document::parse_string(r#"<?xml version="1.0" encoding="UTF-8"?><a/>"#).unwrap();
    let without = Document::parse_string("<a/>").unwrap();

    assert_eq!(
        with_declaration
            .write_string_with(&WriteOptions {
                declaration: DeclarationStyle::Never,
                ..Default::default()
            })
            .unwrap(),
        "<a/>"
    );
    assert_eq!(
        without
            .write_string_with(&WriteOptions {
                declaration: DeclarationStyle::Always,
                ..Default::default()
            })
            .unwrap(),
        r#"<?xml version="1.0" encoding="UTF-8"?><a/>"#
    );
    // Explicit setting is preserved.
    without.set_xml_declaration(Some(biodivine_lib_xml_dom::xml_spec::XmlDeclaration::new(
        "1.0",
        Some("UTF-8"),
        Some(false),
    )));
    assert_eq!(
        without.write_string().unwrap(),
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?><a/>"#
    );
}

#[test]
fn text_is_escaped_so_that_it_round_trips() {
    // rule: rule.well-formedness.escape-ampersand-and-lt
    rules::assert_rule_exists("rule.well-formedness.escape-ampersand-and-lt");
    let document = Document::empty();
    let root = element(&document, "a");
    document.set_root(root.clone());
    root.append_child(document.create_text("A & B < C > D").unwrap());

    let serialized = document.write_string().unwrap();
    assert_eq!(serialized, "<a>A &amp; B &lt; C &gt; D</a>");
    assert_eq!(root_text(&serialized), "A & B < C > D");
}

#[test]
fn a_literal_carriage_return_in_text_round_trips() {
    let document = Document::empty();
    let root = element(&document, "a");
    document.set_root(root.clone());
    root.append_child(document.create_text("x\ry\nz").unwrap());

    let serialized = document.write_string().unwrap();
    assert_eq!(
        serialized, "<a>x&#xD;y\nz</a>",
        "a literal CR would be normalised to LF by the next parse"
    );
    assert_eq!(root_text(&serialized), "x\ry\nz");
}

#[test]
fn text_containing_a_cdata_close_round_trips() {
    let document = Document::empty();
    let root = element(&document, "a");
    document.set_root(root.clone());
    root.append_child(document.create_text("a ]]> b").unwrap());
    let serialized = document.write_string().unwrap();
    assert!(serialized.contains("]]&gt;"), "{serialized}");
    assert_eq!(root_text(&serialized), "a ]]> b");
}

#[test]
fn adjacent_text_nodes_are_merged_on_output() {
    let document = Document::empty();
    let root = element(&document, "a");
    document.set_root(root.clone());
    root.append_child(document.create_text("one").unwrap());
    root.append_child(document.create_text("two").unwrap());
    root.append_child(element(&document, "b"));
    root.append_child(document.create_text("three").unwrap());
    root.append_child(document.create_text("four").unwrap());

    let serialized = document.write_string().unwrap();
    assert_eq!(serialized, "<a>onetwo<b/>threefour</a>");
    // Parsing the output again gives the same structure, so writing is idempotent.
    assert_eq!(
        Document::parse_string(&serialized)
            .unwrap()
            .write_string()
            .unwrap(),
        serialized
    );
}

#[test]
fn processing_instruction_content_is_kept_verbatim() {
    // rule: rule.well-formedness.pi-pass-through
    rules::assert_rule_exists("rule.well-formedness.pi-pass-through");

    // The whitespace separating target and content is syntax: exactly one character of it is
    // removed, the rest of the content is data and is preserved (including leading and trailing
    // whitespace, which the pre-rewrite implementation trimmed away).
    for (input, expected) in [
        ("<?a?>", ""),
        ("<?a ?>", ""),
        ("<?a x?>", "x"),
        ("<?a  x?>", " x"),
        ("<?a x ?>", "x "),
        ("<?a x y?>", "x y"),
        ("<?a echo \"hi\"; ?>", "echo \"hi\"; "),
    ] {
        let document = Document::parse_string(&format!("<r>{input}</r>")).unwrap();
        let (_, data) = document
            .root()
            .unwrap()
            .children()
            .iter()
            .find_map(|node| node.processing_instruction())
            .unwrap_or_else(|| panic!("no PI parsed from {input}"));
        assert_eq!(data.as_str(), expected, "input: {input}");

        // ... and the output re-parses to the same content.
        let serialized = document.write_string().unwrap();
        let reparsed = Document::parse_string(&serialized).unwrap();
        let (_, data) = reparsed
            .root()
            .unwrap()
            .children()
            .iter()
            .find_map(|node| node.processing_instruction())
            .unwrap();
        assert_eq!(data.as_str(), expected, "output: {serialized}");
    }
}

#[test]
fn empty_text_nodes_have_no_representation_and_are_omitted() {
    let document = Document::empty();
    let root = element(&document, "a");
    document.set_root(root.clone());
    root.append_child(document.create_text("").unwrap());
    root.append_child(document.create_text("text").unwrap());
    root.append_child(document.create_text("").unwrap());
    root.append_child(element(&document, "b"));
    root.append_child(document.create_text("").unwrap());

    assert_eq!(document.write_string().unwrap(), "<a>text<b/></a>");
    // Comments and CDATA sections have a representation for "empty" and keep it.
    let document = Document::empty();
    let root = element(&document, "a");
    document.set_root(root.clone());
    root.append_child(document.create_comment("").unwrap());
    root.append_child(document.create_cdata("").unwrap());
    assert_eq!(
        document.write_string().unwrap(),
        "<a><!----><![CDATA[]]></a>"
    );
}

#[test]
fn comments_cdata_and_processing_instructions_are_written_verbatim() {
    let document = Document::empty();
    let root = element(&document, "a");
    document.set_root(root.clone());
    root.append_child(
        document
            .create_comment(" note -- with dashes? no ")
            .is_err()
            .then(|| document.create_comment(" ok ").unwrap())
            .unwrap(),
    );
    root.append_child(document.create_cdata("raw <content> & more").unwrap());
    root.append_child(
        document
            .create_processing_instruction("target", "data=\"v\"")
            .unwrap(),
    );
    root.append_child(document.create_processing_instruction("empty", "").unwrap());

    let serialized = document.write_string().unwrap();
    assert_eq!(
        serialized,
        "<a><!-- ok --><![CDATA[raw <content> & more]]><?target data=\"v\"?><?empty?></a>"
    );
    let reparsed = Document::parse_string(&serialized).unwrap();
    assert_eq!(reparsed.write_string().unwrap(), serialized);
}

#[test]
fn a_document_without_a_root_serializes_to_nothing_useful_but_does_not_fail() {
    let document = Document::empty();
    assert_eq!(document.write_string().unwrap(), "");
    document.set_xml_declaration(Some(biodivine_lib_xml_dom::xml_spec::XmlDeclaration::utf8()));
    assert_eq!(
        document.write_string().unwrap(),
        r#"<?xml version="1.0" encoding="UTF-8"?>"#
    );
}

// ---------------------------------------------------------------------------------------------
// Depth
// ---------------------------------------------------------------------------------------------

#[test]
fn a_deeply_nested_document_round_trips() {
    // Serialization and parsing are both iterative, so neither can overflow the stack on a deeply
    // nested document (a stack overflow would abort the process, which no API could report).
    const DEPTH: usize = 5_000;

    let document = Document::empty();
    let root = element(&document, "root");
    document.set_root(root.clone());
    let mut current = root;
    for _ in 0..DEPTH {
        let child = element(&document, "child");
        current.append_child(child.clone());
        current = child;
    }

    let serialized = document.write_string().unwrap();
    // One `<child` per level, and exactly one of them is an empty-element tag.
    assert_eq!(serialized.matches("<child").count(), DEPTH);
    assert_eq!(serialized.matches("<child/>").count(), 1);

    let reparsed = Document::parse_string(&serialized).unwrap();
    assert_eq!(reparsed.root().unwrap().descendants().len(), DEPTH);
    assert_eq!(reparsed.write_string().unwrap(), serialized);
    // `Display` uses the same writer.
    assert_eq!(reparsed.root().unwrap().to_string(), serialized);
    // Cloning is iterative as well.
    assert_eq!(
        reparsed.root().unwrap().deep_clone().descendants().len(),
        DEPTH
    );
}

// ---------------------------------------------------------------------------------------------
// Rule anchors
// ---------------------------------------------------------------------------------------------

#[test]
fn rules_referenced_by_the_parser_and_serializer_exist() {
    // Every rule this module claims to implement must still have a summary file. This is the check
    // that keeps the annotations in `parse.rs`/`write.rs` honest, and it is what makes the
    // corresponding rows of the rule coverage document checkable rather than asserted.
    let rules_referenced = [
        "rule.attributes.char-ref-legal-char",
        "rule.attributes.char-refs-expanded",
        "rule.attributes.entity-refs-expanded",
        "rule.attributes.linebreaks-normalized-to-lf",
        "rule.attributes.no-lt-in-values",
        "rule.attributes.no-undeclared-entity-refs",
        "rule.attributes.values-must-be-normalized",
        "rule.attributes.whitespace-normalized-to-space",
        "rule.document-structure.cdata-section-must-not-contain-cdend",
        "rule.document-structure.processor-must-normalize-line-breaks",
        "rule.elements-and-tags.end-tag-must-match-start-tag",
        "rule.elements-and-tags.every-start-tag-must-have-end-tag",
        "rule.elements-and-tags.no-less-than-in-attribute-values",
        "rule.elements-and-tags.unique-attribute-specification",
        "rule.entities.charref-legal-character",
        "rule.entities.default-encoding-utf8",
        "rule.entities.encoding-must-match-declaration",
        "rule.entities.encoding-name-case-insensitive",
        "rule.entities.entity-declared-wfc",
        "rule.entities.illegal-byte-sequence-fatal",
        "rule.entities.no-encoding-legal-utf-required",
        "rule.entities.predefined-entities-recognized",
        "rule.entities.utf8-bom-optional",
        "rule.namespace-usage.attributes-unique-expanded-name",
        "rule.namespace-usage.empty-default-namespace",
        "rule.namespace-usage.no-prefix-undeclaring",
        "rule.namespace-usage.prefix-declaration-scope",
        "rule.namespace-usage.prefix-declared",
        "rule.well-formedness.char-ref-legal-char",
        "rule.well-formedness.comment-no-double-hyphen",
        "rule.well-formedness.document-production",
        "rule.well-formedness.elements-nest-properly",
        "rule.well-formedness.escape-ampersand-and-lt",
        "rule.well-formedness.pi-no-contains-close",
        "rule.well-formedness.pi-target-is-name",
        "rule.well-formedness.pi-target-not-xml",
        "rule.well-formedness.single-root-element",
    ];
    for rule in rules_referenced {
        assert!(
            rules::rule_summary_exists(rule),
            "the parser/serializer relies on {rule}, but its summary file is missing"
        );
    }
}
