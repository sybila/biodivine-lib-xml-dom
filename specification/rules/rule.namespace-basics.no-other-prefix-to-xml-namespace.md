# Rule: Other prefixes must not be bound to the xml namespace

No prefix other than "xml" must be bound to the namespace name "http://www.w3.org/XML/1998/namespace".

## Rationale

> §3 Declaring Namespaces [NSC: Reserved Prefixes and Namespace Names]: "Other prefixes MUST NOT be bound to this namespace name."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns">
  <ex:item xml:lang="en"/>
</root>
```

## Violating Example

```xml
<!-- Binding a non-xml prefix to the xml namespace -->
<root xmlns:my="http://www.w3.org/XML/1998/namespace">
  <my:item/>
</root>
```
