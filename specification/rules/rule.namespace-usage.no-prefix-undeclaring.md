# Rule: Prefix namespace declaration value must not be empty

A namespace declaration for a prefix (xmlns:prefix) must not have an empty string as its attribute value.

## Rationale

> §5 Using Qualified Names: "[NSC: No Prefix Undeclaring] In a namespace declaration for a prefix (i.e., where the NSAttName is a PrefixedAttName), the attribute value MUST NOT be empty."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item>value</ex:item>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="">
  <ex:item>value</ex:item>
</root>
```
