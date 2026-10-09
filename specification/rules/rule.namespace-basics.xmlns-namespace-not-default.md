# Rule: xmlns namespace must not be declared as default namespace

The namespace name "http://www.w3.org/2000/xmlns/" must not be declared as the default namespace.

## Rationale

> §3 Declaring Namespaces [NSC: Reserved Prefixes and Namespace Names]: "It MUST NOT be declared as the default namespace."

## Valid Example

```xml
<root xmlns="http://example.org/ns">
  <item/>
</root>
```

## Violating Example

```xml
<!-- Declaring the xmlns namespace as the default -->
<root xmlns="http://www.w3.org/2000/xmlns/">
  <item/>
</root>
```
