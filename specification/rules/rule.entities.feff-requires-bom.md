# Rule: U+FEFF at entity start requires BOM when no text declaration is present

If the replacement text of an external entity is to begin with the character U+FEFF, and no text declaration is present, then a Byte Order Mark must be present, whether the entity is encoded in UTF-8 or UTF-16.

## Rationale

> If the replacement text of an external entity is to begin with the character U+FEFF, and no text declaration is present, then a Byte Order Mark MUST be present, whether the entity is encoded in UTF-8 or UTF-16. (§4.3.3)

## Valid Example

```xml
<root>Content starting with U+FEFF, preceded by BOM</root>
```

## Violating Example

```xml
<root>Content starting with U+FEFF, no text declaration, no BOM</root>
```
