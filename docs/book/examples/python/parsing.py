"""Book chapter "Parsing and serializing" — the Python half."""

import biodivine_lib_xml_dom as xml

# A round trip is faithful: prefixes, declarations, escaping and the declaration survive.
source = (
    '<?xml version="1.0" encoding="UTF-8"?><ex:a xmlns:ex="http://e">'
    '<ex:b x="1 &amp; 2">t</ex:b><!--c--><![CDATA[<raw>]]><?pi data?></ex:a>'
)
document = xml.parse(source)
assert xml.write(document) == source
assert document.xml_declaration is not None

# Output options.
options = xml.WriteOptions(
    declaration=xml.DeclarationStyle.Never,
    empty_elements=xml.EmptyElementStyle.ExplicitEndTag,
)
assert xml.write(xml.parse("<a><b/></a>"), options) == "<a><b></b></a>"

# Malformed input raises a typed error instead of panicking.
for bad in [
    "<a>",                      # unclosed element
    "<a/><b/>",                 # more than one root
    "text<a/>",                 # content outside the root
    '<a b="x<y"/>',             # `<` in an attribute value
    '<a xmlns:p=""/>',          # an empty prefix declaration
    "<p:a/>",                   # an undeclared prefix
    '<a a="1" a="2"/>',         # duplicate attribute
    "<a>&undefined;</a>",       # an entity this library cannot resolve
]:
    try:
        xml.parse(bad)
    except xml.XmlSyntaxError as error:
        print(f"{bad:26} -> {error}")
    except xml.XmlNamespaceError as error:
        print(f"{bad:26} -> {error}")

# Predefined entities and character references are expanded.
text = xml.parse("<a>AT&amp;T &#65;</a>")
assert text.root[0].text() == "AT&T A"
print(xml.write(document))
