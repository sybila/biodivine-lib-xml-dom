# Rule: Encoding names SHOULD follow IANA conventions

XML processors SHOULD use IANA-registered character encoding names. For non-registered names, the x- prefix convention SHOULD be used. Specific encoding names are recommended for Unicode, ISO-8859, and JIS encodings.

## Rationale

> XML processors SHOULD use IANA-registered character encoding names. For other encodings, the x- prefix convention SHOULD be used. (§4.3.3)

## Note

> This is a SHOULD-level recommendation about encoding name conventions. Common mappings include: Unicode -> UTF-8/UTF-16, ISO-8859-1 -> ISO-8859-1, JIS X-0201 -> Shift_JIS, etc.
