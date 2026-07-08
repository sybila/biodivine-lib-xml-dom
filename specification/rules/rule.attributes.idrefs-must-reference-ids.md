# Rule: IDREFS values must match Names production and each name must reference an existing ID

Values of type IDREFS must match the Names production; each Name must match the value of an ID attribute on some element in the XML document.

## Rationale

> [VC: IDREF] Values of type IDREFS MUST match Names; each Name MUST match the value of an ID attribute on some element in the XML document. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (def*, ref)>
  <!ELEMENT def EMPTY>
  <!ELEMENT ref EMPTY>
  <!ATTLIST def id ID #REQUIRED>
  <!ATTLIST ref targets IDREFS #REQUIRED>
]>
<root>
  <def id="a"/>
  <def id="b"/>
  <ref targets="a b"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (def*, ref)>
  <!ELEMENT def EMPTY>
  <!ELEMENT ref EMPTY>
  <!ATTLIST def id ID #REQUIRED>
  <!ATTLIST ref targets IDREFS #REQUIRED>
]>
<root>
  <def id="a"/>
  <ref targets="a missing"/>
</root>
```
