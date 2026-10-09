# Rule: Markup must not span entity boundaries

No start-tag, end-tag, empty-element tag, element, comment, processing instruction, character reference, or entity reference can begin in one entity and end in another.

## Rationale

> A consequence of well-formedness in general entities is that the logical and physical structures in an XML document are properly nested; no start-tag, end-tag, empty-element tag, element, comment, processing instruction, character reference, or entity reference can begin in one entity and end in another. (§4.3.2)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY fulltag "<b>bold</b>">
]>
<root>&fulltag;</root>
```

> The entire `<b>bold</b>` element is contained within a single entity, satisfying this rule.

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE root [
  <!ENTITY half "<b">
]>
<root>&half;text</b></root>
```

> The start-tag `<b>` begins in the entity `half`, but the end-tag `</b>` appears outside the entity in the document content, spanning an entity boundary.
