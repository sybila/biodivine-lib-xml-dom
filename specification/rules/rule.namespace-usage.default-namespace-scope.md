# Rule: Default namespace declaration scope extends from start-tag to end-tag

A default namespace declaration applies from the start-tag in which it appears to the corresponding end-tag, excluding any inner default namespace declarations.

## Rationale

> §6.2 Namespace Defaulting: "The scope of a default namespace declaration extends from the beginning of the start-tag in which it appears to the end of the corresponding end-tag, excluding the scope of any inner default namespace declarations."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns="http://example.org">
  <item>in default namespace</item>
  <child xmlns="http://other.org">
    <item>in other namespace</item>
  </child>
  <after>back to original default</after>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>
  <child xmlns="http://example.org"/>
  <item>default namespace not in scope here</item>
</root>
```
