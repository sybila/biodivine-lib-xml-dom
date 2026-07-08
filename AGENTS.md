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