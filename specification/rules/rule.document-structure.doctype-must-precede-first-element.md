# Rule: Document type declaration must appear before the first element

If a document type declaration is present, it must appear before the first element in the document.

## Rationale

> The document type declaration MUST appear before the first element in the document. (§2.8)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "example.dtd">
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>content</root>
<!DOCTYPE root SYSTEM "example.dtd">
```
