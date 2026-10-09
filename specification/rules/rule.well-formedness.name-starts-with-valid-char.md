# Rule: Name Must Start With NameStartChar

The first character of an XML Name must be a NameStartChar (letter, colon, underscore, or certain Unicode ranges).

## Rationale

> §2.3: The first character of a Name MUST be a NameStartChar. [4] NameStartChar ::= ":" | [A-Z] | "_" | [a-z] | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]

## Valid Example

```xml
<root>content</root>
```

## Violating Example

```xml
<1root>content</1root>
```