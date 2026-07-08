# Rule: #REQUIRED attributes must be specified on all elements of the type

If an attribute's default declaration is #REQUIRED, then the attribute must be specified for all elements of the type in the attribute-list declaration.

## Rationale

> [VC: Required Attribute] If the default declaration is the keyword #REQUIRED, then the attribute MUST be specified for all elements of the type in the attribute-list declaration. (§3.3.2)

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
  <!ATTLIST root id ID #REQUIRED>
]>
<root/>
```
