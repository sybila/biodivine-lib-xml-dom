# Rule: Processors must report namespace well-formedness violations

A processor conforming to the XML Namespaces specification must report violations of namespace well-formedness.

## Rationale

> §8 Conformance of Processors: "To conform to this specification, a processor MUST report violations of namespace well-formedness, with the exception that it is not REQUIRED to check that namespace names are URI references."

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
<root>
  <undeclared:prefix>processor must report this error</undeclared:prefix>
</root>
```
