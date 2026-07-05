# Rule: General entity references in EntityValue must be bypassed

When a general entity reference appears in the EntityValue in an entity declaration, it must be bypassed and left as is.

## Rationale

> When a general entity reference appears in the EntityValue in an entity declaration, it MUST be bypassed and left as is. (§4.4.7)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY other "replacement">
  <!ENTITY wrapper "text &other; more">
]>
<root>&wrapper;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY other "replacement">
  <!ENTITY wrapper "text &other; more">
]>
<root>&wrapper;</root>
```
