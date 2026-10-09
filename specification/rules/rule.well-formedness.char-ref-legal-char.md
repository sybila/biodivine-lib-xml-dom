# Rule: Character references must refer to legal Char values

All characters referred to using character references must match the production for Char: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF].

## Rationale

> §2.2: Char ::= #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]

> [WFC: Legal Character] Characters referred to using character references MUST match the production for Char. (§4.1)

## Valid Example

```xml
<?xml version="1.0"?>
<root>&#x41;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>&#xD800;</root>
```
