# Rule: Replacement text markup must be recognized

When an entity is included, its replacement text may contain both character data and markup, which must be recognized in the usual way.

## Rationale

> The replacement text may contain both character data and (except for parameter entities) markup, which MUST be recognized in the usual way. (§4.4.2)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY bold "<b>important</b>">
]>
<root>&bold; text</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY bold "<b>unclosed">
]>
<root>&bold; text</root>
```
