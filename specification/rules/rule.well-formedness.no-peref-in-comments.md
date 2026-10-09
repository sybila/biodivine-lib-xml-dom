# Rule: Parameter Entity References Not Recognized In Comments

Parameter entity references must not be recognized within XML comments.

## Rationale

> §2.5: Parameter entity references MUST NOT be recognized within comments.

## Valid Example

```xml
<!-- fragment: comment with literal percent sign -->
<!-- %entity; is just text here -->
```

## Violating Example

```xml
<!-- fragment: processor must not expand parameter entities in comments -->
<!-- %somePE; should not be expanded -->
```