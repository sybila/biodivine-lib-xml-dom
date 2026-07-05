# Rule: Attribute default values must be syntactically correct for the declared type

The declared default value must meet the syntactic constraints of the declared attribute type.

## Rationale

> [VC: Attribute Default Value Syntactically Correct] The declared default value MUST meet the syntactic constraints of the declared attribute type. (§3.3.2)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root type (bullets|ordered|glossary) "ordered">
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root type (bullets|ordered|glossary) "numbered">
]>
<root/>
```
