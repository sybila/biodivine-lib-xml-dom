# Rule: URI reference comparison must be literal and case-sensitive

URI references identifying namespaces must be compared as identical strings with case-sensitivity and no %-escaping processing.

## Rationale

> §2.3 Comparing URI References: "The two URIs are treated as strings, and they are identical if and only if the strings are identical, that is, if they are the same sequence of characters. The comparison is case-sensitive, and no %-escaping is done or undone."

## Valid Example

```xml
<!-- These are treated as two distinct namespaces -->
<root xmlns:a="http://example.org/ns"
      xmlns:b="http://example.org/NS">
  <a:item/>
  <b:item/>
</root>
```

## Violating Example

```xml
<!-- A processor that treats "http://example.org/ns" and
     "http://example.org/NS" as the same namespace would
     violate this rule. The correct behavior is to treat
     them as two distinct namespace names. -->
<root xmlns:a="http://example.org/ns"
      xmlns:b="http://example.org/NS">
  <!-- a:item and b:item are in DIFFERENT namespaces -->
</root>
```
