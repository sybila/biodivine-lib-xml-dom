# Rule: CDATA section must end with ]]>

A CDATA section must end with the exact string `]]>`.

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
<root><![CDATA[Some text</root>
```
