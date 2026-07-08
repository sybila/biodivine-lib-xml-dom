# Rule: Interoperability recommendations for parameter entities and EMPTY elements

For interoperability: (1) if a parameter-entity reference appears in a choice, seq, or Mixed construct, its replacement text SHOULD contain at least one non-blank character, and neither the first nor last non-blank character SHOULD be a connector (`|` or `,`). (2) the empty-element tag SHOULD be used, and SHOULD only be used, for elements which are declared EMPTY.

## Rationale

> For interoperability, if a parameter-entity reference appears in a choice, seq, or Mixed construct, its replacement text SHOULD contain at least one non-blank character, and neither the first nor last non-blank character of the replacement text SHOULD be a connector (`|` or `,`). (§3.2.1)

> For interoperability, the empty-element tag SHOULD be used, and SHOULD only be used, for elements which are declared EMPTY. (§3.1)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root EMPTY>
]>
<root/>
```

> An EMPTY element uses the empty-element tag, satisfying the interoperability recommendation.
