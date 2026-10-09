# Rule: Entity references in attribute values must be expanded

During attribute-value normalization, entity references must be recursively expanded using the entity's replacement text.

## Rationale

> For an entity reference, recursively apply step 3 of this algorithm to the replacement text of the entity. (§3.3.3, step 3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
  <!ENTITY greet "hello">
]>
<root label="&greet; world"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
  <!ENTITY greet "hello">
]>
<root label="&greet; world"/>
<!-- A violating example cannot be expressed in source XML alone.
     An implementation that passes the literal string "&greet; world"
     instead of the expanded "hello world" would violate this constraint. -->
```
