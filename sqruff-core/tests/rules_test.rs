//! Fixture-based integration tests for sqruff lint rules.
//!
//! Convention:
//!   tests/fixtures/rules/<rule>/pass_*.sql  → should produce 0 violations for that rule
//!   tests/fixtures/rules/<rule>/fail_*.sql  → should produce ≥1 violation for that rule

use sqruff_core::config::Config;
use sqruff_core::rules::select_star::SelectStarRule;
use sqruff_core::rules::missing_where::MissingWhereRule;
use sqruff_core::rules::keyword_casing::KeywordCasingRule;
use sqruff_core::rules::trailing_comma::check_trailing_comma_in_source;
use sqruff_core::rules::unused_alias::UnusedAliasRule;
use sqruff_core::rules::Rule;
use sqruff_parser::{Dialect, Parser};
use std::fs;
use std::path::Path;

/// Load all .sql files matching a glob-like prefix from a directory.
fn load_fixtures(dir: &str, prefix: &str) -> Vec<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rules")
        .join(dir);
    if !path.exists() {
        panic!("Fixture directory does not exist: {:?}", path);
    }
    let mut results = Vec::new();
    for entry in fs::read_dir(&path).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) && name.ends_with(".sql") {
            let content = fs::read_to_string(entry.path()).unwrap();
            results.push((name, content));
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

/// Run a rule against source and return violation count for that specific rule code.
fn check_rule(rule: &dyn Rule, source: &str) -> usize {
    let file = match Parser::parse_str(source, Dialect::Generic) {
        Ok(f) => f,
        Err(_) => return 0, // parse errors are not rule violations
    };
    rule.check(&file)
        .iter()
        .filter(|v| v.rule_code == rule.code())
        .count()
}

// ── SQ001: SELECT * ──────────────────────────────────

#[test]
fn sq001_pass_fixtures() {
    let rule = SelectStarRule;
    for (name, sql) in load_fixtures("sq001", "pass_") {
        let count = check_rule(&rule, &sql);
        assert_eq!(
            count, 0,
            "SQ001 pass fixture '{}' unexpectedly produced {} violations",
            name, count
        );
    }
}

#[test]
fn sq001_fail_fixtures() {
    let rule = SelectStarRule;
    for (name, sql) in load_fixtures("sq001", "fail_") {
        let count = check_rule(&rule, &sql);
        assert!(
            count > 0,
            "SQ001 fail fixture '{}' should have produced violations but got 0",
            name
        );
    }
}

// ── SQ002: Missing WHERE on UPDATE/DELETE ────────────

#[test]
fn sq002_pass_fixtures() {
    let rule = MissingWhereRule;
    for (name, sql) in load_fixtures("sq002", "pass_") {
        let count = check_rule(&rule, &sql);
        assert_eq!(
            count, 0,
            "SQ002 pass fixture '{}' unexpectedly produced {} violations",
            name, count
        );
    }
}

#[test]
fn sq002_fail_fixtures() {
    let rule = MissingWhereRule;
    for (name, sql) in load_fixtures("sq002", "fail_") {
        let count = check_rule(&rule, &sql);
        assert!(
            count > 0,
            "SQ002 fail fixture '{}' should have produced violations but got 0",
            name
        );
    }
}

// ── SQ003: Keyword casing ────────────────────────────

#[test]
fn sq003_pass_fixtures() {
    let rule = KeywordCasingRule;
    for (name, sql) in load_fixtures("sq003", "pass_") {
        let count = check_rule(&rule, &sql);
        assert_eq!(
            count, 0,
            "SQ003 pass fixture '{}' unexpectedly produced {} violations",
            name, count
        );
    }
}

#[test]
fn sq003_fail_fixtures() {
    let rule = KeywordCasingRule;
    for (name, sql) in load_fixtures("sq003", "fail_") {
        let count = check_rule(&rule, &sql);
        assert!(
            count > 0,
            "SQ003 fail fixture '{}' should have produced violations but got 0",
            name
        );
    }
}

// ── SQ004: Trailing comma ────────────────────────────

#[test]
fn sq004_pass_fixtures() {
    for (name, sql) in load_fixtures("sq004", "pass_") {
        let count = check_trailing_comma_in_source(&sql).len();
        assert_eq!(
            count, 0,
            "SQ004 pass fixture '{}' unexpectedly produced {} violations",
            name, count
        );
    }
}

#[test]
fn sq004_fail_fixtures() {
    for (name, sql) in load_fixtures("sq004", "fail_") {
        let count = check_trailing_comma_in_source(&sql).len();
        assert!(
            count > 0,
            "SQ004 fail fixture '{}' should have produced violations but got 0",
            name
        );
    }
}

// ── SQ005: Unused alias ──────────────────────────────

#[test]
fn sq005_pass_fixtures() {
    let rule = UnusedAliasRule;
    for (name, sql) in load_fixtures("sq005", "pass_") {
        let count = check_rule(&rule, &sql);
        assert_eq!(
            count, 0,
            "SQ005 pass fixture '{}' unexpectedly produced {} violations",
            name, count
        );
    }
}

#[test]
fn sq005_fail_fixtures() {
    let rule = UnusedAliasRule;
    for (name, sql) in load_fixtures("sq005", "fail_") {
        let count = check_rule(&rule, &sql);
        assert!(
            count > 0,
            "SQ005 fail fixture '{}' should have produced violations but got 0",
            name
        );
    }
}

// ── Integration: full lint pipeline ──────────────────

#[test]
fn lint_integration_all_rules_clean() {
    let config = Config::default();
    let sql = "SELECT id, name FROM users WHERE id = 1";
    let violations = sqruff_core::lint(sql, &config).unwrap();
    assert!(violations.is_empty(), "Clean SQL should have no violations: {:?}", violations);
}

#[test]
fn lint_integration_multiple_violations() {
    let config = Config::default();
    let sql = "SELECT * FROM users";
    let violations = sqruff_core::lint(sql, &config).unwrap();
    // Should at minimum detect SELECT *
    assert!(
        violations.iter().any(|v| v.rule_code == "SQ001"),
        "Should detect SELECT *"
    );
}

#[test]
fn lint_integration_disabled_rule() {
    let mut config = Config::default();
    config.rules.disable.push("SQ001".to_string());
    let sql = "SELECT * FROM users";
    let violations = sqruff_core::lint(sql, &config).unwrap();
    assert!(
        !violations.iter().any(|v| v.rule_code == "SQ001"),
        "Disabled rule SQ001 should not produce violations"
    );
}

// ── Edge cases ───────────────────────────────────────

#[test]
fn sq001_select_star_with_count_star_is_ok() {
    // COUNT(*) should NOT trigger SQ001 — it's not SELECT *
    let rule = SelectStarRule;
    let sql = "SELECT COUNT(*), MAX(id) FROM users";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    let violations = rule.check(&file);
    assert!(violations.is_empty(), "COUNT(*) should not trigger SQ001");
}

#[test]
fn sq002_insert_without_where_is_ok() {
    // INSERT never needs WHERE
    let rule = MissingWhereRule;
    let sql = "INSERT INTO users (name) VALUES ('Alice')";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    let violations = rule.check(&file);
    assert!(violations.is_empty());
}

#[test]
fn sq003_single_keyword_no_violation() {
    // A statement with only one tracked keyword can't be inconsistent
    let rule = KeywordCasingRule;
    let sql = "SELECT 1";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    let violations = rule.check(&file);
    assert!(violations.is_empty());
}

#[test]
fn sq005_multiple_joins_all_used() {
    let rule = UnusedAliasRule;
    let sql = "SELECT u.id, o.total, p.name FROM users u INNER JOIN orders o ON u.id = o.user_id LEFT JOIN products p ON o.product_id = p.id";
    let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
    let violations = rule.check(&file);
    assert!(violations.is_empty());
}
