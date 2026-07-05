# Rule: Undeclared entity references in attribute values are an error

It is an error if an attribute value contains a reference to an entity for which no declaration has been read.

## Rationale

> It is an error if an attribute value contains a reference to an entity for which no declaration has been read. (§3.3.3)

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
