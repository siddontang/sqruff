use crate::rules::{Rule, Severity, Violation};
use sqruff_parser::{Expr, SqlFile, StatementKind};

/// SQ001: Avoid using `SELECT *` — explicitly list columns.
pub struct SelectStarRule;

impl Rule for SelectStarRule {
    fn code(&self) -> &'static str {
        "SQ001"
    }

    fn name(&self) -> &'static str {
        "select-star"
    }

    fn check(&self, file: &SqlFile) -> Vec<Violation> {
        let mut violations = Vec::new();
        for stmt in &file.statements {
            if let StatementKind::Select(sel) = &stmt.kind {
                for col in &sel.columns {
                    if matches!(col.expr, Expr::Star) {
                        violations.push(Violation {
                            rule_code: self.code(),
                            rule_name: self.name(),
                            message: "Avoid using SELECT *; explicitly list the columns you need."
                                .to_string(),
                            line: col.line,
                            col: col.col,
                            severity: Severity::Warning,
                        });
                    }
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
    fn test_select_star_detected() {
        let sql = "SELECT * FROM users";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = SelectStarRule.check(&file);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_code, "SQ001");
    }

    #[test]
    fn test_select_columns_ok() {
        let sql = "SELECT id, name FROM users";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = SelectStarRule.check(&file);
        assert!(v.is_empty());
    }
}
