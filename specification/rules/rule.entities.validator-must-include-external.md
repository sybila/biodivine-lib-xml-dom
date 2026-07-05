# Rule: Validating processor must include external parsed entity replacement text

When an XML processor recognizes a reference to a parsed entity, in order to validate the document, the processor must include its replacement text.

## Rationale

> When an XML processor recognizes a reference to a parsed entity, in order to validate the document, the processor MUST include its replacement text. (§4.4.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY external SYSTEM "external.xml">
  <!ELEMENT root (#PCDATA)>
]>
<root>&external;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY external SYSTEM "external.xml">
  <!ELEMENT root (#PCDATA)>
]>
<root>&external;</root>
```
