# Rule: xml:space attribute must have enumerated type with values default and/or preserve

When declared, the xml:space attribute must be given as an enumerated type whose values are one or both of "default" and "preserve".

## Rationale

> When declared, it MUST be given as an enumerated type whose values are one or both of "default" and "preserve". (§2.10)

## Valid Example

```xml
<!ATTLIST poem xml:space (default|preserve) 'preserve'>
```

## Violating Example

```xml
<!ATTLIST poem xml:space CDATA #IMPLIED>
```
