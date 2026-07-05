# Rule: ID values must match the Name production

Values of type ID must match the Name production.

## Rationale

> [VC: ID] Values of type ID MUST match the Name production. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #REQUIRED>
]>
<root id="my_element"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #REQUIRED>
]>
<root id="123abc"/>
```
