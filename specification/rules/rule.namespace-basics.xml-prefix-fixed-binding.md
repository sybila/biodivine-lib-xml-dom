# Rule: "xml" prefix must be bound to its fixed namespace

The prefix "xml" must be bound to the namespace name "http://www.w3.org/XML/1998/namespace" and must not be bound to any other namespace name.

## Rationale

> §3 Declaring Namespaces [NSC: Reserved Prefixes and Namespace Names]: "The prefix xml is by definition bound to the namespace name http://www.w3.org/XML/1998/namespace. It MAY, but need not, be declared, and MUST NOT be bound to any other namespace name."

## Valid Example

```xml
<!-- "xml" prefix used without explicit declaration -->
<root xml:lang="en">
  <item/>
</root>
```

## Violating Example

```xml
<!-- Attempting to rebind "xml" to another namespace -->
<root xmlns:xml="http://example.org/custom">
  <xml:item/>
</root>
```
