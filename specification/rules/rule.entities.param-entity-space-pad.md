# Rule: Parameter entity replacement text in DTD must be space-padded

When a parameter-entity reference is recognized in the DTD and included, its replacement text must be enlarged by the attachment of one leading and one following space (#x20) character.

## Rationale

> When a parameter-entity reference is recognized in the DTD and included, its replacement text MUST be enlarged by the attachment of one leading and one following space (#x20) character; the intent is to constrain the replacement text of parameter entities to contain an integral number of grammatical tokens in the DTD. (§4.4.8)

## Valid Example

```xml
<!-- %pe; expands to " hello " (with surrounding spaces) in the DTD -->
<!DOCTYPE root [
  <!ENTITY % pe "hello">
  <!ELEMENT root (%pe;)*>
  <!-- Effectively: <!ELEMENT root ( hello )* -->
]>
<root/>
```

## Violating Example

```xml
<!-- If a processor failed to add space padding, %pe; would expand
     to "hello" without spaces, causing token merging. For example,
     <!ELEMENT root before%pe;after> would become
     <!ELEMENT root beforehelloafter> (one invalid token)
     instead of <!ELEMENT root before hello after> (three tokens). -->
<!DOCTYPE root [
  <!ENTITY % pe "hello">
  <!ELEMENT root before%pe;after>
]>
<root/>
```
