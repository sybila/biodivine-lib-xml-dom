# Rule: [WFC] Entity references in attribute values must reference declared entities

In a document without any DTD, or with only an internal DTD subset containing no parameter entity references, or with standalone='yes', the Name given in an entity reference must match that in an entity declaration (except for the five predefined entities: amp, lt, gt, apos, quot).

## Rationale

> [WFC: Entity Declared] In a document without any DTD, a document with only an internal DTD subset which contains no parameter entity references, or a document with standalone='yes', for an entity reference that does not occur within the external subset or a parameter entity, the Name given in the entity reference MUST match that in an entity declaration that does not occur within the external subset or a parameter entity, except that well-formed documents need not declare any of the following entities: amp, lt, gt, apos, quot. (§4.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ENTITY greet "hello">
]>
<root label="&greet;"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
]>
<root label="&undeclared;"/>
```
