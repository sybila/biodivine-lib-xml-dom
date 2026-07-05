# Rule: Standalone declaration must be "no" when external markup declarations affect document content

The standalone document declaration must have the value "no" if any external markup declarations contain declarations of attributes with default values, entities (other than amp, lt, gt, apos, quot) referenced in the document, attributes with tokenized types where normalization differs, or element types with element content where whitespace occurs.

## Rationale

> [VC: Standalone Document Declaration] The standalone document declaration MUST have the value "no" if any external markup declarations contain declarations of: attributes with default values, if elements to which these attributes apply appear in the document without specifications of values for these attributes, or entities (other than amp, lt, gt, apos, quot), if references to those entities appear in the document, or attributes with tokenized types, where the attribute appears in the document with a value such that normalization will produce a different value from that which would be produced in the absence of the declaration, or element types with element content, if white space occurs directly within any instance of those types. (§2.9)

## Valid Example

```xml
<?xml version="1.0" standalone="no"?>
<!DOCTYPE root SYSTEM "external.dtd">
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0" standalone="yes"?>
<!DOCTYPE root SYSTEM "external.dtd">
<root attr="value">content</root>
```
