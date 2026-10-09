# Rule: An element type must not have more than one ID attribute

An element type must not have more than one ID attribute specified.

## Rationale

> [VC: One ID per Element Type] An element type MUST NOT have more than one ID attribute specified. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT item EMPTY>
  <!ATTLIST item id ID #REQUIRED label CDATA #IMPLIED>
]>
<item id="a" label="first"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT item EMPTY>
  <!ATTLIST item id ID #REQUIRED alt ID #IMPLIED>
]>
<item id="a"/>
```
