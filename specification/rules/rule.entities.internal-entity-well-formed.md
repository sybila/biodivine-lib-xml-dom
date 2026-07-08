# Rule: Internal general parsed entity replacement text must match content production

An internal general parsed entity is well-formed if its replacement text matches the production labeled content.

## Rationale

> An internal general parsed entity is well-formed if its replacement text matches the production labeled content. (§4.3.2)

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
  <!ENTITY broken "<unclosed">
]>
<root>&broken;</root>
```
