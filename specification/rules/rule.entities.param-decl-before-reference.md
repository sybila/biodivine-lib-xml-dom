# Rule: Parameter entity declaration must precede references

The declaration of a parameter entity must precede any reference to it.

## Rationale

> The declaration of a parameter entity MUST precede any reference to it. (§4.1, [VC: Entity Declared])

## Valid Example

```xml
<!DOCTYPE root [
  <!ENTITY % pe "hello">
  <!ELEMENT root (%pe;)*>
]>
```

## Violating Example

```xml
<!DOCTYPE root [
  <!ELEMENT root (%pe;)*>
  <!ENTITY % pe "hello">
]>
```
