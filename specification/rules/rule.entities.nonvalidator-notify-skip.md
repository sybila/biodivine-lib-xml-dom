# Rule: Non-validating processor must notify when skipping external entity

If a non-validating processor does not include the replacement text of an external entity, it must inform the application that it recognized, but did not read, the entity.

## Rationale

> If a non-validating processor does not include the replacement text, it MUST inform the application that it recognized, but did not read, the entity. (§4.4.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY external SYSTEM "external.xml">
]>
<root>&external;</root>
```

> A non-validating processor that recognizes the entity but does not read its replacement text must still notify the application of this fact, satisfying this rule.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY external SYSTEM "external.xml">
]>
<root>&external;</root>
```

> A non-validating processor that recognizes the entity, skips its replacement text, and fails to notify the application would violate this rule.
