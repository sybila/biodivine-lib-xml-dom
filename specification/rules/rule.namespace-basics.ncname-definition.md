# Rule: NCName is Name minus colons

NCName is defined as a Name minus any Name containing a colon. This is used throughout the Namespaces specification for prefixes, local parts, entity names, PI targets, notation names, and ID/IDREF attribute values.

## Rationale

> §7 Conformance of Documents: "All other tokens in the document which are REQUIRED, for XML 1.0 well-formedness, to match the XML production for Name MUST match this specification's production for NCName."

> NCName ::= Name - (Char * ':' * Char)

## Note

> The NCName production is foundational to the Namespaces specification. It ensures that colons are used exclusively as the prefix/local-part separator in qualified names, and never appear in entity names, PI targets, notation names, or typed attribute values.
