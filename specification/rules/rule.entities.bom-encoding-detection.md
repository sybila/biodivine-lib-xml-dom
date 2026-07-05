# Rule: XML processors must use BOM to differentiate UTF-8 from UTF-16

XML processors must be able to use the Byte Order Mark character to differentiate between UTF-8 and UTF-16 encoded documents.

## Rationale

> XML processors MUST be able to use this character to differentiate between UTF-8 and UTF-16 encoded documents. (§4.3.3)

## Valid Example

```xml
<root>Document with BOM for encoding detection</root>
```

## Violating Example

```xml
<root>UTF-16 document without BOM - processor cannot distinguish from UTF-8</root>
```
