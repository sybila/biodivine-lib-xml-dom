# Rule: ENTITIES values must match Names and each name must reference a declared unparsed entity

Values of type ENTITIES must match the Names production; each Name must match the name of an unparsed entity declared in the DTD.

## Rationale

> [VC: Entity Name] Values of type ENTITIES MUST match Names; each Name MUST match the name of an unparsed entity declared in the DTD. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item)>
  <!ELEMENT item EMPTY>
  <!ATTLIST item files ENTITIES #REQUIRED>
  <!NOTATION gif SYSTEM "gifviewer">
  <!NOTATION jpg SYSTEM "jpgviewer">
  <!ENTITY img1 SYSTEM "a.gif" NDATA gif>
  <!ENTITY img2 SYSTEM "b.jpg" NDATA jpg>
]>
<root>
  <item files="img1 img2"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item)>
  <!ELEMENT item EMPTY>
  <!ATTLIST item files ENTITIES #REQUIRED>
  <!NOTATION gif SYSTEM "gifviewer">
  <!ENTITY img1 SYSTEM "a.gif" NDATA gif>
]>
<root>
  <item files="img1 missing"/>
</root>
```
