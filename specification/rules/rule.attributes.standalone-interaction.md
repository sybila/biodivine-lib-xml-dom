# Rule: [VC] Standalone declaration must be "no" when external declarations affect attribute values

The standalone document declaration must have the value "no" if any external markup declarations contain declarations of attributes with tokenized types where normalization would produce a different value than the unnormalized value.

## Rationale

> [VC: Standalone Document Declaration] The standalone document declaration MUST have the value "no" if any external markup declarations contain declarations of: attributes with tokenized types, where the attribute appears in the document with a value such that normalization will produce a different value from that which would be produced in the absence of the declaration. (§2.9)

## Valid Example

```xml
<?xml version="1.0" standalone="no"?>
<!DOCTYPE root SYSTEM "external.dtd">
<root attr="a  b"/>
```

> If `external.dtd` declares `attr` as NMTOKENS, the normalized value would collapse spaces, so standalone must be "no".

## Violating Example

```xml
<?xml version="1.0" standalone="yes"?>
<!DOCTYPE root SYSTEM "external.dtd">
<root attr="a  b"/>
```
