# Rule: Parameter entity replacement text must be properly nested with markup declarations

If either the first character or the last character of a markup declaration is contained in the replacement text for a parameter-entity reference, both must be contained in the same replacement text.

## Rationale

> [VC: Proper Declaration/PE Nesting] Parameter-entity replacement text MUST be properly nested with markup declarations. That is to say, if either the first character or the last character of a markup declaration (markupdecl above) is contained in the replacement text for a parameter-entity reference, both MUST be contained in the same replacement text. (§2.8)

## Valid Example

```xml
<!DOCTYPE root [
  <!ENTITY % term '<!ELEMENT term (#PCDATA)>'>
  %term;
]>
<root><term>definition</term></root>
```

## Violating Example

```xml
<!DOCTYPE root [
  <!ENTITY % broken '<!ELEMENT term'>
  %broken; (#PCDATA)>;
]>
<root><term>definition</term></root>
```
