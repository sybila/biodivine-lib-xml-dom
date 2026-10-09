# Rule: [WFC] Parsed entities must not contain recursive references

A parsed entity must not contain a recursive reference to itself, either directly or indirectly.

## Rationale

> [WFC: No Recursion] A parsed entity MUST NOT contain a recursive reference to itself, either directly or indirectly. (§4.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY greeting "Hello, World!">
]>
<root>&greeting;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY self "&self;">
]>
<root>&self;</root>
```
