# Rule: Prefixes beginning with "xml" (any case) are reserved

All prefixes beginning with the three-letter sequence x, m, l in any case combination are reserved. Processors MUST NOT treat them as fatal errors, and users SHOULD NOT use them.

## Rationale

> All other prefixes beginning with the three-letter sequence x, m, l, in any case combination, are reserved. (§3, [NSC: Reserved Prefixes and Namespace Names])

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:xhtml="http://www.w3.org/1999/xhtml">
  <xhtml:div/>
</root>
```

> Note: `xhtml` begins with "xml" and is reserved. Using it is not a fatal error but is discouraged for interoperability.

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns:XMLNS="http://example.org">
  <XMLNS:item/>
</root>
```

> The prefix `XMLNS` begins with "xml" (case-insensitive) and is reserved. While not a fatal error, it should not be used.
