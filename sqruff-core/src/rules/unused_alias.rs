use crate::rules::{Rule, Severity, Violation};
use sqruff_parser::{Expr, SqlFile, StatementKind};
use std::collections::HashSet;

/// SQ005: Table alias is defined but never referenced.
pub struct UnusedAliasRule;

impl Rule for UnusedAliasRule {
    fn code(&self) -> &'static str {
        "SQ005"
    }

    fn name(&self) -> &'static str {
        "unused-alias"
    }

    fn check(&self, file: &SqlFile) -> Vec<Violation> {
        let mut violations = Vec::new();

        for stmt in &file.statements {
            if let StatementKind::Select(sel) = &stmt.kind {
                if let Some(from) = &sel.from {
                    // Collect all defined table aliases
                    let mut defined_aliases: Vec<(&str, usize, usize)> = Vec::new();

                    if let Some(alias) = &from.table.alias {
                        defined_aliases.push((alias, from.table.line, from.table.col));
                    }

                    for join in &from.joins {
                        if let Some(alias) = &join.table.alias {
                            defined_aliases.push((alias, join.table.line, join.table.col));
                        }
                    }

                    if defined_aliases.is_empty() {
                        continue;
                    }

                    // Collect all referenced identifiers (qualified table refs)
                    let mut referenced: HashSet<String> = HashSet::new();
                    for col in &sel.columns {
                        collect_references(&col.expr, &mut referenced);
                    }
                    if let Some(wc) = &sel.where_clause {
                        collect_references(wc, &mut referenced);
                    }
                    for gb in &sel.group_by {
                        collect_references(gb, &mut referenced);
                    }
                    if let Some(h) = &sel.having {
                        collect_references(h, &mut referenced);
                    }
                    for ob in &sel.order_by {
                        collect_references(&ob.expr, &mut referenced);
                    }
                    // Also check join ON clauses
                    for join in &from.joins {
                        if let Some(on) = &join.on {
                            collect_references(on, &mut referenced);
                        }
                    }

                    for (alias, line, col) in &defined_aliases {
                        if !referenced.contains(*alias) {
                            violations.push(Violation {
                                rule_code: self.code(),
                                rule_name: self.name(),
                                message: format!(
                                    "Table alias '{}' is defined but never referenced.",
                                    alias
                                ),
                                line: *line,
                                col: *col,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        violations
    }
}

fn collect_references(expr: &Expr, refs: &mut HashSet<String>) {
    match expr {
        Expr::QualifiedIdentifier(table, _) => {
            refs.insert(table.clone());
        }
        Expr::BinaryOp { left, right, .. } => {
            collect_references(left, refs);
            collect_references(right, refs);
        }
        Expr::UnaryOp { expr, .. } => {
            collect_references(expr, refs);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_references(arg, refs);
            }
        }
        Expr::IsNull(e) | Expr::IsNotNull(e) | Expr::Nested(e) => {
            collect_references(e, refs);
        }
        Expr::InList { expr, list } => {
            collect_references(expr, refs);
            for item in list {
                collect_references(item, refs);
            }
        }
        Expr::Between { expr, low, high } => {
            collect_references(expr, refs);
            collect_references(low, refs);
            collect_references(high, refs);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqruff_parser::{Dialect, Parser};

    #[test]
    fn test_unused_alias_detected() {
        let sql = "SELECT id, name FROM users u";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = UnusedAliasRule.check(&file);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_code, "SQ005");
    }

    #[test]
    fn test_used_alias_ok() {
        let sql = "SELECT u.id, u.name FROM users u";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = UnusedAliasRule.check(&file);
        assert!(v.is_empty());
    }

    #[test]
    fn test_alias_used_in_join() {
        let sql = "SELECT u.id FROM users u INNER JOIN orders o ON u.id = o.user_id";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = UnusedAliasRule.check(&file);
        assert!(v.is_empty());
    }

    #[test]
    fn test_no_alias_no_violation() {
        let sql = "SELECT id FROM users";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let v = UnusedAliasRule.check(&file);
        assert!(v.is_empty());
    }
}
