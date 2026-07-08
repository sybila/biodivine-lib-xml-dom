# Rule: No external entity references in attribute values

Attribute values must not contain direct or indirect entity references to external entities.

## Rationale

> Well-formedness constraint: No External Entity References — Attribute values MUST NOT contain direct or indirect entity references to external entities.
> — §3.1 Start-Tags, End-Tags, and Empty-Element Tags

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [<!ENTITY internal "text">]>
<root attr="&internal;">content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [<!ENTITY external SYSTEM "external.txt">]>
<root attr="&external;">content</root>
```
