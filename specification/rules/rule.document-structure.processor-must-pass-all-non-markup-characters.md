# Rule: XML processor must pass all non-markup characters to the application

An XML processor must always pass all characters in a document that are not markup through to the application.

## Rationale

> An XML processor MUST always pass all characters in a document that are not markup through to the application. (§2.10)

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
