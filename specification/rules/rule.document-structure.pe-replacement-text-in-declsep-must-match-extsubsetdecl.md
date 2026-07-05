# Rule: Parameter entity replacement text in DeclSep must match extSubsetDecl

The replacement text of a parameter entity reference in a DeclSep must match the production extSubsetDecl.

## Rationale

> [WFC: PE Between Declarations] The replacement text of a parameter entity reference in a DeclSep MUST match the production extSubsetDecl. (§2.8)

## Valid Example

```xml
<!DOCTYPE root SYSTEM "external.dtd" [
  <!ENTITY % term '<!ELEMENT term (#PCDATA)>'>
  %term;
]>
<root><term>definition</term></root>
```

## Violating Example

```xml
<!DOCTYPE root SYSTEM "external.dtd" [
  <!ENTITY % invalid '<!ELEMENT term'>
  %invalid;
]>
<root><term>definition</term></root>
```
