# Rule: Space padding must not apply to parameter entities within entity values

The space-padding behavior for parameter entity replacement text must not apply to parameter entity references within entity values.

## Rationale

> This behavior MUST NOT apply to parameter entity references within entity values; these are described in 4.4.5 Included in Literal. (§4.4.8)

## Valid Example

```xml
<!-- %pe; inside EntityValue expands without space padding -->
<!DOCTYPE root [
  <!ENTITY % pe "world">
  <!ENTITY greeting "Hello, %pe;!">
  <!-- Replacement text: "Hello, world!" (no extra spaces) -->
]>
```

## Violating Example

```xml
<!-- If a processor incorrectly applied space padding inside
     EntityValue, the result would be "Hello,  world ;!"
     instead of "Hello, world!" -->
<!DOCTYPE root [
  <!ENTITY % pe "world">
  <!ENTITY greeting "Hello, %pe;!">
  <!-- Wrong: "Hello,  world ;!" (with padding spaces)
     Right: "Hello, world!" (no padding) -->
]>
```
