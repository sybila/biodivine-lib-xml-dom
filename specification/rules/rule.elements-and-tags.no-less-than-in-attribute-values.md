# Rule: No less-than in attribute values

The replacement text of any entity referred to directly or indirectly in an attribute value must not contain a `<`.

## Rationale

> Well-formedness constraint: No < in Attribute Values — The replacement text of any entity referred to directly or indirectly in an attribute value MUST NOT contain a <.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<root attr="a &lt; b">content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [<!ENTITY lt "<">]>
<root attr="a &lt; b">content</root>
```
