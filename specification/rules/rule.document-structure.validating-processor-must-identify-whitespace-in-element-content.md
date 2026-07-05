# Rule: Validating processor must identify whitespace in element content

A validating XML processor must inform the application which characters constitute white space appearing in element content.

## Rationale

> A validating XML processor MUST also inform the application which of these characters constitute white space appearing in element content. (§2.10)

## Valid Example

```xml
<?xml version="1.0"?>
<root>  hello   world  </root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>  hello   world  </root>
```
