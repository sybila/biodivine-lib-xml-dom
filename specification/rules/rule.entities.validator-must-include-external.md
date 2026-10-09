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

> A validating processor that reads and includes the replacement text of `external.xml` in the document content satisfies this rule.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY external SYSTEM "external.xml">
  <!ELEMENT root (#PCDATA)>
]>
<root>&external;</root>
```

> A validating processor that recognizes the entity reference but fails to include the replacement text from `external.xml` in the document content would violate this rule.
