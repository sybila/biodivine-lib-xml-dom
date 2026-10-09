# Rule: CDATA attribute whitespace normalization preserves character references

During attribute-value normalization, literal whitespace characters (#x20, #xD, #xA, #x9) are replaced with a space character (#x20). However, character references to whitespace characters (e.g., `&#xA;`, `&#x9;`) are NOT replaced — the actual character is appended to the normalized value. This applies to CDATA and other non-CDATA attribute types.

## Rationale

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

> The normalized value of `label` is `hello` + TAB + `world` (tab character preserved).

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
  <!ATTLIST root label CDATA #IMPLIED>
]>
<root label="hello&#x9;world"/>
<!-- A processor that replaces the tab character (from &#x9;)
     with a space (#x20) would produce "hello world" instead
     of "hello<TAB>world", violating this constraint. -->
```
