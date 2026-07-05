# Rule: Entity declaration must precede references in default attribute values

The declaration of a general entity must precede any reference to it which appears in a default value in an attribute-list declaration.

## Rationale

> The declaration of a general entity MUST precede any reference to it which appears in a default value in an attribute-list declaration. (§4.1, [WFC: Entity Declared])

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY unit "kg">
  <!ATTLIST root weight CDATA "#&unit;">
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ATTLIST root weight CDATA "#&unit;">
  <!ENTITY unit "kg">
]>
<root/>
```
