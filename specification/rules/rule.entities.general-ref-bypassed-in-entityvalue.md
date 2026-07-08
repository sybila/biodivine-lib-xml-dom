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

> In the valid case, `&wrapper;` expands to `text &other; more` — the `&other;` inside the EntityValue is left as a literal entity reference and is NOT expanded when `&wrapper;` is later processed.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY other "replacement">
  <!ENTITY wrapper "text &other; more">
]>
<root>&wrapper;</root>
```

> A processor that incorrectly expands `&other;` inside the EntityValue, producing `text replacement more` instead of `text &other; more`, would violate this rule.
