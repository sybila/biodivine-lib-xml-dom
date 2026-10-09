# Rule: Character references must refer to legal characters

Characters referred to using character references must match the production for Char.

## Rationale

> [WFC: Legal Character] Characters referred to using character references MUST match the production for Char. (§4.1)

## Valid Example

```xml
<?xml version="1.0"?>
<root>&#65;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>&#0;</root>
```
