# Rule: xml:lang attribute must be declared in valid documents

In valid documents, the xml:lang attribute must be declared if it is used.

## Rationale

> In valid documents, this attribute, like any other, MUST be declared if it is used. (§2.12)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ATTLIST root xml:lang CDATA #IMPLIED>
]>
<root xml:lang="en">Hello</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (#PCDATA)>
]>
<root xml:lang="en">Hello</root>
```
