# Rule: NOTATION attribute values must match a declared notation name

Values of a NOTATION attribute type must match one of the notation names included in the declaration; all notation names in the declaration must be declared.

## Rationale

> [VC: Notation Attributes] Values of this type MUST match one of the notation names included in the declaration; all notation names in the declaration MUST be declared. (§3.3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (image)>
  <!ELEMENT image EMPTY>
  <!ATTLIST image type NOTATION (gif|png) #IMPLIED>
  <!NOTATION gif SYSTEM "gifviewer">
  <!NOTATION png SYSTEM "pngviewer">
]>
<root>
  <image type="gif"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (image)>
  <!ELEMENT image EMPTY>
  <!ATTLIST image type NOTATION (gif|png) #IMPLIED>
  <!NOTATION gif SYSTEM "gifviewer">
]>
<root>
  <image type="png"/>
</root>
```
