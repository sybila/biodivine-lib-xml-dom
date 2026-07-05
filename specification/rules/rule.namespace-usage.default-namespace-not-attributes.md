# Rule: Default namespace does not apply to attributes

Default namespace declarations do not apply to attribute names; an unprefixed attribute name always has no namespace, regardless of any default namespace declaration in scope.

## Rationale

> §6.2 Namespace Defaulting: "Default namespace declarations do not apply directly to attribute names; the interpretation of unprefixed attributes is determined by the element on which they appear."

> §6.2 Namespace Defaulting: "The namespace name for an unprefixed attribute name always has no value."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns="http://example.org">
  <item id="123"/>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xmlns="http://example.org">
  <item><!-- expecting id to be in http://example.org is wrong -->
    <!-- unprefixed attributes never have a namespace -->
  </item>
</root>
```
