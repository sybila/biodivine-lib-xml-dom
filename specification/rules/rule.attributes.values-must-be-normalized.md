# Rule: Attribute values must be normalized before validation or application

Before the value of an attribute is passed to the application or checked for validity, the XML processor must normalize the attribute value.

## Rationale

> Before the value of an attribute is passed to the application or checked for validity, the XML processor MUST normalize the attribute value by applying the algorithm below, or by using some other method such that the value passed to the application is the same as that produced by the algorithm. (§3.3.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="hello world"/>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="hello world"/>
<!-- Violation of normalization is internal to the processor;
     a violating example cannot be expressed in source XML alone.
     An implementation that passes unnormalized values to the
     application would violate this constraint. -->
```
