# Rule: Non-UTF-8/UTF-16 entities must have text declaration with encoding

In the absence of external character encoding information, parsed entities stored in an encoding other than UTF-8 or UTF-16 must begin with a text declaration containing an encoding declaration.

## Rationale

> In the absence of external character encoding information (such as MIME headers), parsed entities which are stored in an encoding other than UTF-8 or UTF-16 MUST begin with a text declaration containing an encoding declaration. (§4.3.3)

## Valid Example

```xml
<?xml encoding='ISO-8859-1'?>
<root>content</root>
```

## Violating Example

```xml
<root>ISO-8859-1 content without encoding declaration</root>
```
