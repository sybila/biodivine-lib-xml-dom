//! Anchors between the code that enforces an XML rule and the rule summaries shipped in
//! `specification/rules/`.
//!
//! The project convention is that every place where the XML 1.0 / Namespaces 1.0 specifications
//! are enforced names the corresponding rule file in a comment, and that a test then asserts the
//! file really exists. This module owns the second half of that convention, so that the check is
//! identical in `xml_spec`, in the parser and in the serializer.
//!
//! The rule files are documentation, not data: nothing in the library reads them at runtime. They
//! are used by tests to detect the situation where a rule is renamed or deleted while the code
//! (and its comment) still claims to implement it.

use std::path::{Path, PathBuf};

/// Directory (relative to the crate root) that holds the rule summaries.
pub const RULES_DIRECTORY: &str = "specification/rules";

/// The path of the summary file for `rule`, where `rule` is a file name such as
/// `rule.well-formedness.legal-characters.md`.
pub fn rule_summary_path(rule: &str) -> PathBuf {
    Path::new(RULES_DIRECTORY).join(rule)
}

/// Whether a summary file exists for `rule`.
pub fn rule_summary_exists(rule: &str) -> bool {
    rule_summary_path(rule).is_file()
}

/// Asserts that a summary file exists for `rule`.
///
/// This turns "this code implements rule X" from a comment into something the test suite checks, so
/// a renamed or deleted rule file cannot go unnoticed. It is public because the check is just as
/// useful from this crate's own integration tests (which only see the public API) and from
/// downstream crates that want to assert their own rule anchors.
///
/// # Panics
///
/// Panics if `specification/rules/<rule>` does not exist.
pub fn assert_rule_exists(rule: &str) {
    assert!(
        rule_summary_exists(rule),
        "the code claims to implement `{rule}`, but {} does not exist",
        rule_summary_path(rule).display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_directory_is_where_we_think_it_is() {
        assert!(Path::new(RULES_DIRECTORY).is_dir());
        assert!(rule_summary_exists(
            "rule.well-formedness.legal-characters.md"
        ));
        assert!(!rule_summary_exists("rule.this.does-not-exist.md"));
        assert_eq!(
            rule_summary_path("rule.x.y.md"),
            Path::new("specification/rules/rule.x.y.md")
        );
    }
}
