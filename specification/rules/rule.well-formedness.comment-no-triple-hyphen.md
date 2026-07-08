# Rule: Comments Must Not End With Triple Hyphen

An XML comment must not end with "---&gt;" (a hyphen immediately before the closing "--&gt;").

## Rationale

> §2.5: Note that the grammar does not allow a comment ending in --->. [15] Comment ::= '&lt;!--' ((Char - '-') | ('-' (Char - '-')))* '--&gt;'

## Valid Example

```xml
<root><!-- Valid comment --></root>
```

## Violating Example

```xml
<root><!-- Invalid---&gt;</root>
```