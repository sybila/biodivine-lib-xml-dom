# Rule: External entity references forbidden in attribute values

A reference to an external entity in an attribute value is forbidden and constitutes a fatal error.

## Rationale

> The following are forbidden, and constitute fatal errors: a reference to an external entity in an attribute value. (§4.4.4)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY internal "value">
]>
<root attr="&internal;">text</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY external SYSTEM "external.xml">
]>
<root attr="&external;">text</root>
```
