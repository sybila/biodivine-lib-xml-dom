# Rule: UTF-8 entities MAY begin with BOM

Entities encoded in UTF-8 MAY begin with the Byte Order Mark (U+FEFF). Unlike UTF-16, the BOM is optional for UTF-8.

## Rationale

> Entities encoded in UTF-16 MUST and entities encoded in UTF-8 MAY begin with the Byte Order Mark. (§4.3.3)

## Valid Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!-- UTF-8 with BOM (EF BB BF bytes at start) -->
<root>content</root>
```

## Valid Example (no BOM, also valid)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!-- UTF-8 without BOM is also valid -->
<root>content</root>
```
