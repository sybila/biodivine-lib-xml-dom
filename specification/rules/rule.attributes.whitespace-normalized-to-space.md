# Rule: Whitespace characters in attribute values must be normalized to space

During attribute-value normalization, each whitespace character (#x20, #xD, #xA, #x9) must be replaced with a space character (#x20).

## Rationale

> For a white space character (#x20, #xD, #xA, #x9), append a space character (#x20) to the normalized value. (§3.3.3, step 3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="hello&#x9;world"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="hello&#x9;world"/>
<!-- A violating example cannot be expressed in source XML alone.
     An implementation that preserves the tab character (#x9) instead
     of replacing it with a space (#x20) would violate this constraint. -->
```
