# Rule: Element valid with mixed content model

When an element type is declared with a Mixed content model, its content must consist of character data, comments, PIs, and child elements whose types match names in the content model.

## Rationale

> Validity constraint: Element Valid — The declaration matches Mixed, and the content (after replacing any entity references with their replacement text) consists of character data (including CDATA sections), comments, PIs and child elements whose types match names in the content model.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT p (#PCDATA|em|b)*>
  <!ELEMENT em (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
]>
<p>This is <em>emphasized</em> and <b>bold</b> text.</p>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT p (#PCDATA|em|b)*>
  <!ELEMENT em (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
  <!ELEMENT i (#PCDATA)>
]>
<p>This has <i>invalid</i> child.</p>
```
