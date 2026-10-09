# Rule: Attribute values must not contain external entity references

Attribute values must not contain direct or indirect entity references to external entities.

## Rationale

> [WFC: No External Entity References] Attribute values MUST NOT contain direct or indirect entity references to external entities. (§3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
  <!ENTITY text "hello">
]>
<root label="&text;"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
  <!ENTITY ext SYSTEM "external.txt">
]>
<root label="&ext;"/>
```
