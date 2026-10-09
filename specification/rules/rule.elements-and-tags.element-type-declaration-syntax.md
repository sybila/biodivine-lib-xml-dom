# Rule: Element type declaration syntax

An element type declaration must match the production `<!ELEMENT S Name S contentspec S?>`, where contentspec is `EMPTY`, `ANY`, `Mixed`, or `children`.

## Rationale

> [45] elementdecl ::= '<!ELEMENT' S Name S contentspec S? '>'
> [46] contentspec ::= 'EMPTY' | 'ANY' | Mixed | children
> — §3.2 Element Type Declarations

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ELEMENT para (#PCDATA|emph)*>
  <!ELEMENT container (header, body, footer?)>
  <!ELEMENT free ANY>
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root>
]>
<root/>
```
