# Rule: Proper group/PE nesting

Parameter-entity replacement text must be properly nested with parenthesized groups; if either the opening or closing parenthesis in a choice, seq, or Mixed construct is in parameter-entity replacement text, both must be in the same replacement text.

## Rationale

> Validity constraint: Proper Group/PE Nesting — Parameter-entity replacement text MUST be properly nested with parenthesized groups. That is to say, if either of the opening or closing parentheses in a choice, seq, or Mixed construct is contained in the replacement text for a parameter entity, both MUST be contained in the same replacement text.
> — §3.2.1 Element Content

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % group "(a | b)">
  <!ELEMENT root %group;*>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
]>
<root><a>1</a><b>2</b></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY % partial "a | b)">
  <!ELEMENT root (%partial;*>
  <!ELEMENT a (#PCDATA)>
  <!ELEMENT b (#PCDATA)>
]>
<root><a>1</a></root>
```
