# Rule: Namespace names must be URI references

An XML namespace name must be a URI reference as defined by RFC 3986.

## Rationale

> §2.1 Basic Concepts: "An XML namespace is identified by a URI reference [RFC3986]; element and attribute names may be placed in an XML namespace using the mechanisms described in this specification."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns">
  <ex:item/>
</root>
```

## Violating Example

```xml
<!-- Not a valid URI reference -->
<root xmlns:ex="not a uri">
  <ex:item/>
</root>
```
