# Rule: CDATA section must begin with <![CDATA[

A CDATA section must begin with the exact string `<![CDATA[`.

## Rationale

> CDATA sections begin with the string "<![CDATA[" and end with the string "]]>". (§2.7)

## Valid Example

```xml
<?xml version="1.0"?>
<root><![CDATA[Some text]]></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>[CDATA[Some text]]></root>
```
