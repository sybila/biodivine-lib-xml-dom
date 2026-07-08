# Rule: Every start-tag must have a corresponding end-tag

The end of every element that begins with a start-tag must be marked by an end-tag.

## Rationale

> The end of every element that begins with a start-tag MUST be marked by an end-tag containing a name that echoes the element's type as given in the start-tag.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>content
```
