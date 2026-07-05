# Rule: Unprocessable encoding is a fatal error

It is a fatal error when an XML processor encounters an entity with an encoding that it is unable to process.

## Rationale

> It is a fatal error when an XML processor encounters an entity with an encoding that it is unable to process. (§4.3.3)

## Valid Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0" encoding="unknown-encoding-x"?>
<root>content</root>
```
