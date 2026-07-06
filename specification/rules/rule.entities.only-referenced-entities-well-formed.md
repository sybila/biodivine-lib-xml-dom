# Rule: Only referenced entities are required to be well-formed

Only parsed entities that are referenced directly or indirectly within the document are required to be well-formed. Unreferenced entities need not be well-formed.

## Rationale

> Only parsed entities that are referenced directly or indirectly within the document are required to be well-formed. (§4.3.2, note)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY referenced "well-formed">
  <!-- unreferenced entity need not be well-formed -->
]>
<root>&referenced;</root>
```
