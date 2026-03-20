use crate::rules::{Rule, Severity, Violation};
use sqruff_parser::{SqlFile, StatementKind};

/// SQ002: UPDATE/DELETE without WHERE clause is dangerous.
pub struct MissingWhereRule;

impl Rule for MissingWhereRule {
    fn code(&self) -> &'static str {
        "SQ002"
    }

    fn name(&self) -> &'static str {
        "missing-where"
    }

    fn check(&self, file: &SqlFile) -> Vec<Violation> {
        let mut violations = Vec::new();
        for stmt in &file.statements {
            match &stmt.kind {
                StatementKind::Update(upd) => {
                    if upd.where_clause.is_none() {
                        violations.push(Violation {
                            rule_code: self.code(),
                            rule_name: self.name(),
                            message: "UPDATE without WHERE clause will affect all rows."
                                .to_string(),
                            line: stmt.line,
                            col: stmt.col,
                            severity: Severity::Error,
                        });
                    }
                }
                StatementKind::Delete(del) => {
                    if del.where_clause.is_none() {
                        violations.push(Violation {
                            rule_code: self.code(),
                            rule_name: self.name(),
                            message: "DELETE without WHERE clause will delete all rows."
                                .to_string(),
                            line: stmt.line,
                            col: stmt.col,
                            severity: Severity::Error,
                        });
                    }
                }
                _ => {}
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
    fn test_update_without_where() {
        let sql = "UPDATE users SET name = 'x'";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = MissingWhereRule.check(&file);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_code, "SQ002");
        assert_eq!(v[0].severity, Severity::Error);
    }

    #[test]
    fn test_update_with_where() {
        let sql = "UPDATE users SET name = 'x' WHERE id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = MissingWhereRule.check(&file);
        assert!(v.is_empty());
    }

    #[test]
    fn test_delete_without_where() {
        let sql = "DELETE FROM users";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = MissingWhereRule.check(&file);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn test_delete_with_where() {
        let sql = "DELETE FROM users WHERE id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = MissingWhereRule.check(&file);
        assert!(v.is_empty());
    }
}
