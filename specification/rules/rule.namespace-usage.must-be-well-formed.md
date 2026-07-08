# Rule: Document must be XML 1.0 well-formed

A document conforming to the XML Namespaces specification must be well-formed according to the XML 1.0 specification.

## Rationale

> §7 Conformance of Documents: "This specification applies to XML 1.0 documents. To conform to this specification, a document MUST be well-formed according to the XML 1.0 specification."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item>value</ex:item>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item>value
</root>
```
