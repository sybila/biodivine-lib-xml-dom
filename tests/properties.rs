//! Property tests for I/O.
//!
//! Three properties are checked, each with at least 256 (round trip) / 512 (no panics) generated
//! cases:
//!
//! 1. **Round trip.** For a generated *namespace-consistent* document, `parse(write(d))` is
//!    structurally identical to `d`. "Structurally identical" is compared through a canonical
//!    signature: expanded names (prefix + URI), the declarations stored on each element, attribute
//!    values, and the children in order with adjacent text merged. The generator is restricted to
//!    documents the data model can represent — that is the documented policy in `io::parse` (one
//!    root element, nothing else at the top level).
//! 2. **No panics.** Arbitrary bytes, arbitrary strings, and *mutations of well-formed documents*
//!    are all fed to the parser, which must return `Ok` or `Err` every time. A parser that panics
//!    (or aborts, which is what a stack overflow does) is a library defect regardless of whether
//!    the input is valid.
//! 3. **Idempotence.** `write(parse(write(d))) == write(d)`.

use biodivine_lib_xml_dom::{
    Document, Element, Namespace, Node, NodeContent, QualifiedName, parse_bytes, parse_string,
    write_string,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------------------------
// A canonical signature of a document
// ---------------------------------------------------------------------------------------------

/// A structural signature that ignores everything the XML data model does not prescribe
/// (attribute order, declaration order, the text/comment splitting the parser happens to see).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Sig {
    Element {
        name: String,
        declarations: Vec<(String, String)>,
        attributes: Vec<(String, String)>,
        children: Vec<Sig>,
    },
    Text(String),
    Comment(String),
    CData(String),
    Pi(String, String),
}

fn sig_document(document: &Document) -> Sig {
    sig_element(&document.root().expect("a generated document has a root"))
}

fn sig_element(element: &Element) -> Sig {
    let name = element.qualified_name();
    // `prefix|uri|local`, spelled exactly like the attribute signature below, so that a serializer
    // or parser that drops, renames or mangles an element's *local name* (the REVIEW D2 bug class)
    // is caught by the round-trip property rather than passing unnoticed.
    let name = format!(
        "{}|{}|{}",
        name.namespace()
            .and_then(|namespace| namespace.prefix_str())
            .unwrap_or(""),
        name.namespace()
            .map(|namespace| namespace.uri().to_string())
            .unwrap_or_default(),
        name.local_name(),
    );
    let declarations: Vec<(String, String)> = element
        .namespace_declarations()
        .into_iter()
        .map(|(prefix, namespace)| {
            (
                prefix
                    .map(|prefix| prefix.as_str().to_string())
                    .unwrap_or_default(),
                namespace
                    .map(|namespace| namespace.uri().to_string())
                    .unwrap_or_default(),
            )
        })
        .collect();
    let attributes: Vec<(String, String)> = element
        .attributes()
        .into_iter()
        .map(|(name, value)| {
            (
                format!(
                    "{}|{}|{}",
                    name.namespace()
                        .and_then(|namespace| namespace.prefix_str())
                        .unwrap_or(""),
                    name.namespace()
                        .map(|namespace| namespace.uri().to_string())
                        .unwrap_or_default(),
                    name.local_name(),
                ),
                value.to_string(),
            )
        })
        .collect();
    Sig::Element {
        name,
        declarations,
        attributes,
        children: sig_children(&element.children()),
    }
}

/// Signatures of a child list, with runs of adjacent text nodes merged (XML cannot express a
/// boundary between two text nodes, so the serializer merges them and the parser never produces
/// them) and empty text nodes dropped (an empty text node carries no character data and therefore
/// has no XML representation at all).
fn sig_children(children: &[Node]) -> Vec<Sig> {
    let mut result: Vec<Sig> = Vec::new();
    for child in children {
        match child.content() {
            NodeContent::Text(text) if text.as_str().is_empty() => {}
            NodeContent::Text(text) if matches!(result.last(), Some(Sig::Text(_))) => {
                let Some(Sig::Text(previous)) = result.last_mut() else {
                    unreachable!("checked by the match guard")
                };
                previous.push_str(text.as_str());
            }
            NodeContent::Text(text) => result.push(Sig::Text(text.as_str().to_string())),
            NodeContent::Element(element) => result.push(sig_element(&element)),
            NodeContent::Comment(comment) => {
                result.push(Sig::Comment(comment.as_str().to_string()))
            }
            NodeContent::CData(cdata) => result.push(Sig::CData(cdata.as_str().to_string())),
            NodeContent::ProcessingInstruction(target, data) => {
                result.push(Sig::Pi(
                    target.as_str().to_string(),
                    data.as_str().to_string(),
                ));
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------------------------
// The generator
// ---------------------------------------------------------------------------------------------

const NS_0: &str = "http://example.com/ns0";
const NS_1: &str = "http://example.com/ns1";
const NS_DEFAULT: &str = "http://example.com/default";

/// Which namespace an expanded name uses.
#[derive(Debug, Clone)]
enum NameChoice {
    /// No namespace at all (only legal for an unprefixed name when no default namespace is in
    /// scope, and always legal for attributes).
    None,
    /// The `p0` prefix.
    P0,
    /// The `p1` prefix.
    P1,
    /// The default namespace (elements only).
    Default,
}

#[derive(Debug, Clone)]
struct GenElement {
    /// The element's local name; varied so that the signature is sensitive to it.
    local: String,
    name: NameChoice,
    /// Which of `p0`, `p1`, the default namespace are (redundantly) declared on this element.
    redeclare: [bool; 3],
    /// Attribute names are unique by construction: the index is the attribute name.
    attributes: Vec<(u8, NameChoice, String)>,
    children: Vec<GenNode>,
}

#[derive(Debug, Clone)]
enum GenNode {
    Element(Box<GenElement>),
    Text(String),
    Comment(String),
    CData(String),
    Pi(String, String),
}

/// The document-level shape.
#[derive(Debug, Clone)]
struct GenDocument {
    /// Whether the root declares a default namespace.
    default_namespace: bool,
    root: GenElement,
}

fn arb_name_char() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('<'),
        Just('>'),
        Just('&'),
        Just('"'),
        Just('\''),
        Just('\t'),
        Just('\n'),
        Just('\r'),
        Just(']'),
        Just('-'),
        Just('?'),
        Just(' '),
        Just('|'),
        Just('a'),
        Just('Z'),
        Just('0'),
        Just('\u{e9}'),
        Just('\u{4e2d}'),
        Just('\u{1f600}'),
        Just('\u{fffd}'),
    ]
}

/// Content that always contains markup characters, whitespace and line breaks, so the escaping
/// and the attribute-value normalisation are actually exercised.
fn arb_content() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_name_char(), 0..14)
        .prop_map(|characters| characters.into_iter().collect())
}

fn arb_comment() -> impl Strategy<Value = String> {
    arb_content().prop_filter("comment content", |content| {
        !content.contains("--") && !content.ends_with('-')
    })
}

fn arb_cdata() -> impl Strategy<Value = String> {
    arb_content().prop_filter("CDATA content", |content| !content.contains("]]>"))
}

fn arb_pi_data() -> impl Strategy<Value = String> {
    arb_content().prop_filter("PI content", |content| !content.contains("?>"))
}

fn arb_local_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,6}"
        .prop_map(|name| name.to_string())
        .prop_filter("PI targets must not be `xml`", |name| {
            !name.eq_ignore_ascii_case("xml")
        })
}

fn arb_name_choice(allow_default: bool) -> impl Strategy<Value = NameChoice> {
    if allow_default {
        prop_oneof![
            Just(NameChoice::None),
            Just(NameChoice::P0),
            Just(NameChoice::P1),
            Just(NameChoice::Default),
        ]
        .boxed()
    } else {
        prop_oneof![
            Just(NameChoice::None),
            Just(NameChoice::P0),
            Just(NameChoice::P1)
        ]
        .boxed()
    }
}

fn arb_attribute() -> impl Strategy<Value = (u8, NameChoice, String)> {
    // Attributes are never affected by the default namespace, so they may always have "no
    // namespace"; the index makes the names unique within the element.
    (0u8..6, arb_name_choice(false), arb_content())
}

fn arb_element(budget: u32, default_in_scope: bool) -> BoxedStrategy<GenElement> {
    // An unprefixed element name only means "no namespace" if no default namespace is in scope.
    // Conversely, an element can only be *in* the default namespace if that default is in scope.
    let name = if default_in_scope {
        prop_oneof![
            Just(NameChoice::P0),
            Just(NameChoice::P1),
            Just(NameChoice::Default)
        ]
        .boxed()
    } else {
        prop_oneof![
            Just(NameChoice::None),
            Just(NameChoice::P0),
            Just(NameChoice::P1)
        ]
        .boxed()
    };
    let attributes = prop::collection::vec(arb_attribute(), 0..3).prop_map(|attributes| {
        let mut unique = BTreeMap::new();
        for (index, name, value) in attributes {
            unique.insert(index, (index, name, value));
        }
        unique.into_values().collect::<Vec<_>>()
    });
    let redeclare = (
        any::<bool>(),
        any::<bool>(),
        if default_in_scope {
            any::<bool>().boxed()
        } else {
            Just(false).boxed()
        },
    )
        .prop_map(|redeclare| [redeclare.0, redeclare.1, redeclare.2]);

    if budget == 0 {
        (
            arb_local_name(),
            name,
            redeclare,
            attributes,
            Just(Vec::new()),
        )
            .prop_map(
                |(local, name, redeclare, attributes, children)| GenElement {
                    local,
                    name,
                    redeclare,
                    attributes,
                    children,
                },
            )
            .boxed()
    } else {
        (
            arb_local_name(),
            name,
            redeclare,
            attributes,
            prop::collection::vec(arb_node(budget - 1, default_in_scope), 0..3),
        )
            .prop_map(
                |(local, name, redeclare, attributes, children)| GenElement {
                    local,
                    name,
                    redeclare,
                    attributes,
                    children,
                },
            )
            .boxed()
    }
}

fn arb_leaf() -> BoxedStrategy<GenNode> {
    prop_oneof![
        3 => arb_content().prop_map(GenNode::Text),
        1 => arb_comment().prop_map(GenNode::Comment),
        1 => arb_cdata().prop_map(GenNode::CData),
        1 => (arb_local_name(), arb_pi_data())
            .prop_map(|(target, data)| GenNode::Pi(target, data)),
    ]
    .boxed()
}

fn arb_node(budget: u32, default_in_scope: bool) -> BoxedStrategy<GenNode> {
    if budget == 0 {
        return arb_leaf();
    }
    let leaf = arb_leaf();
    let nested = leaf.clone();
    prop_oneof![
        3 => leaf,
        2 => arb_element(budget - 1, default_in_scope)
            .prop_map(|element| GenNode::Element(Box::new(element))),
        1 => nested.prop_map(|node| node),
    ]
    .boxed()
}

fn arb_document_consistent() -> impl Strategy<Value = GenDocument> {
    any::<bool>().prop_flat_map(|default_namespace| {
        arb_element(4, default_namespace).prop_map(move |mut root| {
            // The root decides whether a default namespace exists at all, and the generator only
            // produces names that are consistent with that choice (an unprefixed element name with
            // no default namespace in scope has no namespace; with one in scope it has that
            // namespace). This is what makes the generated documents representable and parseable.
            if !default_namespace && matches!(root.name, NameChoice::Default) {
                root.name = NameChoice::None;
            }
            GenDocument {
                default_namespace,
                root,
            }
        })
    })
}

// ---------------------------------------------------------------------------------------------
// Building a document from a generated description
// ---------------------------------------------------------------------------------------------

fn qualified(name: &NameChoice, local: &str) -> QualifiedName {
    match name {
        NameChoice::None => QualifiedName::without_namespace(local).unwrap(),
        NameChoice::P0 => {
            QualifiedName::with_namespace(local, &Namespace::prefixed(NS_0, "p0").unwrap()).unwrap()
        }
        NameChoice::P1 => {
            QualifiedName::with_namespace(local, &Namespace::prefixed(NS_1, "p1").unwrap()).unwrap()
        }
        NameChoice::Default => {
            QualifiedName::with_namespace(local, &Namespace::without_prefix(NS_DEFAULT).unwrap())
                .unwrap()
        }
    }
}

fn build_document(generated: &GenDocument) -> Document {
    let document = Document::empty();
    let root = build_element(
        &document,
        &generated.root,
        generated.default_namespace,
        true,
    );
    document.set_root(root);
    document
}

fn build_element(
    document: &Document,
    generated: &GenElement,
    in_default_scope: bool,
    is_root: bool,
) -> Element {
    let element = document.create_element(qualified(&generated.name, &generated.local));
    if is_root {
        element.declare_namespace(Namespace::prefixed(NS_0, "p0").unwrap());
        element.declare_namespace(Namespace::prefixed(NS_1, "p1").unwrap());
        if in_default_scope {
            element.declare_namespace(Namespace::without_prefix(NS_DEFAULT).unwrap());
        }
    } else {
        if generated.redeclare[0] {
            element.declare_namespace(Namespace::prefixed(NS_0, "p0").unwrap());
        }
        if generated.redeclare[1] {
            element.declare_namespace(Namespace::prefixed(NS_1, "p1").unwrap());
        }
        if generated.redeclare[2] && in_default_scope {
            element.declare_namespace(Namespace::without_prefix(NS_DEFAULT).unwrap());
        }
    }
    for (index, name, value) in &generated.attributes {
        element.set_attribute(qualified(name, &format!("a{index}")), value);
    }
    for child in &generated.children {
        match child {
            GenNode::Element(child) => {
                let child = build_element(document, child, in_default_scope, false);
                element.append_child(child);
            }
            GenNode::Text(text) => {
                element.append_child(document.create_text(text).expect("generated text is legal"))
            }
            GenNode::Comment(comment) => element.append_child(
                document
                    .create_comment(comment)
                    .expect("generated comment is legal"),
            ),
            GenNode::CData(cdata) => element.append_child(
                document
                    .create_cdata(cdata)
                    .expect("generated CDATA is legal"),
            ),
            GenNode::Pi(target, data) => element.append_child(
                document
                    .create_processing_instruction(target, data)
                    .expect("generated PI is legal"),
            ),
        }
    }
    element
}

// ---------------------------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// `parse(write(d))` is structurally identical to `d`.
    #[test]
    fn round_trip_preserves_the_document(generated in arb_document_consistent()) {
        let document = build_document(&generated);
        let serialized = write_string(&document).expect("serializing an in-memory document");
        let reparsed = parse_string(&serialized);
        prop_assert!(
            reparsed.is_ok(),
            "the serializer produced unparseable XML: {}",
            reparsed.as_ref().err().map(|error| error.to_string()).unwrap_or_default()
        );
        let reparsed = reparsed.expect("checked above");
        prop_assert_eq!(sig_document(&reparsed), sig_document(&document), "output was:\n{}", serialized);
    }

    /// Writing is idempotent: `write(parse(write(d))) == write(d)`.
    #[test]
    fn writing_is_idempotent(generated in arb_document_consistent()) {
        let document = build_document(&generated);
        let once = write_string(&document).expect("serialize");
        let twice = write_string(&parse_string(&once).expect("parse")).expect("re-serialize");
        prop_assert_eq!(twice, once);
    }

    /// A document with a declaration keeps it through a round trip.
    #[test]
    fn the_declaration_survives_a_round_trip(generated in arb_document_consistent()) {
        let document = build_document(&generated);
        document.set_xml_declaration(Some(biodivine_lib_xml_dom::xml_spec::XmlDeclaration::utf8()));
        let serialized = write_string(&document).expect("serialize");
        let reparsed = parse_string(&serialized).expect("parse");
        prop_assert_eq!(reparsed.xml_declaration(), document.xml_declaration());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The complementary direction of validation: a document the model considers consistent must
    /// produce **zero** issues. An over-eager rule that flags legitimate documents is the main risk
    /// of the validation pass, and a generator finds that far better than curated fixtures: this
    /// covers every name/declaration/attribute/child-kind combination the generator can build,
    /// including documents with a default namespace, `p0`/`p1` scopes, redundant declarations and
    /// markup-heavy values.
    #[test]
    fn generated_documents_validate_clean(generated in arb_document_consistent()) {
        let document = build_document(&generated);
        if let Err(errors) = document.validate() {
            prop_assert!(false, "a consistent document was reported as invalid:\n{errors}");
        }
        // ... and the round trip must not introduce a problem either.
        let serialized = write_string(&document).expect("serialize");
        let reparsed = parse_string(&serialized).expect("parse");
        if let Err(errors) = reparsed.validate() {
            prop_assert!(
                false,
                "the round trip introduced a validation problem:\n{errors}\nin:\n{serialized}"
            );
        }
        prop_assert!(document.is_valid());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Arbitrary bytes must never make the parser panic.
    #[test]
    fn arbitrary_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let _ = parse_bytes(&bytes);
    }

    /// Arbitrary strings must never make the parser panic.
    #[test]
    fn arbitrary_strings_never_panic(characters in prop::collection::vec(any::<char>(), 0..64)) {
        let text: String = characters.into_iter().collect();
        let _ = parse_string(&text);
    }

    /// Mutating a well-formed document in place must never make the parser panic. This reaches
    /// much deeper into the parser than random bytes do, because most of the input stays valid.
    #[test]
    fn mutated_well_formed_documents_never_panic(
        generated in arb_document_consistent(),
        seed in any::<u64>(),
        mutations in 0u8..8,
    ) {
        let document = build_document(&generated);
        let mut bytes = write_string(&document).expect("serialize").into_bytes();
        let mut state = seed | 1;
        for _ in 0..mutations {
            if bytes.is_empty() {
                break;
            }
            // xorshift64 so the mutation is deterministic given `seed`
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let index = (state as usize) % bytes.len();
            bytes[index] = (state >> 32) as u8;
        }
        let _ = parse_bytes(&bytes);
    }
}
