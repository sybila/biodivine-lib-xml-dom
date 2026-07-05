# Rule: Start-tag syntax

A start-tag must match the production `< Name (S Attribute)* S? >`, where each Attribute is a Name followed by `=` followed by an AttValue.

## Rationale

> [40] STag ::= '<' Name (S Attribute)* S? '>'
> [41] Attribute ::= Name Eq AttValue
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<root id="1" class="main">content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root id 1 class="main">content</root>
```
