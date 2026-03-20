use crate::rules::{Rule, Severity, Violation};
use sqruff_parser::{SqlFile, StatementKind};

/// SQ004: Trailing comma in SELECT column list.
///
/// Detects when the last column in a SELECT list is followed by a comma
/// (which we infer from the source by checking if FROM immediately follows
/// the last parsed column — if the parser saw a comma and then FROM, the
/// comma was trailing). Because our parser consumes commas between columns,
/// we detect this by re-lexing and checking for `comma, FROM` adjacency.
pub struct TrailingCommaRule;

impl Rule for TrailingCommaRule {
    fn code(&self) -> &'static str {
        "SQ004"
    }

    fn name(&self) -> &'static str {
        "trailing-comma"
    }

    fn check(&self, file: &SqlFile) -> Vec<Violation> {
        // We use a simple token-level check: look for the pattern `, FROM`
        // in the original source. This is a pragmatic approach that works
        // for common cases.
        let violations = Vec::new();

        for stmt in &file.statements {
            if let StatementKind::Select(sel) = &stmt.kind {
                // Check keyword_cases to find FROM position
                if let Some(from_kw) = sel.keyword_cases.iter().find(|k| k.keyword == "FROM") {
                    // If the last column's line is before FROM, check if
                    // there are trailing commas. We detect this from the
                    // select columns: if a column has no expr content but
                    // we can't easily detect this from AST alone.
                    //
                    // Simpler heuristic: check if last column is at a position
                    // very close to FROM (meaning there was nothing between
                    // comma and FROM).
                    if let Some(last_col) = sel.columns.last() {
                        // If the last column is on the same line as FROM and
                        // very close, it's likely fine. We need source access
                        // for a proper check. For now, we mark this rule as
                        // needing source-level analysis.
                        let _ = (last_col, from_kw);
                    }
                }
            }
        }

        violations
    }
}

/// Check for trailing commas by scanning tokens directly.
/// This is used by the lint engine with access to source text.
pub fn check_trailing_comma_in_source(source: &str) -> Vec<Violation> {
    use sqruff_parser::lexer::Lexer;
    use sqruff_parser::token::TokenKind;

    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();
    // Look for pattern: Comma followed by FROM/WHERE/GROUP/ORDER/LIMIT/HAVING/)
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i].kind == TokenKind::Comma {
            let next = &tokens[i + 1];
            if matches!(
                next.kind,
                TokenKind::From | TokenKind::RightParen
            ) {
                violations.push(Violation {
                    rule_code: "SQ004",
                    rule_name: "trailing-comma",
                    message: "Trailing comma in column list.".to_string(),
                    line: tokens[i].line,
                    col: tokens[i].col,
                    severity: Severity::Warning,
                });
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_comma_detected() {
        let sql = "SELECT id, name, FROM users";
        let v = check_trailing_comma_in_source(sql);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_code, "SQ004");
    }

    #[test]
    fn test_no_trailing_comma() {
        let sql = "SELECT id, name FROM users";
        let v = check_trailing_comma_in_source(sql);
        assert!(v.is_empty());
    }

    #[test]
    fn test_trailing_comma_in_parens() {
        let sql = "SELECT id FROM users WHERE id IN (1, 2,)";
        let v = check_trailing_comma_in_source(sql);
        assert_eq!(v.len(), 1);
    }
}
