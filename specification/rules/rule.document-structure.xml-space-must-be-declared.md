# Rule: xml:space attribute must be declared in valid documents

In valid documents, the xml:space attribute must be declared if it is used.

## Rationale

> In valid documents, this attribute, like any other, MUST be declared if it is used. (§2.10)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ATTLIST root xml:space (default|preserve) 'default'>
]>
<root xml:space="preserve">  hello  </root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root (#PCDATA)>
]>
<root xml:space="preserve">  hello  </root>
```
