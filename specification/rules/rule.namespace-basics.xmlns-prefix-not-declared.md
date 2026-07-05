# Rule: "xmlns" prefix must not be declared

The prefix "xmlns" must not be declared with a namespace declaration attribute.

## Rationale

> §3 Declaring Namespaces [NSC: Reserved Prefixes and Namespace Names]: "The prefix xmlns is used only to declare namespace bindings and is by definition bound to the namespace name http://www.w3.org/2000/xmlns/. It MUST NOT be declared."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns">
  <ex:item/>
</root>
```

## Violating Example

```xml
<!-- Attempting to declare the "xmlns" prefix -->
<root xmlns:xmlns="http://example.org/ns">
  <xmlns:item/>
</root>
```
