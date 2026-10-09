//! Book chapter "Parsing and serializing": round trips, options, and what parsing rejects.
//!
//! Run with `cargo run --example book_parsing`.

use biodivine_lib_xml_dom::{
    DeclarationStyle, EmptyElementStyle, WriteOptions, parse_string, write_string,
    write_string_with,
};

fn main() {
    // A round trip is faithful: prefixes, declarations, escaping and the XML declaration survive.
    let source = r#"<?xml version="1.0" encoding="UTF-8"?><ex:a xmlns:ex="http://e"><ex:b x="1 &amp; 2">t</ex:b><!--c--><![CDATA[<raw>]]><?pi data?></ex:a>"#;
    let document = parse_string(source).unwrap();
    assert_eq!(write_string(&document).unwrap(), source);
    assert!(document.xml_declaration().is_some());

    // Output options.
    let options = WriteOptions {
        declaration: DeclarationStyle::Never,
        empty_elements: EmptyElementStyle::ExplicitEndTag,
    };
    assert_eq!(
        write_string_with(&parse_string("<a><b/></a>").unwrap(), &options).unwrap(),
        "<a><b></b></a>"
    );

    // The parser rejects malformed input with a typed error instead of panicking.
    for bad in [
        "<a>",                   // unclosed element
        "<a/><b/>",              // more than one root
        "text<a/>",              // content outside the root
        r#"<a b="x<y"/>"#,       // `<` in an attribute value
        r#"<a xmlns:p=""/>"#,    // an empty prefix declaration
        "<p:a/>",                // an undeclared prefix
        r#"<a a="1" a="2"/>"#,   // duplicate attribute
        r#"<a>&undefined;</a>"#, // an entity this library cannot resolve
    ] {
        let error = parse_string(bad).unwrap_err();
        println!("{bad:28} -> {error}");
    }

    // Predefined entities and character references are expanded.
    let text = parse_string("<a>AT&amp;T &#65;</a>").unwrap();
    assert_eq!(
        text.root().unwrap().children()[0].text().unwrap().as_str(),
        "AT&T A"
    );
    println!("{}", write_string(&document).unwrap());
}
