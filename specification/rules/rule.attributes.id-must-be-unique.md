# Rule: ID values must be unique across the document

A name must not appear more than once in an XML document as a value of type ID; ID values must uniquely identify the elements which bear them.

## Rationale

> [VC: ID] A name MUST NOT appear more than once in an XML document as a value of this type; i.e., ID values MUST uniquely identify the elements which bear them. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item*)>
  <!ELEMENT item EMPTY>
  <!ATTLIST item id ID #REQUIRED>
]>
<root>
  <item id="a"/>
  <item id="b"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (item*)>
  <!ELEMENT item EMPTY>
  <!ATTLIST item id ID #REQUIRED>
]>
<root>
  <item id="a"/>
  <item id="a"/>
</root>
```
