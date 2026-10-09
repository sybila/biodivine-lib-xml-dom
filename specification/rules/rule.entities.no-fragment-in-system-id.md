# Rule: System identifiers must not contain fragment identifiers

It is an error for a fragment identifier (beginning with a # character) to be part of a system identifier.

## Rationale

> It is an error for a fragment identifier (beginning with a # character) to be part of a system identifier. (§4.2.2)

## Valid Example

```xml
<!ENTITY data SYSTEM "http://example.com/data.xml">
```

## Violating Example

```xml
<!ENTITY data SYSTEM "http://example.com/data.xml#section1">
```
