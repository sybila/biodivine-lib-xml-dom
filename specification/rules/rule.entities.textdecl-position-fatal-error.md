# Rule: TextDecl must occur only at beginning of external entity

It is a fatal error for a TextDecl to occur other than at the beginning of an external entity.

## Rationale

> It is a fatal error for a TextDecl to occur other than at the beginning of an external entity. (§4.3.3)

## Valid Example

```xml
<?xml encoding='UTF-8'?>
<root>content</root>
```

## Violating Example

```xml
<root>
<?xml encoding='UTF-8'?>
content
</root>
```
