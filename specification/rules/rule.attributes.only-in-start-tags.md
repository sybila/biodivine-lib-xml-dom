# Rule: Attributes must only appear in start-tags and empty-element tags

Attribute specifications must not appear outside of start-tags and empty-element tags. This is enforced by the XML grammar: the productions for STag and EmptyElemTag define where attribute specifications may appear.

## Rationale

> Attribute specifications MUST NOT appear outside of start-tags and empty-element tags; thus, the productions used to recognize them appear in 3.1 Start-Tags, End-Tags, and Empty-Element Tags. (§3.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root id ID #REQUIRED>
]>
<root id="r1"/>
```

## Note

> This rule is enforced by the XML grammar itself (the STag and EmptyElemTag productions). A violating example cannot be expressed in well-formed XML source, as any attribute specification outside a start-tag or empty-element tag would cause a parse error before this constraint could be evaluated.
