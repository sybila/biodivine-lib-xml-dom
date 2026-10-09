# Rule: Single Root Element

An XML document must contain exactly one root element, no part of which appears in the content of any other element.

## Rationale

> §2.1: There is exactly one element, called the root, or document element, no part of which appears in the content of any other element.

## Valid Example

```xml
<root><child>content</child></root>
```

## Violating Example

```xml
<a>content</a><b>content</b>
```