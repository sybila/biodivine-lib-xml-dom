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

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % pub "&&#xC9;ditions">
  <!ENTITY book "La Peste %pub;">
]>
<root>&book;</root>
```
