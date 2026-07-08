# Rule: Unparsed entity reference in EntityValue is an error

It is an error for a reference to an unparsed entity to appear in the EntityValue in an entity declaration.

## Rationale

> It is an error for a reference to an unparsed entity to appear in the EntityValue in an entity declaration. (§4.4.9)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!ENTITY image SYSTEM "image.gif" NDATA gif>
  <!ENTITY desc "This is an image">
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!ENTITY image SYSTEM "image.gif" NDATA gif>
  <!ENTITY desc "See &image;">
]>
<root/>
```
