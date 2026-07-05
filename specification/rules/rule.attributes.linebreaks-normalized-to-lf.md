# Rule: Line breaks in attribute values must be normalized to #xA

All line breaks in attribute values must have been normalized to #xA on input before attribute-value normalization proceeds.

## Rationale

> All line breaks MUST have been normalized on input to #xA as described in 2.11 End-of-Line Handling, so the rest of this algorithm operates on text normalized in this way. (§3.3.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="line1
line2"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="line1
line2"/>
<!-- A violating example cannot be expressed in source XML alone.
     An implementation that fails to normalize line breaks to #xA
     before attribute-value normalization would violate this constraint. -->
```
