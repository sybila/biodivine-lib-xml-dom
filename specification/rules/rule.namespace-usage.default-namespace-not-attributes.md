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

> The `id` attribute has no namespace, even though the element is in the default namespace.

## Misinterpretation Example

```xml
<?xml version="1.0"?>
<root xmlns="http://example.org">
  <item id="123"/>
</root>
```

> This document is namespace-well-formed. The violation would be an
> implementation or application treating the unprefixed `id` attribute as if it
> were in the default namespace `http://example.org`; by §6.2, it has no
> namespace name.
