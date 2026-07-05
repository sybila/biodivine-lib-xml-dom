# Rule: Document Must Match Production

An XML document must match the production `document ::= prolog element Misc*`.

## Rationale

> §2.1: Taken as a whole, it matches the production labeled document. [1] document ::= prolog element Misc*

## Valid Example

```xml
<?xml version="1.0"?>
<!-- comment -->
<root>content</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>content</root>
<other>content</other>
```