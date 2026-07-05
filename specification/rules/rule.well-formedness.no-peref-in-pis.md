# Rule: Parameter Entity References Not Recognized In Processing Instructions

Parameter entity references must not be recognized within processing instructions.

## Rationale

> §2.6: Parameter entity references MUST NOT be recognized within processing instructions.

## Valid Example

```xml
<?target %literal; is just text?>
```

## Violating Example

```xml
<?target %somePE; should not expand?>
```