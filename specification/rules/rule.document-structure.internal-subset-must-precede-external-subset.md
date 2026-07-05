# Rule: Internal subset must precede external subset

If both the external and internal subsets are used, the internal subset must be considered to occur before the external subset, so that declarations in the internal subset take precedence.

## Rationale

> If both the external and internal subsets are used, the internal subset MUST be considered to occur before the external subset. This has the effect that entity and attribute-list declarations in the internal subset take precedence over those in the external subset. (§2.8)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "external.dtd" [
  <!ENTITY name "Internal">
]>
<root>&name;</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "external.dtd" [
  <!ENTITY name "Internal">
]>
<root>&name;</root>
```
