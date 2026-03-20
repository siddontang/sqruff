use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqruff_core::config::Config;

fn generate_mixed_sql(num_statements: usize) -> String {
    let mut sql = String::new();
    for i in 0..num_statements {
        match i % 6 {
            // Triggers SQ001 (SELECT *)
            0 => sql.push_str("SELECT * FROM users WHERE id > 0;\n"),
            // Triggers SQ002 (UPDATE without WHERE)
            1 => sql.push_str("UPDATE users SET active = 0;\n"),
            // Triggers SQ003 (mixed casing)
            2 => sql.push_str("SELECT id from users WHERE active = 1;\n"),
            // Clean query
            3 => sql.push_str(&format!(
                "SELECT u.id, u.name FROM users u WHERE u.id = {};\n",
                i
            )),
            // Triggers SQ005 (unused alias)
            4 => sql.push_str("SELECT id, name FROM users u;\n"),
            // Triggers SQ002 (DELETE without WHERE)
            5 => sql.push_str("DELETE FROM temp_data;\n"),
            _ => unreachable!(),
        }
    }
    sql
}

fn bench_lint(c: &mut Criterion) {
    let config = Config::default();
    let mut group = c.benchmark_group("lint");

    for size in [100, 500, 2000] {
        let sql = generate_mixed_sql(size);
        let bytes = sql.len();
        group.bench_with_input(
            BenchmarkId::new("all_rules", format!("{}_stmts_{}KB", size, bytes / 1024)),
            &sql,
            |b, sql| {
                b.iter(|| {
                    // We lint each statement individually since our parser
                    // may fail on intentionally malformed SQL
                    let mut total_violations = 0;
                    for line in sql.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(violations) = sqruff_core::lint(black_box(trimmed), &config) {
                            total_violations += violations.len();
                        }
                    }
                    total_violations
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_lint);
criterion_main!(benches);
