# Rule: CDATA sections must not nest

A CDATA section must not contain another CDATA section within it.

## Rationale

> CDATA sections cannot nest. (§2.7)

## Valid Example

```xml
<?xml version="1.0"?>
<root><![CDATA[First section]]></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root><![CDATA[Outer <![CDATA[Inner]]> Outer]]></root>
```
