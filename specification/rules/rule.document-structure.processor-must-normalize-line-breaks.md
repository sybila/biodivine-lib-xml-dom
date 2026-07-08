# Rule: XML processor must normalize line breaks to #xA

An XML processor must normalize all line breaks in external parsed entities on input by translating both the two-character sequence #xD #xA and any standalone #xD not followed by #xA to a single #xA character.

## Rationale

> the XML processor MUST behave as if it normalized all line breaks in external parsed entities (including the document entity) on input, before parsing, by translating both the two-character sequence #xD #xA and any #xD that is not followed by #xA to a single #xA character. (§2.11)

## Valid Example

```xml
<?xml version="1.0"?>
<root>line one
line two</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!-- An implementation violation cannot be shown by this document alone.
     If the source entity contains a literal carriage return between the
     two text lines, the processor must report it to the application as #xA. -->
<root>line one
line two</root>
```
