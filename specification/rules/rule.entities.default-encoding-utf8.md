# Rule: Entity without BOM or encoding declaration defaults to UTF-8

It is a fatal error for an entity which begins with neither a Byte Order Mark nor an encoding declaration to use an encoding other than UTF-8.

## Rationale

> In the absence of information provided by an external transport protocol (e.g. HTTP or MIME), it is a fatal error for an entity which begins with neither a Byte Order Mark nor an encoding declaration to use an encoding other than UTF-8. (§4.3.3)

## Valid Example

```xml
<root>Plain ASCII/UTF-8 content</root>
```

## Violating Example

```xml
<root>ISO-8859-1 content without BOM or encoding declaration</root>
```
