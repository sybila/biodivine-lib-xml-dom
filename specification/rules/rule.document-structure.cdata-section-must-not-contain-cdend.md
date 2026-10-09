# Rule: CDATA section content must not contain ]]>

The content of a CDATA section must not contain the string `]]>`, because that sequence terminates the section.

## Rationale

> CData ::= (Char* - (Char* ']]>' Char*)) (§2.7, production [20])

## Valid Example

```xml
<?xml version="1.0"?>
<root><![CDATA[This is safe content]]></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root><![CDATA[This contains ]]> and more]]></root>
```
