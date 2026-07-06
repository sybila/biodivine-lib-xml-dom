# Rule: UTF-16 entities must begin with BOM

Entities encoded in UTF-16 must begin with the Byte Order Mark (U+FEFF, encoded as bytes FE FF for UTF-16BE or FF FE for UTF-16LE).

## Rationale

> Entities encoded in UTF-16 MUST begin with the Byte Order Mark described by Annex H of ISO/IEC 10646:2000, section 16.8 of Unicode (the ZERO WIDTH NO-BREAK SPACE character, #xFEFF). (§4.3.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!-- UTF-16 encoded with BOM (FE FF bytes at start of file) -->
<!-- The first two bytes of the file are the BOM, followed by the text declaration -->
```

## Violating Example

```xml
<?xml version="1.0"?>
<!-- UTF-16 encoded without BOM (no FE FF bytes at start of file) -->
<!-- This is a fatal error per the specification -->
```
