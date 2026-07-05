# Rule: ENTITY values must match Name production and reference a declared unparsed entity

Values of type ENTITY must match the Name production; each Name must match the name of an unparsed entity declared in the DTD.

## Rationale

> [VC: Entity Name] Values of type ENTITY MUST match the Name production, values of type ENTITIES MUST match Names; each Name MUST match the name of an unparsed entity declared in the DTD. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item)>
  <!ELEMENT item EMPTY>
  <!ATTLIST item file ENTITY #REQUIRED>
  <!ENTITY % image SYSTEM "image.ent">
  <!NOTATION gif SYSTEM "gifviewer">
  <!ENTITY mygif NDATA gif SYSTEM "image.gif">
]>
<root>
  <item file="mygif"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item)>
  <!ELEMENT item EMPTY>
  <!ATTLIST item file ENTITY #REQUIRED>
]>
<root>
  <item file="undeclared_entity"/>
</root>
```
