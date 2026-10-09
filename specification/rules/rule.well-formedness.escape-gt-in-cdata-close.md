# Rule: Right Angle Bracket MAY Be Escaped In ]] > Sequence

When the string "]]>" appears in content and is not marking the end of a CDATA section, the right angle bracket (>) MAY be escaped using "&gt;" or a character reference for compatibility. The unescaped form `]] >` is well-formed.

## Rationale

> §2.4: The right angle bracket (>) MAY, for compatibility, be escaped using either "&gt;" or a character reference when it appears in the string "]]>" in content, when that string is not marking the end of a CDATA section.

## Valid Example (escaped form, for compatibility)

```xml
<root>some ]]&gt; text</root>
```

## Valid Example (unescaped form, also well-formed)

```xml
<root>some ]] > text</root>
```
