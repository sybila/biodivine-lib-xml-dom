# Rule: Text declaration must not appear outside entity beginning

The text declaration must not appear at any position other than the beginning of an external parsed entity.

## Rationale

> The text declaration MUST NOT appear at any position other than the beginning of an external parsed entity. (§4.3.1)

## Valid Example

```xml
<?xml encoding='UTF-8'?>
<root>content</root>
```

## Violating Example

```xml
<root>
<?xml encoding='UTF-8'?>
text
</root>
```
