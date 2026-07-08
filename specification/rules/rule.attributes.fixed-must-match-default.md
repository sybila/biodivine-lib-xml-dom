# Rule: #FIXED attribute instances must match the fixed default value

If an attribute has a default value declared with the #FIXED keyword, instances of that attribute must match the default value.

## Rationale

> [VC: Fixed Attribute Default] If an attribute has a default value declared with the #FIXED keyword, instances of that attribute MUST match the default value. (§3.3.2)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root method CDATA #FIXED "POST">
]>
<root method="POST"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root method CDATA #FIXED "POST">
]>
<root method="GET"/>
```
