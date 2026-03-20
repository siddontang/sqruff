use crate::rules::{Rule, Severity, Violation};
use sqruff_parser::{SqlFile, StatementKind};

/// SQ003: SQL keywords should be consistently cased (all uppercase or all lowercase).
pub struct KeywordCasingRule;

impl Rule for KeywordCasingRule {
    fn code(&self) -> &'static str {
        "SQ003"
    }

    fn name(&self) -> &'static str {
        "keyword-casing"
    }

    fn check(&self, file: &SqlFile) -> Vec<Violation> {
        let mut violations = Vec::new();

        for stmt in &file.statements {
            let cases = match &stmt.kind {
                StatementKind::Select(s) => &s.keyword_cases,
                StatementKind::Insert(s) => &s.keyword_cases,
                StatementKind::Update(s) => &s.keyword_cases,
                StatementKind::Delete(s) => &s.keyword_cases,
                StatementKind::CreateTable(s) => &s.keyword_cases,
            };

            if cases.len() < 2 {
                continue;
            }

            // Determine the dominant style from the first keyword
            let first_is_upper = cases[0].original == cases[0].keyword;

            for kc in cases.iter().skip(1) {
                let is_upper = kc.original == kc.keyword;
                let is_lower = kc.original == kc.keyword.to_lowercase();

                if first_is_upper && !is_upper {
                    violations.push(Violation {
                        rule_code: self.code(),
                        rule_name: self.name(),
                        message: format!(
                            "Inconsistent keyword casing: '{}' should be '{}' (uppercase style).",
                            kc.original, kc.keyword
                        ),
                        line: kc.line,
                        col: kc.col,
                        severity: Severity::Warning,
                    });
                } else if !first_is_upper && !is_lower {
                    violations.push(Violation {
                        rule_code: self.code(),
                        rule_name: self.name(),
                        message: format!(
                            "Inconsistent keyword casing: '{}' should be '{}' (lowercase style).",
                            kc.original,
                            kc.keyword.to_lowercase()
                        ),
                        line: kc.line,
                        col: kc.col,
                        severity: Severity::Warning,
                    });
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqruff_parser::{Dialect, Parser};

    #[test]
    fn test_consistent_uppercase_ok() {
        let sql = "SELECT id FROM users WHERE id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = KeywordCasingRule.check(&file);
        assert!(v.is_empty());
    }

    #[test]
    fn test_consistent_lowercase_ok() {
        let sql = "select id from users where id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = KeywordCasingRule.check(&file);
        assert!(v.is_empty());
    }

    #[test]
    fn test_mixed_casing_detected() {
        let sql = "SELECT id from users WHERE id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = KeywordCasingRule.check(&file);
        assert!(!v.is_empty());
        assert_eq!(v[0].rule_code, "SQ003");
    }
}
