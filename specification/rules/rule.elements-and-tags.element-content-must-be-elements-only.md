# Rule: Element content must be elements only

An element type with element content must contain only child elements, optionally separated by whitespace.

## Rationale

> An element type has element content when elements of that type MUST contain only child elements (no character data), optionally separated by white space (characters matching the nonterminal S).
> — §3.2.1 Element Content

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (section+)>
  <!ELEMENT section (title, paragraph*)>
  <!ELEMENT title (#PCDATA)>
  <!ELEMENT paragraph (#PCDATA)>
]>
<root><section><title>Hello</title><paragraph>World</paragraph></section></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (section+)>
  <!ELEMENT section (title, paragraph*)>
  <!ELEMENT title (#PCDATA)>
  <!ELEMENT paragraph (#PCDATA)>
]>
<root>text before<section><title>Hello</title><paragraph>World</paragraph></section></root>
```
