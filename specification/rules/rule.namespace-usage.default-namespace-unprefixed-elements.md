# Rule: Default namespace applies to unprefixed elements

An unprefixed element name within the scope of a default namespace declaration takes that default namespace as its namespace name. Without a default namespace declaration in scope, the unprefixed element has no namespace.

## Rationale

> §6.2 Namespace Defaulting: "A default namespace declaration applies to all unprefixed element names within its scope."

> §6.2 Namespace Defaulting: "If there is a default namespace declaration in scope, the expanded name corresponding to an unprefixed element name has the URI of the default namespace as its namespace name. If there is no default namespace declaration in scope, the namespace name has no value."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns="http://example.org">
  <item>in default namespace</item>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!-- An element with an undeclared prefix is incorrectly assumed
     to be in the default namespace when xmlns="" has removed it.
     The prefix "ex" must be declared to use ex:item. -->
<root xmlns="http://example.org">
  <ex:item xmlns=""/>
</root>
```
