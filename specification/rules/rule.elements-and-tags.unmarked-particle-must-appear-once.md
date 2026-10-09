# Rule: Unmarked particle must appear exactly once

The absence of a repetition operator on a name or content particle means that the element or content particle must appear exactly once.

## Rationale

> The absence of such an operator means that the element or content particle MUST appear exactly once.
> — §3.2.1 Element Content

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (a, b?)>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
]>
<root><a>1</a></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (a, b)>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
]>
<root><a>1</a></root>
```
