# Rule: Non-CDATA attribute values must have leading and trailing spaces stripped

For attribute types other than CDATA, the XML processor must discard any leading and trailing space (#x20) characters from the normalized attribute value.

## Rationale

> If the attribute type is not CDATA, then the XML processor MUST further process the normalized attribute value by discarding any leading and trailing space (#x20) characters, and by replacing sequences of space (#x20) characters by a single space (#x20) character. (§3.3.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root token NMTOKEN #IMPLIED>
]>
<root token="  hello  "/>
<!-- The normalized value is "hello" (leading/trailing spaces stripped) -->
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root token NMTOKEN #IMPLIED>
]>
<root token="  hello  "/>
<!-- A violating example cannot be expressed in source XML alone.
     An implementation that passes "  hello  " (with leading/trailing
     spaces) as the value of an NMTOKEN attribute would violate this
     constraint, since NMTOKEN must match the Nmtoken production and
     leading/trailing spaces would cause a mismatch. -->
```
