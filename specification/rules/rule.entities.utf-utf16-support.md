# Rule: XML processors must support UTF-8 and UTF-16

All XML processors must be able to read entities in both the UTF-8 and UTF-16 encodings.

## Rationale

> All XML processors MUST be able to read entities in both the UTF-8 and UTF-16 encodings. (§4.3.3)

## Valid Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0" encoding="unsupported-encoding"?>
<root>content</root>
```
