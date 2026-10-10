//! Anchors between the code that enforces an XML rule and the rule summaries shipped in
//! `specification/rules/`.
//!
//! The convention this module supports: every place where the XML 1.0 or Namespaces 1.0
//! specifications are enforced names the corresponding rule in a comment (`// rule: <identifier>`)
//! *and* links it from the documentation, and the tests then assert that a summary file exists for
//! every rule the code claims to implement. The documentation side gives a reader the specification
//! text (see [`crate::xml_spec::rule_set`]); this module owns the test side, so the check is
//! identical in `xml_spec`, in the parser and in the serializer.
//!
//! The rule summaries are documentation, not data: nothing in the library reads them at run time.
//! The tests read them to detect the situation where a rule is renamed or deleted while the code
//! (and its comment) still claims to implement it.

use std::path::{Path, PathBuf};

/// Directory (relative to the crate root) that holds the rule summaries.
const RULES_DIRECTORY: &str = "specification/rules";

/// The path of the summary file for the rule `identifier`, which is a rule name such as
/// `rule.well-formedness.legal-characters` (without the `.md` suffix).
fn rule_summary_path(identifier: &str) -> PathBuf {
    Path::new(RULES_DIRECTORY).join(format!("{identifier}.md"))
}

/// Whether a summary file exists for the rule `identifier`.
pub fn rule_summary_exists(identifier: &str) -> bool {
    rule_summary_path(identifier).is_file()
}

/// Asserts that a summary file exists for the rule `identifier`.
///
/// This turns "this code enforces rule X" from a comment into something the test suite checks, so a
/// renamed or deleted rule summary cannot go unnoticed. The check is public because the convention
/// it enforces is checked from this crate's integration tests, which only see the public API, and
/// because a downstream crate that mirrors the convention can reuse it.
///
/// # Panics
///
/// Panics if `specification/rules/<identifier>.md` does not exist.
#[track_caller]
pub fn assert_rule_exists(identifier: &str) {
    assert!(
        rule_summary_exists(identifier),
        "the code claims to implement `{identifier}`, but {} does not exist",
        rule_summary_path(identifier).display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_directory_is_where_we_think_it_is() {
        assert!(Path::new(RULES_DIRECTORY).is_dir());
        assert!(rule_summary_exists("rule.well-formedness.legal-characters"));
        assert!(!rule_summary_exists("rule.this.does-not-exist"));
        assert_eq!(
            rule_summary_path("rule.x.y"),
            Path::new("specification/rules/rule.x.y.md")
        );
    }

    /// The helper rejects an identifier that carries the `.md` suffix instead of accepting it
    /// silently, because the identifier is what the documentation links use.
    #[test]
    fn the_identifier_has_no_md_suffix() {
        assert!(!rule_summary_exists(
            "rule.well-formedness.legal-characters.md"
        ));
    }
}
