# Rule: Element and attribute names contain zero or one colon

In a namespace-well-formed document, all element and attribute names contain either zero or one colon.

## Rationale

> It follows that in a namespace-well-formed document: All element and attribute names contain either zero or one colon. (§7)

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <item/>
  <ex:child/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <a:b:c/>
</root>
```

> The name `a:b:c` contains two colons, which is not a valid QName.
