# Rule: PI Target Must Be A Valid Name

A processing instruction target must be a valid XML Name.

## Rationale

> §2.6: The PI begins with a target (PITarget) used to identify the application to which the instruction is directed. [17] PITarget ::= Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))

## Valid Example

```xml
<?my-target some data?>
```

## Violating Example

```xml
<?1invalid-target data?>
```