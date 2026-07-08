# Rule: NMTOKENS values must match the Nmtokens production

Values of type NMTOKENS must match the Nmtokens production (a space-separated list of Nmtoken values).

## Rationale

> [VC: Name Token] Values of type NMTOKENS MUST match Nmtokens. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root tokens NMTOKENS #IMPLIED>
]>
<root tokens="token1 token2"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root tokens NMTOKENS #IMPLIED>
]>
<root tokens="valid invalid-name-with-dash!"/>
```
