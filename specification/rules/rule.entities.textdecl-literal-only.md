# Rule: Text declaration must be provided literally

The text declaration must be provided literally, not by reference to a parsed entity.

## Rationale

> The text declaration MUST be provided literally, not by reference to a parsed entity. (§4.3.1)

## Valid Example

```xml
<?xml encoding='UTF-8'?>
<root>content</root>
```

## Violating Example

```xml
<!ENTITY % textdecl "<?xml encoding='UTF-8'?>">
%textdecl;
<root>content</root>
```
