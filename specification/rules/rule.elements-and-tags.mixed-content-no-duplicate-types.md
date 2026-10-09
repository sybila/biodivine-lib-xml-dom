# Rule: Mixed content no duplicate types

The same name must not appear more than once in a single mixed-content declaration.

## Rationale

> Validity constraint: No Duplicate Types — The same name MUST NOT appear more than once in a single mixed-content declaration.
> — §3.2.2 Mixed Content

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT p (#PCDATA|a|b|em)*>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
  <!ELEMENT em (#PCDATA)>
]>
<p>text <a>link</a> and <b>bold</b></p>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT p (#PCDATA|a|a|b)*>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
]>
<p>text <a>link</a></p>
```
