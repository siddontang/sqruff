use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqruff_core::config::Config;

fn generate_format_sql(num_statements: usize) -> String {
    let mut sql = String::new();
    for i in 0..num_statements {
        match i % 4 {
            0 => sql.push_str(&format!(
                "select u.id,u.name,u.email,o.total from users u inner join orders o on u.id=o.user_id where u.active=1 and o.total>{} order by o.total desc limit 50;\n",
                i
            )),
            1 => sql.push_str(&format!(
                "insert into products(name,price,category)values('item_{}',{}.99,'category_{}');\n",
                i, i, i % 10
            )),
            2 => sql.push_str(&format!(
                "update inventory set stock=stock-1,updated_at='2024-06-15' where product_id={} and stock>0;\n",
                i
            )),
            3 => sql.push_str(&format!(
                "delete from audit_log where created_at<'2023-01-01' and level='debug';\n"
            )),
            _ => unreachable!(),
        }
    }
    sql
}

fn bench_format(c: &mut Criterion) {
    let config = Config::default();
    let mut group = c.benchmark_group("format");

    for size in [100, 500, 2000] {
        let sql = generate_format_sql(size);
        let bytes = sql.len();
        group.bench_with_input(
            BenchmarkId::new("format", format!("{}_stmts_{}KB", size, bytes / 1024)),
            &sql,
            |b, sql| {
                b.iter(|| {
                    let mut total_len = 0;
                    for line in sql.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(formatted) = sqruff_core::format(black_box(trimmed), &config) {
                            total_len += formatted.len();
                        }
                    }
                    total_len
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_format);
criterion_main!(benches);
