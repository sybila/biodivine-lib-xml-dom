# Rule: Empty string must not be used as a namespace name

The empty string must not be used as a namespace name, even though it is a legal URI reference. An empty default namespace declaration (`xmlns=""`) is allowed, but it removes the default namespace instead of binding the empty string as a namespace name.

## Rationale

> §2.2 Use of URIs as Namespace Names: "The empty string, though it is a legal URI reference, cannot be used as a namespace name."

## Valid Example

```xml
<root xmlns:ex="http://example.org/ns" xmlns="">
  <ex:item/>
  <item/>
</root>
```

## Violating Example

```xml
<root xmlns:ex="">
  <ex:item/>
</root>
```
