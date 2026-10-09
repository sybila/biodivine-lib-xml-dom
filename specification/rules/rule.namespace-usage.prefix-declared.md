# Rule: Prefix must be declared before use

A namespace prefix, other than the predefined prefixes `xml` and `xmlns`, must be declared via a namespace declaration attribute on the element where it is used or on an ancestor element.

## Rationale

> §5 Using Qualified Names: "[NSC: Prefix Declared] The namespace prefix, unless it is `xml` or `xmlns`, MUST have been declared in a namespace declaration attribute in either the start-tag of the element where the prefix is used or in an ancestor element (i.e., an element in whose content the prefixed markup occurs)."

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
<root>
  <ex:item>value</ex:item>
</root>
```
