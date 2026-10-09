# Rule: External general parsed entities must match extParsedEnt production

An external general parsed entity is well-formed if it matches the production labeled extParsedEnt (TextDecl? content).

## Rationale

> An external general parsed entity is well-formed if it matches the production labeled extParsedEnt. (§4.3.2)

## Valid Example

```xml
<?xml encoding='UTF-8'?>
<root>content</root>
```

## Violating Example

```xml
<?xml encoding='UTF-8'?>
<root>unclosed
```
