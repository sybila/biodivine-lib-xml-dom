# Rule: Processors are not required to check namespace names are URI references

To conform to the Namespaces specification, a processor must report violations of namespace well-formedness, with the exception that it is not REQUIRED to check that namespace names are URI references.

## Rationale

> To conform to this specification, a processor MUST report violations of namespace well-formedness, with the exception that it is not REQUIRED to check that namespace names are URI references. (§8)

## Note

> While namespace names are identified by URI references per §2.1, processors are exempted from validating this constraint. This is because checking URI validity can be complex and is often handled by higher-level protocols.
