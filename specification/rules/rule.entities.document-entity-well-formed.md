# Rule: Document entity must be well-formed

The document entity is well-formed if it matches the production labeled document.

## Rationale

> The document entity is well-formed if it matches the production labeled document. (§4.3.2)

## Valid Example

```xml
<?xml version="1.0"?>
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<a/>
<b/>
```
