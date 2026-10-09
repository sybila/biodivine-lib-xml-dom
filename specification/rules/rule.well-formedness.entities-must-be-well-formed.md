# Rule: Referenced Parsed Entities Must Be Well-Formed

Each parsed entity referenced directly or indirectly within an XML document must itself be well-formed.

## Rationale

> Each of the parsed entities which is referenced directly or indirectly within the document is well-formed. — §2.1

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY good "valid text">
]>
<root>&good;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY bad "<unclosed">
]>
<root>&bad;</root>
```
