# Rule: First entity declaration encountered is binding

If the same entity is declared more than once, the first declaration encountered is binding; subsequent declarations are ignored.

## Rationale

> If the same entity is declared more than once, the first declaration encountered is binding; at user option, an XML processor MAY issue a warning if entities are declared multiple times. (§4.2)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY greeting "Hello">
  <!ENTITY greeting "Goodbye">
]>
<root>&greeting;</root>
```

## Violating Example

```xml
<!-- The second declaration is silently ignored;
     &greeting; resolves to "Hello", not "Goodbye".
     Relying on the second declaration violates this rule. -->
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY greeting "First value">
  <!ENTITY greeting "Second value">
]>
<!-- Application expects "Second value" but gets "First value" -->
<root>&greeting;</root>
```
