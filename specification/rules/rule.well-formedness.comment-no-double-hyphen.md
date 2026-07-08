# Rule: Comments Must Not Contain Double Hyphen

The string "--" (double-hyphen) must not occur within XML comments.

## Rationale

> §2.5: For compatibility, the string "--" (double-hyphen) MUST NOT occur within comments.

## Valid Example

```xml
<root><!-- This is a valid comment --></root>
```

## Violating Example

```xml
<root><!-- This is -- not valid --></root>
```