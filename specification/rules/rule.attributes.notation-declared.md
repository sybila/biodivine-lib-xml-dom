# Rule: [VC] Notation name in unparsed entity declaration must be declared

The Name in an NDataDecl must match the declared name of a notation.

## Rationale

> [VC: Notation Declared] The Name MUST match the declared name of a notation. (§4.2.2)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!ENTITY image SYSTEM "image.gif" NDATA gif>
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY image SYSTEM "image.gif" NDATA undeclared-notation>
]>
<root/>
```
