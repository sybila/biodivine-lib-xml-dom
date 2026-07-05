# Rule: Sequence particles must appear in order

Content particles occurring in a sequence list must each appear in the element content in the order given in the list.

## Rationale

> Content particles occurring in a sequence list MUST each appear in the element content in the order given in the list.
> — §3.2.1 Element Content

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (a, b, c)>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
  <!ELEMENT c (#PCDATA)>
]>
<root><a>1</a><b>2</b><c>3</c></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (a, b, c)>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
  <!ELEMENT c (#PCDATA)>
]>
<root><c>3</c><b>2</b><a>1</a></root>
```
