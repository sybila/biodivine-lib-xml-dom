# Rule: Empty element representation

An empty element must be represented either as a start-tag immediately followed by an end-tag, or as an empty-element tag using the special form `< Name (S Attribute)* S? />`.

## Rationale

> The representation of an empty element is either a start-tag immediately followed by an end-tag, or an empty-element tag.
> [44] EmptyElemTag ::= '<' Name (S Attribute)* S? '/>'
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<root><br/><br></br></root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root><br>text</br></root>
```
