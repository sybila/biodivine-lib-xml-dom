# Rule: Character references in attribute values must be expanded

During attribute-value normalization, character references must be expanded by appending the referenced character to the normalized value.

## Rationale

> For a character reference, append the referenced character to the normalized value. (§3.3.3, step 3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="&#60;hello&#62;"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="&#60;hello&#62;"/>
<!-- A violating example cannot be expressed in source XML alone.
     An implementation that passes the literal string "&#60;hello&#62;"
     instead of the expanded "<hello>" would violate this constraint. -->
```
