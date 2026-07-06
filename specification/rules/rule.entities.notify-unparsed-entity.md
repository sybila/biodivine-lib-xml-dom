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

> A validating processor that provides the application with the system identifier of the unparsed entity `image` and its notation `gif` satisfies this rule.

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

> A validating processor that fails to provide the application with the entity and notation identifiers would violate this rule.
