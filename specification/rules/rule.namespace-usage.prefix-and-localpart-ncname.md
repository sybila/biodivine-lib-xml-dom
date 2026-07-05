# Rule: Prefix and local part must be NCNames

The Prefix and LocalPart components of a qualified name must each be a valid NCName, meaning they must not contain colons.

## Rationale

> §4 Qualified Names: "Prefix ::= NCName" and "LocalPart ::= NCName"

> §7 Conformance of Documents: "All other tokens in the document which are REQUIRED, for XML 1.0 well-formedness, to match the XML production for Name MUST match this specification's production for NCName."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item-id ref="some-ref"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <in:valid:prefix name="bad"/>
</root>
```
