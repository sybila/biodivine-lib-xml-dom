# Rule: Empty default namespace declaration removes default namespace

A default namespace declaration with an empty string value removes the default namespace within its scope, making unprefixed elements belong to no namespace.

## Rationale

> §6.2 Namespace Defaulting: "The attribute value in a default namespace declaration MAY be empty. This has the same effect, within the scope of the declaration, of there being no default namespace."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns="http://example.org">
  <item>in default namespace</item>
  <child xmlns="">
    <item>no namespace</item>
  </child>
  <after>back to default namespace</after>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns="http://example.org">
  <item xmlns="">in no namespace, not default</item>
</root>
```
