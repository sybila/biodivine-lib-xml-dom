# Rule: NOTATION attributes must not be declared on EMPTY elements

An attribute of type NOTATION must not be declared on an element declared EMPTY.

## Rationale

> [VC: No Notation on Empty Element] For compatibility, an attribute of type NOTATION MUST NOT be declared on an element declared EMPTY. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT image (#PCDATA)>
  <!ATTLIST image type NOTATION (gif|png) #IMPLIED>
  <!NOTATION gif SYSTEM "gifviewer">
  <!NOTATION png SYSTEM "pngviewer">
]>
<image type="gif">content</image>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT image EMPTY>
  <!ATTLIST image type NOTATION (gif|png) #IMPLIED>
  <!NOTATION gif SYSTEM "gifviewer">
  <!NOTATION png SYSTEM "pngviewer">
]>
<image type="gif"/>
```
