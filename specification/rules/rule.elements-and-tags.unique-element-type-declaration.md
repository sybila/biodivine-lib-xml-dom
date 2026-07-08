# Rule: Unique element type declaration

An element type must not be declared more than once.

## Rationale

> Validity constraint: Unique Element Type Declaration — An element type MUST NOT be declared more than once.
> — §3.2 Element Type Declarations

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ELEMENT root ANY>
]>
<root/>
```
