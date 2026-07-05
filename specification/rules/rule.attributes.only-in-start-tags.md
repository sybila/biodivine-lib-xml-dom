# Rule: Attributes must only appear in start-tags and empty-element tags

Attribute specifications must not appear outside of start-tags and empty-element tags.

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

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
]>
<root id="r1"/>
<!-- The attribute id="r1" appears in a start-tag, which is valid.
     A violating example would place attributes in a context other than
     start-tags or empty-element tags, which the grammar itself prevents. -->
```
