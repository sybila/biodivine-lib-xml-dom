# Rule: Enumeration tokens in a single declaration must be distinct

The Nmtoken values in a single Enumeration attribute declaration must all be distinct.

## Rationale

> [VC: No Duplicate Tokens] The notation names in a single NotationType attribute declaration, as well as the NmTokens in a single Enumeration attribute declaration, MUST all be distinct. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root type (bullets|ordered|glossary) "ordered">
]>
<root type="bullets"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root type (bullets|ordered|bullets) "ordered">
]>
<root type="bullets"/>
```
