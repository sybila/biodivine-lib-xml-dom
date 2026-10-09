# Rule: Other prefixes must not be bound to the xmlns namespace

No prefix other than "xmlns" must be bound to the namespace name "http://www.w3.org/2000/xmlns/".

## Rationale

> §3 Declaring Namespaces [NSC: Reserved Prefixes and Namespace Names]: "Other prefixes MUST NOT be bound to this namespace name."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns">
  <ex:item/>
</root>
```

## Violating Example

```xml
<!-- Binding another prefix to the xmlns namespace -->
<root xmlns:my="http://www.w3.org/2000/xmlns/">
  <my:item/>
</root>
```
