# Rule: Name Subsequent Characters Must Be NameChar

Every character after the first in an XML Name must be a NameChar (NameStartChar, hyphen, full stop, digit, #xB7, or combining/mark ranges).

## Rationale

> §2.3: Any other characters MUST be NameChars. [4a] NameChar ::= NameStartChar | "-" | "." | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]

## Valid Example

```xml
<my-element_2.0>content</my-element_2.0>
```

## Violating Example

```xml
<my element>content</my element>
```