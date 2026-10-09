"""Book chapter "The Python layer" — what is Pythonic rather than mirrored."""

import biodivine_lib_xml_dom as xml

document = xml.parse('<root><child class="a"/><child class="b"/></root>')
root = document.root

# Iteration, indexing and length over children.
assert [child.local_name for child in root] == ["child", "child"]
assert root[1].local_name == "child"
assert len(root) == 2

# `attributes` is a dict keyed by `QualifiedName`; `get`/`set` accept a plain local name.
child = root[0]
assert child.attributes == {xml.QualifiedName("class"): "a"}
assert child.get("class") == "a"
child.set_attribute("class", "changed")
assert child.get(("class", None)) == "changed"

# Names are accepted as a string, as `(local, namespace)` or as a `QualifiedName`.
ex = xml.Namespace("http://example.com", "ex")
assert str(document.create_element("plain").qualified_name) == "plain"
assert str(document.create_element(("ns", ex)).qualified_name) == "ex:ns"
assert str(document.create_element(xml.QualifiedName("ns", ex)).qualified_name) == "ex:ns"

# Handles are usable as dictionary keys, and `document`/`id`/`parent` are properties.
index = {root: "root", child: "child"}
assert index[document.root] == "root"
assert child.parent == root
assert child.document == document
assert isinstance(child.id, xml.NodeId)

# The same operations in Rust are `child_elements()`, `attribute_local()` and explicit handles;
# see the Rust half of this chapter.
print(f"{len(root)} children")
