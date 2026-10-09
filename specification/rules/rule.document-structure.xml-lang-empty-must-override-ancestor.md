# Rule: Empty xml:lang value must override ancestor language specification

An empty xml:lang value on an element must override any xml:lang specification on an enclosing ancestor, making it as if no language information were available for that element and its descendants.

## Rationale

> In particular, the empty value of xml:lang is used on an element B to override a specification of xml:lang on an enclosing element A, without specifying another language. Within B, it is considered that there is no language information available, just as if xml:lang had not been specified on B or any of its ancestors. (§2.12)

## Valid Example

```xml
<?xml version="1.0"?>
<root xml:lang="en">
  <p>Hello</p>
  <p xml:lang="">No language info here</p>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xml:lang="en">
  <p xml:lang="">Processor still reports English for this element.</p>
</root>
```
