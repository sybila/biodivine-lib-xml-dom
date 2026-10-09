# Rule: Document Characters Must Be Within Legal Range

All characters in an XML document must be within the legal character range: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF].

## Rationale

> §2.2: XML processors MUST accept any character in the range specified for Char. [2] Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]

## Valid Example

```xml
<root>hello</root>
```

## Violating Example

```xml
<root>text with surrogate &#xD800;</root>
```