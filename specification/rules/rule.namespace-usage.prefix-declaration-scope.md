# Rule: Prefix declaration scope extends from start-tag to end-tag

A namespace declaration for a prefix applies from the start-tag in which it appears to the corresponding end-tag, excluding any inner declarations with the same prefix.

## Rationale

> §6.1 Namespace Scoping: "The scope of a namespace declaration declaring a prefix extends from the beginning of the start-tag in which it appears to the end of the corresponding end-tag, excluding the scope of any inner declarations with the same NSAttName part."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item>value</ex:item>
  <child xmlns:ex="http://other.org">
    <ex:item>other namespace</ex:item>
  </child>
  <ex:after>back to original</ex:after>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>
  <child xmlns:ex="http://example.org"/>
  <ex:item>prefix not in scope here</ex:item>
</root>
```
