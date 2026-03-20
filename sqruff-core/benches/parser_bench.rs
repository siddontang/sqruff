use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqruff_parser::{Dialect, Parser};

fn generate_large_sql(num_statements: usize) -> String {
    let mut sql = String::new();
    for i in 0..num_statements {
        match i % 4 {
            0 => sql.push_str(&format!(
                "SELECT u.id, u.name, o.total FROM users u INNER JOIN orders o ON u.id = o.user_id WHERE u.active = 1 AND o.total > {} ORDER BY o.total DESC LIMIT 50;\n",
                i
            )),
            1 => sql.push_str(&format!(
                "INSERT INTO events (user_id, event_type, payload) VALUES ({}, 'click', 'button_{}');\n",
                i, i
            )),
            2 => sql.push_str(&format!(
                "UPDATE inventory SET stock = stock - 1 WHERE product_id = {} AND stock > 0;\n",
                i
            )),
            3 => sql.push_str(&format!(
                "DELETE FROM temp_data WHERE batch_id = {} AND created_at < '2024-01-01';\n",
                i
            )),
            _ => unreachable!(),
        }
    }
    sql
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    for size in [100, 1000, 5000] {
        let sql = generate_large_sql(size);
        let bytes = sql.len();
        group.bench_with_input(
            BenchmarkId::new("parse", format!("{}_stmts_{}KB", size, bytes / 1024)),
            &sql,
            |b, sql| {
                b.iter(|| {
                    let _ = Parser::parse_str(black_box(sql), Dialect::Generic).unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
