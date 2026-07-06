# Rule: DTD-based validation is not namespace-aware

A DTD constrains the elements and attributes that may appear in a document by their uninterpreted names, not by (namespace name, local name) pairs. To validate a document that uses namespaces against a DTD, the same prefixes must be used in the DTD as in the instance.

## Rationale

> A DTD constrains the elements and attributes that may appear in a document by their uninterpreted names, not by (namespace name, local name) pairs. To validate a document that uses namespaces against a DTD, the same prefixes must be used in the DTD as in the instance. (§5)

## Note

> This is an important behavioral constraint: namespace-aware documents validated against a DTD require prefix consistency between the DTD and the instance document.
