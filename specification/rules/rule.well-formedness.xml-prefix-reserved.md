# Rule: Xml-Prefixed Names Are Reserved

Names beginning with the case-insensitive string "xml" are reserved for standardization in this or future versions of the XML specification.

## Rationale

> §2.3: Names beginning with the string "xml", or with any string which would match (('X'|'x') ('M'|'m') ('L'|'l')), are reserved for standardization in this or future versions of this specification.

## Valid Example

```xml
<myxmltag>content</myxmltag>
```

## Violating Example

```xml
<xmltag>content</xmltag>
```