# Rule: No colons in typed attribute values (ID, IDREF, ENTITY, NOTATION)

In a namespace-valid document, no attributes with a declared type of ID, IDREF(S), ENTITY(IES), or NOTATION contain any colons.

## Rationale

> It follows that in a namespace-valid document: No attributes with a declared type of ID, IDREF(S), ENTITY(IES), or NOTATION contain any colons. (§7)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #REQUIRED>
]>
<root id="myid"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #REQUIRED>
]>
<root id="my:id"/>
```
