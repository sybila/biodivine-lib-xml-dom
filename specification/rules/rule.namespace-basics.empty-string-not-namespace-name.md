# Rule: Empty string must not be used as a namespace name

The empty string must not be used as a namespace name in a namespace declaration, even though it is a legal URI reference.

## Rationale

> §2.2 Use of URIs as Namespace Names: "The empty string, though it is a legal URI reference, cannot be used as a namespace name."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns">
  <ex:item/>
</root>
```

## Violating Example

```xml
<root xmlns:ex="">
  <ex:item/>
</root>
```
