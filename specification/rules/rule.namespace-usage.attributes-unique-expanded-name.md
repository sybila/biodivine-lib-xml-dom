# Rule: No duplicate attributes with same expanded name

No tag may contain two attributes with identical names, or with qualified names that share the same local part and prefixes bound to identical namespace names.

## Rationale

> §6.3 Uniqueness of Attributes: "[NSC: Attributes Unique] In XML documents conforming to this specification, no tag may contain two attributes which: (1) have identical names, or (2) have qualified names with the same local part and with prefixes which have been bound to namespace names that are identical."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:n1="http://ns1.org" xmlns:n2="http://ns2.org">
  <item n1:id="1" n2:id="2"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns:n1="http://www.w3.org" xmlns:n2="http://www.w3.org">
  <item n1:id="1" n2:id="2"/>
</root>
```
