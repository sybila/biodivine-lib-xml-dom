# Rule: Element valid when declared ANY

When an element type is declared ANY, its content must consist of character data, CDATA sections, comments, PIs, and child elements whose types have been declared.

## Rationale

> Validity constraint: Element Valid — The declaration matches ANY, and the content (after replacing any entity references with their replacement text) consists of character data, CDATA sections, comments, PIs and child elements whose types have been declared.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root ANY>
  <!ELEMENT child (#PCDATA)>
]>
<root>text <child>more</child> text</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root ANY>
]>
<root><undeclared>text</undeclared></root>
```
