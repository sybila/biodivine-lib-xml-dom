# Rule: Notation names must be unique

A given Name must not be declared in more than one notation declaration.

## Rationale

> [VC: Unique Notation Name] A given Name MUST NOT be declared in more than one notation declaration. (§4.7)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!NOTATION png SYSTEM "image/png">
]>
<root/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!NOTATION gif SYSTEM "image/gif">
  <!NOTATION gif SYSTEM "image/gif-alt">
]>
<root/>
```
