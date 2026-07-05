# Rule: Entity encoding must match encoding declaration

It is a fatal error for an entity including an encoding declaration to be presented to the XML processor in an encoding other than that named in the declaration.

## Rationale

> In the absence of information provided by an external transport protocol (e.g. HTTP or MIME), it is a fatal error for an entity including an encoding declaration to be presented to the XML processor in an encoding other than that named in the declaration. (§4.3.3)

## Valid Example

```xml
<?xml encoding='UTF-8'?>
<root>UTF-8 encoded content</root>
```

## Violating Example

```xml
<?xml encoding='UTF-8'?>
<root>Actually ISO-8859-1 encoded but declared as UTF-8</root>
```
