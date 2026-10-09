# Rule: Parameter entity references must not appear outside the DTD

Parameter-entity references must not appear outside the DTD.

## Rationale

> [WFC: In DTD] Parameter-entity references MUST NOT appear outside the DTD. (§4.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % pe "para">
  <!ELEMENT root (%pe;)*>
]>
<root>text</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % pe "para">
  <!ELEMENT root (#PCDATA)>
]>
<root>%pe;</root>
```
