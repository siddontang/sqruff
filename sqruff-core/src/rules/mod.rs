pub mod select_star;
pub mod missing_where;
pub mod keyword_casing;
pub mod trailing_comma;
pub mod unused_alias;

use sqruff_parser::SqlFile;

/// Severity of a lint violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A single lint violation.
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule_code: &'static str,
    pub rule_name: &'static str,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {} {}: {}",
            self.line, self.col, self.severity, self.rule_code, self.message
        )
    }
}

/// Trait for lint rules.
pub trait Rule {
    /// Unique code for this rule (e.g. "SQ001").
    fn code(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// Check the AST and return violations.
    fn check(&self, file: &SqlFile) -> Vec<Violation>;
}

/// Return all built-in rules.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(select_star::SelectStarRule),
        Box::new(missing_where::MissingWhereRule),
        Box::new(keyword_casing::KeywordCasingRule),
        Box::new(trailing_comma::TrailingCommaRule),
        Box::new(unused_alias::UnusedAliasRule),
    ]
}
