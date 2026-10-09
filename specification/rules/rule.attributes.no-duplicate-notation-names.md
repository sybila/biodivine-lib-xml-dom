# Rule: NOTATION names in a single declaration must be distinct

The notation names in a single NotationType attribute declaration must all be distinct.

## Rationale

> [VC: No Duplicate Tokens] The notation names in a single NotationType attribute declaration, as well as the NmTokens in a single Enumeration attribute declaration, MUST all be distinct. (§3.3.1)

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
  <!ELEMENT image (#PCDATA)>
  <!ATTLIST image type NOTATION (gif|png|gif) #IMPLIED>
  <!NOTATION gif SYSTEM "gifviewer">
  <!NOTATION png SYSTEM "pngviewer">
]>
<image type="gif">content</image>
```
