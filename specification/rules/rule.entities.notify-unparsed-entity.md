# Rule: Validating processor must notify of unparsed entity identifiers

When the name of an unparsed entity appears as a token in the value of an attribute of declared type ENTITY or ENTITIES, a validating processor must inform the application of the system and public (if any) identifiers for both the entity and its associated notation.

## Rationale

> When the name of an unparsed entity appears as a token in the value of an attribute of declared type ENTITY or ENTITIES, a validating processor MUST inform the application of the system and public (if any) identifiers for both the entity and its associated notation. (§4.4.6)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!ENTITY image SYSTEM "image.gif" NDATA gif>
  <!ATTLIST root img ENTITY #IMPLIED>
]>
<root img="image"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!ENTITY image SYSTEM "image.gif" NDATA gif>
  <!ATTLIST root img ENTITY #IMPLIED>
]>
<root img="image"/>
```
