# Rule: Right Angle Bracket Must Be Escaped In ]] > Sequence

When the string "]]>" appears in content and is not marking the end of a CDATA section, the right angle bracket must be escaped using "&gt;" or a character reference.

## Rationale

> §2.4: The right angle bracket (>) MAY, for compatibility, be escaped using either "&gt;" or a character reference when it appears in the string "]]>" in content, when that string is not marking the end of a CDATA section.

## Valid Example

```xml
<root>some ]]&gt; text</root>
```

## Violating Example

```xml
<root>some ]] > text</root>
```