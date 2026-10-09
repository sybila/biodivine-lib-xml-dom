# Rule: End-tag name must match start-tag name

The Name in an element's end-tag must match the element type given in the start-tag.

## Rationale

> Well-formedness constraint: Element Type Match — The Name in an element's end-tag MUST match the element type in the start-tag.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>content</other>
```
