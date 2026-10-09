# Rule: Namespace-validating processors must report namespace validity violations

A namespace-validating processor must, in addition to reporting namespace well-formedness violations, also report violations of namespace validity.

## Rationale

> §8 Conformance of Processors: "A validating XML processor that conforms to this specification is namespace-validating if in addition it reports violations of namespace validity."

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item*)>
  <!ELEMENT item (#PCDATA)>
  <!ATTLIST item xmlns CDATA #FIXED "http://example.org">
]>
<root xmlns="http://example.org">
  <item>value</item>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item*)>
  <!ELEMENT item (#PCDATA)>
  <!ATTLIST item xmlns CDATA #FIXED "http://example.org">
]>
<root xmlns="http://wrong.org">
  <item>value</item>
</root>
```
