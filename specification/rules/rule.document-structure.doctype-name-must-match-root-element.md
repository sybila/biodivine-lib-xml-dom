# Rule: DOCTYPE name must match root element type

The Name in the document type declaration must match the element type of the root element.

## Rationale

> [VC: Root Element Type] The Name in the document type declaration MUST match the element type of the root element. (§2.8)

## Valid Example

```xml
<?xml version="1.0"?>
<!DOCTYPE greeting SYSTEM "hello.dtd">
<greeting>Hello, world!</greeting>
```

## Violating Example

```xml
<?xml version="1.0"?>
<!DOCTYPE greeting SYSTEM "hello.dtd">
<message>Hello, world!</message>
```
