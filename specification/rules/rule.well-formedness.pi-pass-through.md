# Rule: Processing instructions must be passed through to the application

Processing instructions are not part of the document's character data, but XML processors must always pass them through to the application.

## Rationale

> PIs are not part of the document's character data, but MUST be passed through to the application. (§2.6)

## Valid Example

```xml
<?xml version="1.0"?>
<?xml-stylesheet href="style.css"?>
<root>content</root>
```

> The processing instruction `<?xml-stylesheet ...?>` must be passed through to the application.

## Violating Example

```xml
<?xml version="1.0"?>
<?xml-stylesheet href="style.css"?>
<root>content</root>
<!-- A processor that silently discards the processing
     instruction instead of passing it through would
     violate this requirement. -->
```
