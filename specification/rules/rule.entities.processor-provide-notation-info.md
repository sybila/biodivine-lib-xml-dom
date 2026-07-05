# Rule: XML processors must provide notation information to applications

XML processors must provide applications with the name and external identifier(s) of any notation declared and referred to in an attribute value, attribute definition, or entity declaration.

## Rationale

> XML processors MUST provide applications with the name and external identifier(s) of any notation declared and referred to in an attribute value, attribute definition, or entity declaration. (§4.7)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!ENTITY image SYSTEM "image.gif" NDATA gif>
  <!ATTLIST root img ENTITY #IMPLIED>
]>
<root img="image"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!ENTITY image SYSTEM "image.gif" NDATA gif>
  <!ATTLIST root img ENTITY #IMPLIED>
]>
<root img="image"/>
```
