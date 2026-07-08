# Rule: General entity references must be left unexpanded in replacement text

In constructing replacement text, general-entity references must be left as-is, unexpanded.

## Rationale

> However, general-entity references MUST be left as-is, unexpanded. (§4.5)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY rights "All rights reserved">
  <!ENTITY book "Title. &rights;">
]>
<root>&book;</root>
```

> In the valid case, `&book;` expands to `Title. &rights;` — the `&rights;` reference is left as-is in the replacement text of `book`.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY rights "All rights reserved">
  <!ENTITY book "Title. &rights;">
]>
<root>&book;</root>
```

> A processor that expands `&rights;` inside the replacement text of `book`, producing `Title. All rights reserved` instead of `Title. &rights;`, would violate this rule.
