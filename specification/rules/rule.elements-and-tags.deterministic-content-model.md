# Rule: Content models must be deterministic (compatibility)

For compatibility, it is an error if the content model allows an element to match more than one occurrence of an element type in the content model.

## Rationale

> For compatibility, it is an error if the content model allows an element to match more than one occurrence of an element type in the content model. For more information, see E Deterministic Content Models. (§3.2.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (a, b)>
  <!ELEMENT a EMPTY>
  <!ELEMENT b EMPTY>
]>
<root><a/><b/></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (a | a)>
  <!ELEMENT a EMPTY>
]>
<root><a/></root>
```
