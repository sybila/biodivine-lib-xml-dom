## Additional context

 - The project is still in a very experimental, unreleased phase. We are free to make breaking changes if necessary.
 - In the `specification` folder, you can find the HTML specification of XML and XML namespaces. Whenever you are making decisions that can impact document semantics, you should consult these specifications to make sure the code follows the specification.
 - We are currently only targeting XML 1.0 with UTF-8 encoding. We are not doing any DOCTYPE validation. Documents with doctype are still read, but the doctype rules are not checked.
 - Code that is primarily implementing XML specification logic should be placed as separate utility functions with unit tests into the `xml_spec` module.
 - When using `codebase-memory-mcp`, you should always start by calling `codebase-memory-mcp_list_projects` tool. Never assume you can guess the name of the project directly.

## Additional instructions

 - After each non-trivial code change, make sure to run `cargo fmt`. Then check for any issues reported by `cargo clippy` and `cargo test`.
 - If a type is referenced in a documentation, it should be referenced in a way that allows `cargo doc` to resolve it as a link. For example, "[`String`]" instead of just "`String`".
 - If a function returns a `Result` with a possible error, the documentation should have an `Errors` section which explains what are the error conditions, and what error type/variant is returned for these conditions.
 - If a function contains an explicit `panic!` or other expression that could panic (and we have not ensured conditions to avoid the panic), the documentation should have a `Panics` section explaining input conditions leading to the panic. 
 - When using `unsafe` in code, there should be a comment explaining why this particular usage is ok.
 - Do not write "`[Type]`s" to mean plural of `[Type]` when writing documentation comments. Use some other phrasing, e.g. "`[Type]` objects".

## Code organization and linting

 - The public API lives in the crate root, with one canonical location per item: a public sub-module is
   only added when a substantial, self-contained subset of the API deserves it (`xml_spec`), and
   internal modules are never named in public documentation.
 - Feature entry points belong on the type they belong to (parsing and serialization are `Document`
   methods, not free functions), so that the API is discoverable from the type the user already has.
 - Internal modules are organized by feature: one file per type or per independent step of an
   algorithm, named after what it does, with the crate using the modern `foo.rs` + `foo/` layout.
 - Integer conversions are explicit: `as` casts that can truncate, wrap or flip a sign are lint
   errors (`cast_possible_truncation`, `cast_possible_wrap`, `cast_sign_loss`), and an infallible
   conversion is written as `From`/`TryFrom` with an expectation explaining why it cannot fail.
 - Iteration order is part of the contract for anything a user can observe (serialization, validation
   reports, error lists), so iterating a hash container is a lint error
   (`iter_over_hash_type`); hash containers are reserved for pure lookup maps, which are never
   iterated.
 - `clippy.toml` pins `msrv` to `[workspace.package] rust-version`, so a lint suggestion never
   recommends an API the declared minimum supported Rust version does not have.