# Rule: UTF-8 and UTF-16 terms exclude related encodings

The terms 'UTF-8' and 'UTF-16' in the XML specification do not apply to related character encodings, including but not limited to UTF-16BE, UTF-16LE, or CESU-8.

## Rationale

> The terms 'UTF-8' and 'UTF-16' in this specification do not apply to related character encodings, including but not limited to UTF-16BE, UTF-16LE, or CESU-8. (§4.3.3)

## Note

> Processors that support UTF-16BE or UTF-16LE must do so through explicit encoding declarations or external protocol information, not through the general UTF-16 requirements of the specification.
