# Rule: Entity without encoding declaration must be legal UTF-8 or UTF-16

Unless an encoding is determined by a higher-level protocol, it is a fatal error if an XML entity contains no encoding declaration and its content is not legal UTF-8 or UTF-16.

## Rationale

> Unless an encoding is determined by a higher-level protocol, it is also a fatal error if an XML entity contains no encoding declaration and its content is not legal UTF-8 or UTF-16. (§4.3.3)

## Valid Example

```xml
<root>Legal ASCII subset of UTF-8</root>
```

## Violating Example

```xml
<root>Contains bytes illegal in both UTF-8 and UTF-16</root>
```
