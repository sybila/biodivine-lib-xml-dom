"""Book chapter "Namespaces" — the Python half."""

import biodivine_lib_xml_dom as xml

document = xml.parse(
    '<root xmlns:ex="http://example.com" xmlns="http://default"><child ex:attr="v"/></root>'
)
root = document.root
child = root.child_elements()[0]

assert str(root.qualified_name) == "root"
assert child.namespace.uri == "http://default"
assert child.attribute(("attr", xml.Namespace("http://example.com", "ex"))) == "v"

# Resolution helpers work in the scope of any node.
assert child.resolve_qualified_name("item").namespace.uri == "http://default"
assert child.resolve_attribute_name("item").namespace is None  # the default never applies
assert child.resolve_attribute_name("xml:lang").namespace.uri == "http://www.w3.org/XML/1998/namespace"

# `xmlns=""` removes the default namespace from the element it is declared on.
document = xml.parse('<root xmlns="http://d"><inner xmlns=""><leaf/></inner></root>')
inner = document.root.child_elements()[0]
assert inner.namespace is None
assert inner.child_elements()[0].namespace is None
assert inner.namespace_declarations == {None: None}

# Editing never synchronises declarations: removing one is silent, and `validate` reports.
document = xml.parse('<ex:root xmlns:ex="http://example.com"><ex:child/></ex:root>')
root = document.root
root.remove_namespace_declaration("ex")
assert len(document.validation_errors()) == 2
assert [problem.kind for problem in document.validation_errors()] == [
    "undeclared_prefix",
    "undeclared_prefix",
]
assert xml.write(document) == "<ex:root><ex:child/></ex:root>"
print(xml.write(document))
