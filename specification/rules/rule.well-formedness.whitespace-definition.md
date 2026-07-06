# Rule: Whitespace Definition

Whitespace (S) consists of one or more space (#x20), tab (#x9), carriage return (#xD), or line feed (#xA) characters.

## Rationale

> §2.3: S (white space) consists of one or more space (#x20) characters, carriage returns, line feeds, or tabs. [3] S ::= (#x20 | #x9 | #xD | #xA)+

## Valid Example

```xml
<!-- fragment: the following are all valid whitespace -->
<root attr=" value ">content</root>
```

## Violating Example

```xml
<!-- An implementation violation cannot be shown by this document alone.
     If a parser treats the non-breaking space character referenced here
     as XML whitespace (S), it violates the definition of S. -->
<root attr="&#xA0;"/>
```
