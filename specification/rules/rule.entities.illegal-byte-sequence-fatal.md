# Rule: Illegal byte sequences in declared encoding is a fatal error

It is a fatal error if an XML entity is determined to be in a certain encoding but contains byte sequences that are not legal in that encoding. Specifically, it is a fatal error if an entity encoded in UTF-8 contains any ill-formed code unit sequences.

## Rationale

> It is a fatal error if an XML entity is determined (via default, encoding declaration, or higher-level protocol) to be in a certain encoding but contains byte sequences that are not legal in that encoding. Specifically, it is a fatal error if an entity encoded in UTF-8 contains any ill-formed code unit sequences, as defined in section 3.9 of Unicode. (§4.3.3)

## Valid Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>Valid UTF-8 content</root>
```

## Violating Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>Contains invalid UTF-8 byte sequence 0xFF</root>
```
