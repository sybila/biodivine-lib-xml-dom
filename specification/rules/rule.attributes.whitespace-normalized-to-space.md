# Rule: Whitespace characters in attribute values must be normalized to space

During attribute-value normalization, each literal whitespace character (#x20, #xD, #xA, #x9) must be replaced with a space character (#x20). However, character references to whitespace characters (e.g., `&#xA;`, `&#x9;`) are NOT replaced — the actual character is appended to the normalized value. This distinction is important for CDATA attributes.

## Rationale

> For a white space character (#x20, #xD, #xA, #x9), append a space character (#x20) to the normalized value. (§3.3.3, step 3)

> Note that if the unnormalized attribute value contains a character reference to a white space character other than space (#x20), the normalized value contains the referenced character itself (#xD, #xA or #x9). This contrasts with the case where the unnormalized value contains a white space character (not a reference), which is replaced with a space character (#x20) in the normalized value. (§3.3.3)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="hello&#x9;world"/>
```

> For a CDATA attribute, `&#x9;` (tab character reference) is preserved as the actual tab character in the normalized value, since it is a character reference, not a literal whitespace character.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="hello&#x9;world"/>
<!-- A violating example cannot be expressed in source XML alone.
     An implementation that replaces the tab character (from &#x9;)
     with a space (#x20) would violate this constraint for CDATA
     attributes, since character references to whitespace should
     be preserved as the actual character, not replaced. -->
```
