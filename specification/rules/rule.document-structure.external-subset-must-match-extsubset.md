# Rule: External subset must match extSubset production

The external subset, if present, must match the production for extSubset (an optional TextDecl followed by extSubsetDecl).

## Rationale

> [WFC: External Subset] The external subset, if any, MUST match the production for extSubset. (§2.8)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "external.dtd">
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "external.dtd">
<root>content</root>
```
