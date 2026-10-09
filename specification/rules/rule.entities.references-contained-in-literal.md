# Rule: Entity replacement text references must be contained within literal value

Character, parameter-entity, and general-entity references in an internal entity declaration's literal entity value must be contained entirely within the literal entity value.

## Rationale

> Such references MUST be contained entirely within the literal entity value. (§4.5)

## Valid Example

```xml
<!ENTITY greeting "Hello &#38; Welcome">
```

## Violating Example

```xml
<!ENTITY greeting "Hello &#38
">
```
