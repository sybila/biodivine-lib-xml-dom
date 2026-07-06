# Rule: Undeclared entity references in attribute values are an error

It is an error if an attribute value contains a reference to an entity for which no declaration has been read. The specific constraint depends on the document context:

- **[WFC: Entity Declared]** (§4.1): In a document without any DTD, or with only an internal DTD subset containing no parameter entity references, or with standalone='yes', entity references must match an entity declaration (except for the five predefined entities: amp, lt, gt, apos, quot).
- **[VC: Entity Declared]** (§4.1): In a document with an external subset or parameter entity references, if the document is not standalone, entity references must match an entity declaration.

## Rationale

> [WFC: Entity Declared] In a document without any DTD, a document with only an internal DTD subset which contains no parameter entity references, or a document with standalone='yes', for an entity reference that does not occur within the external subset or a parameter entity, the Name given in the entity reference MUST match that in an entity declaration that does not occur within the external subset or a parameter entity, except that well-formed documents need not declare any of the following entities: amp, lt, gt, apos, quot. (§4.1)

> [VC: Entity Declared] In a document with an external subset or parameter entity references, if the document is not standalone (either standalone='no' is specified or there is no standalone declaration), then the Name given in the entity reference MUST match that in an entity declaration. (§4.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
  <!ENTITY greet "hello">
]>
<root label="&greet;"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="&undeclared;"/>
```
