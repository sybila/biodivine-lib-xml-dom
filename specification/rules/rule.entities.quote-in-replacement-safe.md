# Rule: Quote characters in replacement text must not terminate literal

When an entity reference appears in an attribute value or a parameter entity reference appears in a literal entity value, a single or double quote character in the replacement text must always be treated as a normal data character and must not terminate the literal.

## Rationale

> When an entity reference appears in an attribute value, or a parameter entity reference appears in a literal entity value, its replacement text MUST be processed in place of the reference itself as though it were part of the document at the location the reference was recognized, except that a single or double quote character in the replacement text MUST always be treated as a normal data character and MUST NOT terminate the literal. (§4.4.5)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % Y '"Yes"'>
  <!ENTITY WhatHeSaid "He said %Y;">
]>
<root attr="&WhatHeSaid;">text</root>
```

## Violating Example

```xml
<!-- If a processor treated the quote in &EndAttr;'s replacement
     text as terminating the attribute literal, this would break.
     The spec explicitly marks this pattern as not well-formed
     to illustrate the consequence of not following the rule. -->
<!ENTITY EndAttr "27'">
<!-- The quote in "27'" must NOT terminate the surrounding literal -->
```
