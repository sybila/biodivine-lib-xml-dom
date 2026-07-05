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

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY rights "All rights reserved">
  <!ENTITY book "Title. &rights;">
]>
<root>&book;</root>
```
