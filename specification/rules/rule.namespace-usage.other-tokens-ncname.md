# Rule: Non-element/attribute name tokens must be NCNames

All tokens in the document that are required by XML 1.0 well-formedness to match the Name production must instead match the NCName production, meaning they must not contain colons. This includes entity names, processing instruction targets, and notation names.

## Rationale

> §7 Conformance of Documents: "All other tokens in the document which are REQUIRED, for XML 1.0 well-formedness, to match the XML production for Name MUST match this specification's production for NCName."

> §7 Conformance of Documents: "It follows that in a namespace-well-formed document: No entity names, processing instruction targets, or notation names contain any colons."

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY copy "copyright">
]>
<root>&copy;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY my:entity "value">
]>
<root>&my:entity;</root>
```
