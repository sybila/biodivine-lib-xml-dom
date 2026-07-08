# Rule: An element type must not have more than one NOTATION attribute

An element type must not have more than one NOTATION attribute specified.

## Rationale

> [VC: One Notation Per Element Type] An element type MUST NOT have more than one NOTATION attribute specified. (§3.3.1)

## Valid Example

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

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT image EMPTY>
  <!ATTLIST image type NOTATION (gif|png) #IMPLIED format NOTATION (bmp|tiff) #IMPLIED>
  <!NOTATION gif SYSTEM "gifviewer">
  <!NOTATION png SYSTEM "pngviewer">
  <!NOTATION bmp SYSTEM "bmpviewer">
  <!NOTATION tiff SYSTEM "tiffviewer">
]>
<image type="gif"/>
```
