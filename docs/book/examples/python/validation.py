"""Book chapter "Validation" — the Python half."""

import biodivine_lib_xml_dom as xml

XML_NAMESPACE = xml.Namespace("http://www.w3.org/XML/1998/namespace", "xml")

# A document assembled through the API is not validated as you edit: edits are silent.
document = xml.Document()
missing = xml.Namespace("http://example.com", "ex")
root = document.create_element(("root", missing))
document.set_root(root)
root.set_attribute(("space", XML_NAMESPACE), "preserve-everything")
root.set_attribute(("id", XML_NAMESPACE), "not a name")

# Every problem is reported in one call, with the node and the rule it comes from.
problems = document.validation_errors()
assert len(problems) == 3
for problem in problems:
    print(f"{problem.node}: {problem.message} [{problem.rule}]")
assert not document.is_valid()
assert sorted(problems.kinds()) == [
    "invalid_xml_space",
    "undeclared_prefix",
    "xml_id_is_not_a_name",
]

# `validate()` is the raising form, carrying the same list.
try:
    document.validate()
except xml.XmlValidationError as error:
    summary, payload = error.args
    assert "3 validation problems" in summary
    assert len(payload) == 3

# Fixing the problems makes the document valid.
root.declare_namespace(missing)
root.set_attribute(("space", XML_NAMESPACE), "preserve")
root.remove_attribute(("id", XML_NAMESPACE))
assert document.is_valid()
assert xml.parse('<a xmlns:ex="http://e"><ex:b/></a>').is_valid()
print("ok")
