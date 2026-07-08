# Rule: Attribute names must be unique within a start-tag

An attribute name must not appear more than once in the same start-tag or empty-element tag.

## Rationale

> [WFC: Unique Att Spec] An attribute name MUST NOT appear more than once in the same start-tag or empty-element tag. (§3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #IMPLIED class NMTOKEN #IMPLIED>
]>
<root id="r1" class="main"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #IMPLIED>
]>
<root id="r1" id="r2"/>
```
