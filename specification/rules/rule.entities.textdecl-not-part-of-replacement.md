# Rule: Text declaration is not part of entity replacement text

The text declaration in an external parsed entity is not considered part of its replacement text.

## Rationale

> The text declaration in an external parsed entity is not considered part of its replacement text. (§4.3.1)

## Note

> The TextDecl (e.g., `<?xml version="1.0" encoding="UTF-8"?>`) at the beginning of an external entity is used for encoding detection but is not included in the entity's replacement text when the entity is referenced.
