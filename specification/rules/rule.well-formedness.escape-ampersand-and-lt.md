# Rule: Ampersand And Less-Than Must Be Escaped In Content

The ampersand (&) and left angle bracket (<) must not appear in their literal form in content, except when used as markup delimiters or within a comment, processing instruction, or CDATA section.

## Rationale

> §2.4: The ampersand character (&) and the left angle bracket (<) MUST NOT appear in their literal form, except when used as markup delimiters, or within a comment, a processing instruction, or a CDATA section.

## Valid Example

```xml
<root>This is &lt;tagged&gt; and &amp; more</root>
```

## Violating Example

```xml
<root>This is <tagged> and & more</root>
```