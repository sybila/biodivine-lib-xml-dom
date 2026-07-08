# Rule: Character and general-entity references forbidden in DTD outside EntityValue or AttValue

The appearance of any character or general-entity reference in the DTD except within an EntityValue or AttValue is forbidden and constitutes a fatal error.

## Rationale

> The following are forbidden, and constitute fatal errors: the appearance of any character or general-entity reference in the DTD except within an EntityValue or AttValue. (§4.4.4)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY greeting "Hello">
  <!ATTLIST root msg CDATA "&greeting;">
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (&greeting;)*>
  <!ENTITY greeting "para">
]>
```
