# Rule: General entity declaration must precede references in default attribute values

The declaration of a general entity must precede any reference to it which appears in a default value in an attribute-list declaration.

## Rationale

> The declaration of a general entity MUST precede any reference to it which appears in a default value in an attribute-list declaration. (§4.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY greet "hello">
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA "&greet;">
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA "&greet;">
  <!ENTITY greet "hello">
]>
<root/>
```
