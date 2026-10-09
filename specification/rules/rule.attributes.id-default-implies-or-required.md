# Rule: ID attributes must have #IMPLIED or #REQUIRED default

An ID attribute must have a declared default of #IMPLIED or #REQUIRED.

## Rationale

> [VC: ID Attribute Default] An ID attribute MUST have a declared default of #IMPLIED or #REQUIRED. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #REQUIRED>
]>
<root id="r1"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID "default_id">
]>
<root/>
```
