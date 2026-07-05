# Rule: xml namespace must not be declared as default namespace

The namespace name "http://www.w3.org/XML/1998/namespace" must not be declared as the default namespace.

## Rationale

> §3 Declaring Namespaces [NSC: Reserved Prefixes and Namespace Names]: "It MUST NOT be declared as the default namespace."

## Valid Example

```xml
<root xmlns="http://example.org/ns"
      xmlns:xml="http://www.w3.org/XML/1998/namespace">
  <item/>
</root>
```

## Violating Example

```xml
<!-- Declaring the xml namespace as the default -->
<root xmlns="http://www.w3.org/XML/1998/namespace">
  <item/>
</root>
```
