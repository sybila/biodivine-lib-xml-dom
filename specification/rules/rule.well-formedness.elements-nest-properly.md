# Rule: Elements Must Nest Properly

All elements must nest properly: if a start-tag is in the content of another element, its corresponding end-tag must also be in the content of the same element.

## Rationale

> For all other elements, if the start-tag is in the content of another element, the end-tag is in the content of the same element. More simply stated, the elements, delimited by start- and end-tags, nest properly within each other. — §2.1

## Valid Example

```xml
<outer>
  <inner>content</inner>
</outer>
```

## Violating Example

```xml
<a><b></a></b>
```
