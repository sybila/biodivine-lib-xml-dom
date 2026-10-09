# Rule: Non-CDATA attribute values must collapse space sequences to single space

For attribute types other than CDATA, the XML processor must replace sequences of space (#x20) characters with a single space (#x20) character.

## Rationale

> If the attribute type is not CDATA, then the XML processor MUST further process the normalized attribute value by discarding any leading and trailing space (#x20) characters, and by replacing sequences of space (#x20) characters by a single space (#x20) character. (§3.3.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root tokens NMTOKENS #IMPLIED>
]>
<root tokens="a   b   c"/>
<!-- The normalized value is "a b c" (space sequences collapsed) -->
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root tokens NMTOKENS #IMPLIED>
]>
<root tokens="a   b   c"/>
<!-- A violating example cannot be expressed in source XML alone.
     An implementation that passes "a   b   c" (with multiple spaces)
     as the value of an NMTOKENS attribute would violate this constraint,
     since Nmtokens expects space-separated Nmtoken values. -->
```
