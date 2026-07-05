# Rule: PI Content Must Not Contain Question Mark Greater-Than

The content of a processing instruction must not contain the string "?>" before the actual closing "?>" of the PI.

## Rationale

> §2.6: [16] PI ::= '&lt;?' PITarget (S (Char* - (Char* '?&gt;' Char*)))? '?&gt;'

## Valid Example

```xml
<?target some data here?>
```

## Violating Example

```xml
<?target data ?> more?>
```