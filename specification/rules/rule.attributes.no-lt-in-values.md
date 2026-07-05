# Rule: Attribute values must not contain less-than sign

The replacement text of any entity referred to directly or indirectly in an attribute value must not contain a less-than sign.

## Rationale

> [WFC: No < in Attribute Values] The replacement text of any entity referred to directly or indirectly in an attribute value MUST NOT contain a <. (§3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
  <!ENTITY amp "&lt;">
]>
<root label="5 &amp; 10"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
  <!ENTITY bad "<">
]>
<root label="&bad;"/>
```
