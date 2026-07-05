# Rule: Predefined lt and amp entities must use double escaping

If the entities lt or amp are declared, they must be declared as internal entities whose replacement text is a character reference to the respective character being escaped.

## Rationale

> If the entities lt or amp are declared, they MUST be declared as internal entities whose replacement text is a character reference to the respective character (less-than sign or ampersand) being escaped; the double escaping is REQUIRED for these entities so that references to them produce a well-formed result. (§4.6)

## Valid Example

```xml
<!ENTITY lt "&#60;">
<!ENTITY amp "&#38;">
```

## Violating Example

```xml
<!ENTITY lt "<">
<!ENTITY amp "&">
```
