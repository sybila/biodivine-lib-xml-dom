# Rule: Parameter entity references must not occur within markup declarations in internal subset

In the internal DTD subset, parameter-entity references must not occur within markup declarations; they may only occur where markup declarations can occur.

## Rationale

> [WFC: PEs in Internal Subset] In the internal DTD subset, parameter-entity references MUST NOT occur within markup declarations; they may occur where markup declarations can occur. (§2.8)

## Valid Example

```xml
<!DOCTYPE root [
  <!ELEMENT root (#PCDATA)>
  <!ENTITY % term '<!ELEMENT term (#PCDATA)>'>
  %term;
]>
<root><term>definition</term></root>
```

## Violating Example

```xml
<!DOCTYPE root [
  <!ENTITY % p '%term;'>
  <!ELEMENT root (%p;) >
]>
<root>content</root>
```
