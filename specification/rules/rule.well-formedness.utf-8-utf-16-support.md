# Rule: Processors Must Accept UTF-8 And UTF-16

All XML processors must accept the UTF-8 and UTF-16 encodings of Unicode.

## Rationale

> §2.2: All XML processors MUST accept the UTF-8 and UTF-16 encodings of Unicode.

## Valid Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>content</root>
```

## Violating Example

```xml
<!-- A processor refusing a valid UTF-8 document violates this rule -->
<?xml version="1.0" encoding="UTF-8"?>
<root>content</root>
```