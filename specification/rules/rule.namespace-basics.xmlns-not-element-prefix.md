# Rule: Element names must not have the prefix "xmlns"

Element names must not use "xmlns" as their prefix.

## Rationale

> §3 Declaring Namespaces [NSC: Reserved Prefixes and Namespace Names]: "Element names MUST NOT have the prefix xmlns."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns">
  <ex:item/>
</root>
```

## Violating Example

```xml
<!-- Element with "xmlns" prefix -->
<root xmlns:ex="http://example.org/ns">
  <xmlns:item/>
</root>
```
