# Rule: Element valid when declared EMPTY

When an element type is declared EMPTY, the element must have no content (not even entity references, comments, PIs, or whitespace).

## Rationale

> Validity constraint: Element Valid — The declaration matches EMPTY and the element has no content (not even entity references, comments, PIs or white space).
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT br EMPTY>
]>
<root><br/></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT br EMPTY>
]>
<root><br> </br></root>
```
