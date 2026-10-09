# Rule: Namespace declaration value must be a URI reference or empty string

The normalized value of a namespace declaration attribute must be either a URI reference or an empty string.

## Rationale

> §3 Declaring Namespaces: "The attribute's normalized value MUST be either a URI reference — the namespace name identifying the namespace — or an empty string."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns"
      xmlns="">
  <item/>
</root>
```

## Violating Example

```xml
<!-- Value contains spaces and is not a valid URI reference -->
<root xmlns:ex="not a valid uri ">
  <item/>
</root>
```
