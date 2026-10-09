# Rule: Replacement text must expand character and parameter-entity references

The actual replacement text that is included must contain the replacement text of any parameter entities referred to, and must contain the character referred to, in place of any character references in the literal entity value.

## Rationale

> The actual replacement text that is included (or included in literal) as described above MUST contain the replacement text of any parameter entities referred to, and MUST contain the character referred to, in place of any character references in the literal entity value. (§4.5)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % pub "&#xC9;ditions">
  <!ENTITY rights "All rights reserved">
  <!ENTITY book "La Peste, %pub;. &rights;">
]>
<root>&book;</root>
```

> In the valid case, the replacement text for `book` contains `La Peste, Éditions. &rights;` — the parameter entity `%pub;` is expanded, the character reference `&#xC9;` is expanded to `É`, and the general entity reference `&rights;` remains unexpanded in the replacement text.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % pub "&#xC9;ditions">
  <!ENTITY rights "All rights reserved">
  <!ENTITY book "La Peste, %pub;. &rights;">
]>
<root>&book;</root>
```

> A processor that fails to expand `%pub;` or `&#xC9;` while constructing the replacement text would violate this rule.
