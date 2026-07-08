# Rule: xml:lang attribute values must be BCP 47 language tags or empty string

The values of the xml:lang attribute must be language identifiers as defined by IETF BCP 47, or the empty string.

## Rationale

> The values of the attribute are language identifiers as defined by [IETF BCP 47], Tags for the Identification of Languages; in addition, the empty string may be specified. (§2.12)

## Valid Example

```xml
<?xml version="1.0"?>
<root xml:lang="en-GB">Hello</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xml:lang="en_US_very_long_invalid">Hello</root>
```
