# Rule: Attribute value type

An attribute must have been declared, and its value must be of the type declared for it.

## Rationale

> Validity constraint: Attribute Value Type — The attribute MUST have been declared; the value MUST be of the type declared for it.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [<!ATTLIST root id ID #REQUIRED>]>
<root id="r1">content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [<!ATTLIST root id ID #REQUIRED>]>
<root id="not a valid ID">content</root>
```
