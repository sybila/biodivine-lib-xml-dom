"""Book chapter "Getting started" — the Python half.

The chapter includes this file verbatim, and `tests-python/test_book_examples.py` runs it, so the
book cannot show code that no longer works.
"""

import biodivine_lib_xml_dom as xml

document = xml.Document()
assert document.root is None

ex = xml.Namespace("http://example.com", "ex")
root = document.create_element(("root", ex))
root.declare_namespace(ex)
document.set_root(root)

child = document.create_element(("child", ex))
child.set_attribute("id", "first")
child.append_child(document.create_text("Hello, World!"))
root.append_child(child)

assert xml.write(document) == (
    '<ex:root xmlns:ex="http://example.com"><ex:child id="first">Hello, World!</ex:child></ex:root>'
)
assert document.is_valid()

# Names are *expanded* names: the prefix belongs to the namespace, not to the local name.
child = root.child_elements()[0]
assert child.local_name == "child"
assert child.namespace.uri == "http://example.com"
assert child.qualified_name == xml.QualifiedName("child", ex)
print(xml.write(document))
