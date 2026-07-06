## Additional context

 - The project is still in a very experimental, unreleased phase. We are free to make breaking changes if necessary.
 - In the `specification` folder, you can find the HTML specification of XML and XML namespaces. Whenever you are making decisions that can impact document semantics, you should consult these specifications to make sure the code follows the specification.
 - We are currently only targeting XML 1.0 with UTF-8 encoding. We are not doing any DOCTYPE validation. Documents with doctype are still read, but the doctype rules are not checked.

## Additional instructions

 - After each non-trivial code change, make sure to run `cargo fmt`. Then check for any issues reported by `cargo clippy` and `cargo test`.
