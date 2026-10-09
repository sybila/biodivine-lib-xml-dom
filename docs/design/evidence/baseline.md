# Baseline verification — pre-rewrite `master` state

Commit: `76beb74cab4f834b04ac8301b867cd766a064a76` (Merge pull request #1 from sybila/feat/daemontus/initial-implementation)

Toolchain: `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`

All commands were run from the repository root with `source ~/.cargo/env`.

## `cargo build`

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
```

## `cargo test`

```
   Compiling biodivine-lib-xml-dom v0.1.0 (/sandbox/biodivine-lib-xml-dom)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running unittests src/lib.rs (target/debug/deps/biodivine_lib_xml_dom-812216becfe0085a)

running 81 tests
test element::tests::test_add_child_prevents_cycle ... ok
test element::tests::test_add_cdata_validation ... ok
test element::tests::test_add_comment_validation ... ok
test element::tests::test_add_pi_validation ... ok
test element::tests::test_empty_default_namespace_stops_inheritance ... ok
test element::tests::test_add_text_validation ... ok
test element::tests::test_get_namespace_prefixed_still_inherits ... ok
test element::tests::test_is_ancestor_deep_tree ... ok
test element::tests::test_is_ancestor_not_related ... ok
test element::tests::test_is_ancestor_same_element ... ok
test element::tests::test_is_ancestor_direct_parent ... ok
test element::tests::test_namespace_redeclaration_rejected ... ok
test element::tests::test_namespace_redeclaration_same_uri_ok ... ok
test element::tests::test_undeclare_does_not_affect_prefixed_ns ... ok
test element::tests::test_undeclare_re_enables_default_ns ... ok
test element::tests::test_undeclare_records_declaration ... ok
test io::tests::test_cdata_creation ... ok
test io::tests::test_comment_creation ... ok
test io::tests::test_duplicate_default_namespace_rejected ... ok
test io::tests::test_cdata_parsing_and_serialization ... ok
test io::tests::test_duplicate_namespace_prefix_rejected ... ok
test io::tests::test_comment_parsing_and_serialization ... ok
test io::tests::test_duplicate_expanded_name_attributes_rejected ... ok
test io::tests::test_mixed_content ... ok
test io::tests::test_mixed_content_with_processing_instructions ... ok
test io::tests::test_empty_default_namespace_roundtrip ... ok
test io::tests::test_prefix_undeclaring_rejected ... ok
test io::tests::test_namespaced_attributes ... ok
test namespace::tests::test_namespace_is_equal_ns ... ok
test qualified_name::tests::test_creation_and_error ... ok
test qualified_name::tests::test_equality_and_ordering ... ok
test io::tests::test_parse_with_namespaces ... ok
test io::tests::test_empty_default_namespace_at_root ... ok
test io::tests::test_mixed_content_with_cdata ... ok
test io::tests::test_processing_instruction_creation ... ok
test io::tests::test_write_created_document ... ok
test io::tests::test_parse_and_write_simple_xml ... ok
test qualified_name::tests::test_resolve_with_map_prefixed ... ok
test namespace::tests::test_unicode_prefixes ... ok
test io::tests::test_scoped_namespaces ... ok
test io::tests::test_namespaced_attributes_on_parent ... ok
test qualified_name::tests::test_hashing_semantic_equality ... ok
test qualified_name::tests::test_resolve_attribute_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_undefined_prefix ... ok
test qualified_name::tests::test_resolve_no_prefix ... ok
test io::tests::test_processing_instruction_parsing_and_serialization ... ok
test namespace::tests::test_namespace_support ... ok
test namespace::tests::test_namespace_equality ... ok
test qualified_name::tests::test_ord_consistent_with_partial_eq ... ok
test io::tests::test_empty_default_namespace_removes_scope ... ok
test qualified_name::tests::test_qualified_name_string_no_prefix_ns ... ok
test qualified_name::tests::test_resolve_attribute_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_ignores_default_ns ... ok
test qualified_name::tests::test_resolve_attribute_with_map_prefixed ... ok
test qualified_name::tests::test_resolve_with_map_undefined_prefix ... ok
test qualified_name::tests::test_resolve_attribute_with_prefix ... ok
test qualified_name::tests::test_resolve_with_map_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_with_parent_ns ... ok
test qualified_name::tests::test_resolve_xml_prefix_auto ... ok
test qualified_name::tests::test_resolve_xmlns_prefix_rejected ... ok
test tests::test_add_children ... ok
test tests::test_create_document ... ok
test tests::test_create_element ... ok
test tests::test_namespace_declaration ... ok
test qualified_name::tests::test_resolve_with_prefix ... ok
test tests::test_qualified_name_resolution ... ok
test xml_spec::tests::test_cdata_wrapper ... ok
test xml_spec::tests::test_comment_wrapper ... ok
test xml_spec::tests::test_namespace_validation ... ok
test xml_spec::tests::test_ncname_helper ... ok
test xml_spec::tests::test_pi_data_wrapper ... ok
test xml_spec::tests::test_pi_target_wrapper ... ok
test xml_spec::tests::test_prefix_ncname_validation ... ok
test xml_spec::tests::test_reserved_xmlns_prefix ... ok
test tests::test_document_reference ... ok
test xml_spec::tests::test_split_qname_invalid ... ok
test xml_spec::tests::test_split_qname_valid ... ok
test xml_spec::tests::test_text_wrapper ... ok
test xml_spec::tests::test_uri_comparison_case_sensitive ... ok
test xml_spec::tests::test_reserved_xml_prefix ... ok
test xml_spec::tests::test_validate_resolved_prefix ... ok

test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/biodivine_lib_xml_dom-b8ff0070d0b0dca3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests biodivine_lib_xml_dom

running 25 tests
test src/lib.rs - (line 41) ... ok
test src/lib.rs - (line 73) ... ok
test src/namespace.rs - namespace::Namespace::prefix (line 131) ... ok
test src/lib.rs - (line 59) ... ok
test src/namespace.rs - namespace::Namespace::new (line 55) ... ok
test src/namespace.rs - namespace::Namespace::prefixed (line 102) ... ok
test src/lib.rs - (line 94) ... ok
test src/namespace.rs - namespace::Namespace::prefix_str (line 148) ... ok
test src/namespace.rs - namespace::Namespace::without_prefix (line 84) ... ok
test src/namespace.rs - namespace::Namespace::uri (line 117) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute (line 184) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element_with_namespace_map (line 299) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_attribute_with_namespace_map (line 327) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName (line 43) ... ok
test src/namespace.rs - namespace::Namespace::is_equal_ns (line 164) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::with_namespace (line 105) ... ok
test src/xml_spec.rs - xml_spec::Comment (line 301) ... ok
test src/xml_spec.rs - xml_spec::NCName::as_str (line 69) ... ok
test src/xml_spec.rs - xml_spec::NCName (line 44) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::without_namespace (line 87) ... ok
test src/qualified_name.rs - qualified_name::QualifiedName::resolve_element (line 153) ... ok
test src/xml_spec.rs - xml_spec::CData (line 235) ... ok
test src/xml_spec.rs - xml_spec::PiData (line 447) ... ok
test src/xml_spec.rs - xml_spec::Text (line 157) ... ok
test src/xml_spec.rs - xml_spec::PiTarget (line 374) ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.12s; merged doctests compilation took 0.12s
```

## `cargo clippy --all-targets`

```
warning: variables can be used directly in the `format!` string
   --> src/io.rs:825:9
    |
825 |         println!("{}", output);
    |         ^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args
    = note: requested on the command line with `-W clippy::uninlined-format-args`
help: change this to
    |
825 -         println!("{}", output);
825 +         println!("{output}");
    |

warning: variables can be used directly in the `format!` string
   --> src/io.rs:888:9
    |
888 |         println!("Generated XML: {}", output);
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args
help: change this to
    |
888 -         println!("Generated XML: {}", output);
888 +         println!("Generated XML: {output}");
    |

warning: variables can be used directly in the `format!` string
    --> src/io.rs:1047:9
     |
1047 | /         assert!(
1048 | |             err.contains("may not be undeclared"),
1049 | |             "Error should mention undeclared prefix, got: {}",
1050 | |             err
1051 | |         );
     | |_________^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args

warning: variables can be used directly in the `format!` string
    --> src/io.rs:1065:9
     |
1065 | /         assert!(
1066 | |             output.contains("xmlns=\"\""),
1067 | |             "Empty ns declaration should be serialized, got: {}",
1068 | |             output
1069 | |         );
     | |_________^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args

warning: variables can be used directly in the `format!` string
    --> src/io.rs:1104:9
     |
1104 | /         assert!(
1105 | |             err.contains("duplicated"),
1106 | |             "Error should mention duplicate, got: {}",
1107 | |             err
1108 | |         );
     | |_________^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args

warning: variables can be used directly in the `format!` string
    --> src/io.rs:1123:9
     |
1123 | /         assert!(
1124 | |             err.contains("duplicated"),
1125 | |             "Error should mention duplicate, got: {}",
1126 | |             err
1127 | |         );
     | |_________^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args

warning: variables can be used directly in the `format!` string
    --> src/io.rs:1141:9
     |
1141 | /         assert!(
1142 | |             err.contains("Duplicate attribute"),
1143 | |             "Error should mention duplicate attribute, got: {}",
1144 | |             err
1145 | |         );
     | |_________^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args

warning: variables can be used directly in the `format!` string
   --> src/xml_spec.rs:725:25
    |
725 |         let rule_path = format!("specification/rules/{}", rule_file);
    |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args
help: change this to
    |
725 -         let rule_path = format!("specification/rules/{}", rule_file);
725 +         let rule_path = format!("specification/rules/{rule_file}");
    |

warning: variables can be used directly in the `format!` string
   --> src/xml_spec.rs:726:9
    |
726 | /         assert!(
727 | |             std::path::Path::new(&rule_path).exists(),
728 | |             "Rule file {} does not exist",
729 | |             rule_file
730 | |         );
    | |_________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#uninlined_format_args

warning: `biodivine-lib-xml-dom` (lib test) generated 9 warnings (run `cargo clippy --fix --lib -p biodivine-lib-xml-dom --tests -- ` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
```

## `cargo fmt --check`

```
```

## `cargo doc --no-deps`

```
 Documenting biodivine-lib-xml-dom v0.1.0 (/sandbox/biodivine-lib-xml-dom)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
   Generated /sandbox/biodivine-lib-xml-dom/target/doc/biodivine_lib_xml_dom/index.html
```

