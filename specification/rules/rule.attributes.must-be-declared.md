# Rule: Attributes must be declared for validity

An attribute used on an element must have been declared in the DTD; the value must be of the type declared for it.

## Rationale

> [VC: Attribute Value Type] The attribute MUST have been declared; the value MUST be of the type declared for it. (§3.1)

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
<root id="r1" undeclared="value"/>
```
