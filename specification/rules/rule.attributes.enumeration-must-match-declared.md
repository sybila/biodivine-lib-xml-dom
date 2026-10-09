# Rule: Enumerated attribute values must match one of the declared values

Enumerated attributes must take one of the values listed in their declaration.

## Rationale

> Enumerated attributes have a list of allowed values in their declaration. They MUST take one of those values. (§3.3.1)

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
  <!ATTLIST root type (bullets|ordered|glossary) "ordered">
]>
<root type="numbered"/>
```
