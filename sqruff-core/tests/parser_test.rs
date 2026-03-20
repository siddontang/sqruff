//! Fixture-based integration tests for the sqruff parser.
//!
//! Every .sql file in tests/fixtures/parser/ should parse without errors.

use sqruff_parser::{Dialect, Parser};
use std::fs;
use std::path::Path;

fn parser_fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/parser")
}

#[test]
fn all_parser_fixtures_parse_successfully() {
    let dir = parser_fixtures_dir();
    let mut count = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "sql").unwrap_or(false) {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let sql = fs::read_to_string(&path).unwrap();
            let result = Parser::parse_str(&sql, Dialect::Generic);
            assert!(
                result.is_ok(),
                "Parser fixture '{}' failed to parse: {:?}",
                name,
                result.err()
            );
            count += 1;
        }
    }
    assert!(count > 0, "No parser fixtures found");
    eprintln!("Parsed {} fixture files successfully", count);
}

#[test]
fn parser_fixture_simple_select_structure() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("simple_select.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::Select(sel) => {
            assert_eq!(sel.columns.len(), 3); // id, name, email
            assert!(sel.from.is_some());
            assert!(sel.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn parser_fixture_joins_structure() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("select_with_joins.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::Select(sel) => {
            let from = sel.from.as_ref().unwrap();
            assert_eq!(from.joins.len(), 3); // INNER JOIN + 2x LEFT JOIN
            assert!(sel.order_by.len() > 0);
            assert!(sel.limit.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn parser_fixture_multiple_statements() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("multiple_statements.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 3); // SELECT + UPDATE + DELETE
}

#[test]
fn parser_fixture_create_table() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("create_table_full.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::CreateTable(ct) => {
            assert_eq!(ct.name, "orders");
            assert!(ct.if_not_exists);
            assert!(ct.columns.len() >= 5);
            assert!(!ct.constraints.is_empty());
        }
        _ => panic!("Expected CREATE TABLE statement"),
    }
}

#[test]
fn parser_fixture_aggregate_group_by() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("aggregate_group_by.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::Select(sel) => {
            assert!(!sel.group_by.is_empty());
            assert!(sel.having.is_some());
            assert!(!sel.order_by.is_empty());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn parser_fixture_distinct() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("select_distinct.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::Select(sel) => {
            assert!(sel.distinct);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn parser_fixture_limit_offset() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("select_with_limit_offset.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::Select(sel) => {
            assert!(sel.limit.is_some());
            assert!(sel.offset.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn parser_fixture_is_null() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("select_is_null.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

#[test]
fn parser_fixture_comments() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("comments_inline.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

#[test]
fn parser_fixture_quoted_identifiers() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("quoted_identifiers.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

#[test]
fn parser_fixture_escaped_strings() {
    let sql = fs::read_to_string(parser_fixtures_dir().join("escaped_strings.sql")).unwrap();
    let file = Parser::parse_str(&sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

// ── Programmatic parser tests (no fixtures) ──────────

#[test]
fn parser_handles_empty_input() {
    let file = Parser::parse_str("", Dialect::Generic).unwrap();
    assert!(file.statements.is_empty());
}

#[test]
fn parser_handles_only_comments() {
    let sql = "-- just a comment\n/* block comment */";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    assert!(file.statements.is_empty());
}

#[test]
fn parser_handles_only_semicolons() {
    let file = Parser::parse_str(";;;", Dialect::Generic).unwrap();
    assert!(file.statements.is_empty());
}

#[test]
fn parser_nested_parens_in_where() {
    let sql = "SELECT id FROM users WHERE (id > 1 AND (name = 'x' OR name = 'y'))";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

#[test]
fn parser_insert_multiple_value_rows() {
    let sql = "INSERT INTO t (a, b) VALUES (1, 2), (3, 4), (5, 6)";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::Insert(ins) => {
            assert_eq!(ins.values.len(), 3);
        }
        _ => panic!("Expected INSERT"),
    }
}

#[test]
fn parser_between_expression() {
    let sql = "SELECT id FROM users WHERE age BETWEEN 18 AND 65";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

#[test]
fn parser_in_list_expression() {
    let sql = "SELECT id FROM users WHERE status IN ('active', 'pending', 'trial')";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

#[test]
fn parser_like_expression() {
    let sql = "SELECT id FROM users WHERE email LIKE '%@example.com'";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    assert_eq!(file.statements.len(), 1);
}

#[test]
fn parser_complex_create_table_with_constraints() {
    let sql = "CREATE TABLE t (
        id INT PRIMARY KEY,
        name VARCHAR(100) NOT NULL UNIQUE,
        ref_id INT,
        CONSTRAINT fk_ref FOREIGN KEY (ref_id) REFERENCES other_table (id)
    )";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    match &file.statements[0].kind {
        sqruff_parser::StatementKind::CreateTable(ct) => {
            assert_eq!(ct.columns.len(), 3);
            assert_eq!(ct.constraints.len(), 1);
        }
        _ => panic!("Expected CREATE TABLE"),
    }
}
