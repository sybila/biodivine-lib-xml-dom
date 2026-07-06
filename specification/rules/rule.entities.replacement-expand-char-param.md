# Rule: Replacement text must expand character and parameter-entity references

The actual replacement text that is included must contain the replacement text of any parameter entities referred to, and must contain the character referred to, in place of any character references in the literal entity value.

## Rationale

> The actual replacement text that is included (or included in literal) as described above MUST contain the replacement text of any parameter entities referred to, and MUST contain the character referred to, in place of any character references in the literal entity value. (§4.5)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % pub "&&#xC9;ditions">
  <!ENTITY book "La Peste %pub;">
]>
<root>&book;</root>
```

> In the valid case, `&book;` expands to `La Peste &Éditions` — the parameter entity `%pub;` is expanded to its replacement text `&&#xC9;ditions`, and the character reference `&#xC9;` is expanded to the character É.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % pub "&&#xC9;ditions">
  <!ENTITY book "La Peste %pub;">
]>
<root>&book;</root>
```

> A processor that fails to expand `%pub;` or `&#xC9;` in the replacement text would produce incorrect output, violating this rule.
