# Rule: Unparsed entity references are forbidden outside EntityValue

The appearance of a reference to an unparsed entity, except in the EntityValue in an entity declaration, is forbidden and constitutes a fatal error.

## Rationale

> The following are forbidden, and constitute fatal errors: the appearance of a reference to an unparsed entity, except in the EntityValue in an entity declaration. (§4.4.4)

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
]>
<root>&image;</root>
```
