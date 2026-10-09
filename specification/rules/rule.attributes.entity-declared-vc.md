# Rule: [VC] Entity references must reference declared entities (validity context)

In a document with an external subset or parameter entity references, if the document is not standalone, the Name given in an entity reference must match that in an entity declaration.

## Rationale

> [VC: Entity Declared] In a document with an external subset or parameter entity references, if the document is not standalone (either standalone='no' is specified or there is no standalone declaration), then the Name given in the entity reference MUST match that in an entity declaration. (§4.1)

## Valid Example

```xml
<?xml version="1.0" standalone="no"?>
<!DOCTYPE root SYSTEM "external.dtd" [
  <!ENTITY greet "hello">
]>
<root label="&greet;"/>
```

## Violating Example

```xml
<?xml version="1.0" standalone="no"?>
<!DOCTYPE root SYSTEM "external.dtd">
<root label="&undeclared;"/>
```
