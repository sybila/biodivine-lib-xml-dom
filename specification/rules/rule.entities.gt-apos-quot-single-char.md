# Rule: Predefined gt, apos, quot entities must use single character replacement

If the entities gt, apos, or quot are declared, they must be declared as internal entities whose replacement text is the single character being escaped (or a character reference to that character).

## Rationale

> If the entities gt, apos, or quot are declared, they MUST be declared as internal entities whose replacement text is the single character being escaped (or a character reference to that character; the double escaping here is OPTIONAL but harmless). (§4.6)

## Valid Example

```xml
<!ENTITY gt ">">
<!ENTITY apos "'">
<!ENTITY quot """">
```

## Violating Example

```xml
<!ENTITY gt "greater-than">
```
