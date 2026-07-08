# Rule: Namespace declaration attribute names must match xmlns or xmlns:NCName

A namespace declaration attribute name must be either "xmlns" or "xmlns:" followed by an NCName.

## Rationale

> §3 Declaring Namespaces: "A namespace (or more precisely, a namespace binding) is declared using a family of reserved attributes. Such an attribute's name must either be xmlns or begin xmlns:."

## Valid Example

```xml
<root xmlns="http://example.org/default"
      xmlns:ex="http://example.org/prefixed">
  <item/>
</root>
```

## Violating Example

```xml
<!-- "namespace" is not a valid namespace declaration attribute -->
<root namespace="http://example.org/ns">
  <item/>
</root>
```
