# Rule: Element and attribute names must be qualified names

Element and attribute names in namespace-aware XML documents must match the QName production, consisting of either a PrefixedName (Prefix:LocalPart) or an UnprefixedName (LocalPart).

## Rationale

> §4 Qualified Names: "In XML documents conforming to this specification, some names (constructs corresponding to the nonterminal Name) MUST be given as qualified names."

> §7 Conformance of Documents: "In XML documents which conform to this specification, element and attribute names MUST match the production for QName and MUST satisfy the 'Namespace Constraints'."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item id="1"/>
  <description>unprefixed element</description>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item id="1"/>
  <!-- A QName can have at most one colon: Prefix:LocalPart. -->
  <ex:item:detail name="bad"/>
</root>
```
