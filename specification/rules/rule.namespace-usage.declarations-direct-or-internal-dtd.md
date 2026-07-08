# Rule: Namespace declarations must be directly provided or in internal DTD subset

For correct operation with non-validating processors, namespace declarations must be provided either directly in the document or via default attributes declared in the internal subset of the DTD.

## Rationale

> §5 Using Qualified Names: "If correct operation with such applications is required, namespace declarations MUST be provided either directly or via default attributes declared in the internal subset of the DTD."

## Valid Example

```xml
<?xml version="1.0"?>
<root xmlns:ex="http://example.org">
  <ex:item>value</ex:item>
</root>
```

## External-Subset-Only Example

```xml
<?xml version="1.0"?>
<!-- Suppose external.dtd provides xmlns:ex as a defaulted attribute.
     A non-validating processor is not required to read that declaration. -->
<!DOCTYPE root SYSTEM "external.dtd">
<root>
  <ex:item>value</ex:item>
</root>
```

> This can be namespace-well-formed for processors that read the external
> subset, but it does not satisfy the spec's requirement for correct operation
> with non-validating processors. In that case, the namespace declaration should
> be direct or defaulted from the internal subset.
