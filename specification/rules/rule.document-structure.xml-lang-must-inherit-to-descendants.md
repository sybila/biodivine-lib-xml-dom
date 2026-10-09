# Rule: xml:lang must inherit to descendant elements unless overridden

The language specified by xml:lang applies to the element where it is specified and to all elements in its content unless overridden with another instance of xml:lang.

## Rationale

> The language specified by xml:lang applies to the element where it is specified (including the values of its attributes), and to all elements in its content unless overridden with another instance of xml:lang. (§2.12)

## Valid Example

```xml
<?xml version="1.0"?>
<root xml:lang="en">
  <p>This is in English.</p>
  <p xml:lang="fr">Ceci est en français.</p>
</root>
```

## Violating Example

```xml
<?xml version="1.0"?>
<root xml:lang="en">
  <p>Processor treats this as having no language.</p>
</root>
```
