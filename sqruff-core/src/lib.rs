pub mod rules;
pub mod formatter;
pub mod config;

pub use rules::{Rule, Violation, Severity};
pub use formatter::Formatter;
pub use config::Config;

use sqruff_parser::Parser;

/// Lint a SQL source string and return all violations.
pub fn lint(source: &str, config: &Config) -> Result<Vec<Violation>, String> {
    let dialect = config.dialect();
    let file = Parser::parse_str(source, dialect).map_err(|e| e.to_string())?;
    let active_rules = config.active_rules();
    let mut violations = Vec::new();
    for rule in &active_rules {
        violations.extend(rule.check(&file));
    }
    violations.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));
    Ok(violations)
}

/// Format a SQL source string.
pub fn format(source: &str, config: &Config) -> Result<String, String> {
    let dialect = config.dialect();
    let file = Parser::parse_str(source, dialect).map_err(|e| e.to_string())?;
    let fmt = Formatter::new(config);
    Ok(fmt.format(&file))
}
