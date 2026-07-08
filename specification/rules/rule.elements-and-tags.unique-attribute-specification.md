# Rule: Unique attribute specification

An attribute name must not appear more than once in the same start-tag or empty-element tag.

## Rationale

> Well-formedness constraint: Unique Att Spec — An attribute name MUST NOT appear more than once in the same start-tag or empty-element tag.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<root id="1" class="main">content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root id="1" id="2">content</root>
```
