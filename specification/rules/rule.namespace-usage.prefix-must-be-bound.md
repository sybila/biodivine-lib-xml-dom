# Rule: Prefix must be bound to a namespace URI

Every prefix used in a qualified name must be associated with a namespace URI reference through a namespace declaration.

## Rationale

> §4 Qualified Names: "The Prefix provides the namespace prefix part of the qualified name, and MUST be associated with a namespace URI reference in a namespace declaration."

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
  <ex:item>value</ex:item>
</root>
```
