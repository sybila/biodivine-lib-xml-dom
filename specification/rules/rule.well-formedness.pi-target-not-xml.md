# Rule: PI Target Must Not Be Case-Insensitive Xml

A processing instruction target must not match the case-insensitive pattern "xml" (e.g., XML, xml, Xml, XmL).

## Rationale

> §2.6: The target names "XML", "xml", and so on are reserved for standardization in this or future versions of this specification. [17] PITarget ::= Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))

## Valid Example

```xml
<?myapp data?>
```

## Violating Example

```xml
<?XML version="1.0"?>
```