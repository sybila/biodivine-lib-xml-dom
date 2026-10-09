# Rule: XML processors must recognize predefined entities

All XML processors must recognize the predefined entities amp, lt, gt, apos, and quot whether they are declared or not.

## Rationale

> All XML processors MUST recognize these entities whether they are declared or not. (§4.6)

## Valid Example

```xml
<?xml version="1.0"?>
<root>5 &lt; 10 &amp; 10 &gt; 5</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root>5 < 10 & 10 > 5</root>
```
