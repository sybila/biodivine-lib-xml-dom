# Rule: NMTOKEN values must match the Nmtoken production

Values of type NMTOKEN must match the Nmtoken production.

## Rationale

> [VC: Name Token] Values of type NMTOKEN MUST match the Nmtoken production; values of type NMTOKENS MUST match Nmtokens. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root token NMTOKEN #IMPLIED>
]>
<root token="valid_token"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root token NMTOKEN #IMPLIED>
]>
<root token="has space"/>
```
