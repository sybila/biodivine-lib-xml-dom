# Rule: Public identifier whitespace must be normalized

Before a match is attempted, all strings of white space in the public identifier must be normalized to single space characters (#x20), and leading and trailing white space must be removed.

## Rationale

> Before a match is attempted, all strings of white space in the public identifier MUST be normalized to single space characters (#x20), and leading and trailing white space MUST be removed. (§4.2.2)

## Valid Example

```xml
<!-- Whitespace is normalized to single spaces, trimmed -->
<!ENTITY data PUBLIC "  //Example//Data  Set  " "data.xml">
<!-- Normalized: "//Example//Data Set" -->
```

## Violating Example

```xml
<!-- Processor must NOT compare raw whitespace;
     failing to normalize violates this constraint -->
<!ENTITY data PUBLIC "  //Example//Data  Set  " "data.xml">
<!-- Matching against "//Example//Data  Set" (raw) would be wrong -->
```
