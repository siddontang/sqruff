use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqruff_parser::Lexer;

/// Generate a large SQL string with many statements.
fn generate_large_sql(num_statements: usize) -> String {
    let mut sql = String::new();
    for i in 0..num_statements {
        match i % 5 {
            0 => sql.push_str(&format!(
                "SELECT u.id, u.name, u.email, o.total, o.status FROM users u INNER JOIN orders o ON u.id = o.user_id WHERE u.active = 1 AND o.total > {} ORDER BY o.total DESC LIMIT 100;\n",
                i
            )),
            1 => sql.push_str(&format!(
                "INSERT INTO audit_log (user_id, action, details, created_at) VALUES ({}, 'login', 'User logged in from 192.168.1.{}', '2024-06-15');\n",
                i, i % 256
            )),
            2 => sql.push_str(&format!(
                "UPDATE products SET price = price * 1.1, updated_at = '2024-06-15' WHERE category_id = {} AND active = 1;\n",
                i % 50
            )),
            3 => sql.push_str(&format!(
                "DELETE FROM sessions WHERE user_id = {} AND expired_at < '2024-01-01';\n",
                i
            )),
            4 => sql.push_str(&format!(
                "SELECT department_id, COUNT(*) AS cnt, AVG(salary) AS avg_sal FROM employees WHERE hire_date > '2020-01-01' GROUP BY department_id HAVING COUNT(*) > {} ORDER BY avg_sal DESC;\n",
                i % 10
            )),
            _ => unreachable!(),
        }
    }
    sql
}

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    for size in [100, 1000, 5000] {
        let sql = generate_large_sql(size);
        let bytes = sql.len();
        group.bench_with_input(
            BenchmarkId::new("tokenize", format!("{}_stmts_{}KB", size, bytes / 1024)),
            &sql,
            |b, sql| {
                b.iter(|| {
                    let mut lexer = Lexer::new(black_box(sql));
                    let _ = lexer.tokenize().unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
