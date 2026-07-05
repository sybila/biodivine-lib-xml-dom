# Rule: UTF-16 entities must begin with BOM

Entities encoded in UTF-16 must begin with the Byte Order Mark (U+FEFF).

## Rationale

> Entities encoded in UTF-16 MUST begin with the Byte Order Mark described by Annex H of ISO/IEC 10646:2000, section 16.8 of Unicode (the ZERO WIDTH NO-BREAK SPACE character, #xFEFF). (§4.3.3)

## Valid Example

```xml
<root>UTF-16 encoded with BOM prefix</root>
```

## Violating Example

```xml
<root>UTF-16 encoded without BOM prefix</root>
```
