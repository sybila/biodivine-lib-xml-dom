# Rule: IDREF values must match the Name production and reference an existing ID

Values of type IDREF must match the Name production, and each Name must match the value of an ID attribute on some element in the XML document.

## Rationale

> [VC: IDREF] Values of type IDREF MUST match the Name production, and values of type IDREFS MUST match Names; each Name MUST match the value of an ID attribute on some element in the XML document; i.e. IDREF values MUST match the value of some ID attribute. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (def, ref)>
  <!ELEMENT def EMPTY>
  <!ELEMENT ref EMPTY>
  <!ATTLIST def id ID #REQUIRED>
  <!ATTLIST ref target IDREF #REQUIRED>
]>
<root>
  <def id="term1"/>
  <ref target="term1"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (def, ref)>
  <!ELEMENT def EMPTY>
  <!ELEMENT ref EMPTY>
  <!ATTLIST def id ID #REQUIRED>
  <!ATTLIST ref target IDREF #REQUIRED>
]>
<root>
  <def id="term1"/>
  <ref target="nonexistent"/>
</root>
```
