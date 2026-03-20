# sqruff Benchmarks

## Rust Benchmarks (Criterion)

Run all benchmarks:

```bash
cargo bench
```

Run a specific benchmark:

```bash
cargo bench --bench lexer_bench
cargo bench --bench parser_bench
cargo bench --bench lint_bench
cargo bench --bench format_bench
```

Results are saved in `target/criterion/` with HTML reports.

## Python Comparison Scripts

### Generate Large SQL Files

```bash
cd benchmarks/
python generate_large_sql.py --lines 10000 --output large_test.sql
```

Options:
- `--lines N` — number of SQL statements (default: 10000)
- `--output FILE` — output file path (default: large_test.sql)
- `--seed N` — random seed for reproducibility (default: 42)

### Compare sqruff vs sqlfluff

Prerequisites:
```bash
# Build sqruff in release mode
cargo build --release

# Install sqlfluff
pip install sqlfluff
```

Run the comparison:
```bash
python compare_sqlfluff.py --generate 5000 --iterations 5
```

Options:
- `--sql-file FILE` — use an existing SQL file
- `--generate N` — generate a file with N statements
- `--iterations N` — number of timing iterations (default: 3)

Example output:
```
======================================================================
  sqruff vs sqlfluff Performance Comparison
======================================================================
  SQL file:    /tmp/tmpXXXXXX.sql
  File size:   450.2 KB
  Iterations:  3
======================================================================

  Benchmarking sqruff...
    lint:   0.023s (min=0.021s, max=0.025s)
    format: 0.019s (min=0.018s, max=0.020s)

  Benchmarking sqlfluff...
    lint:   12.456s (min=11.890s, max=13.234s)
    format: 15.789s (min=15.012s, max=16.543s)

  🚀 sqruff is 541.6x faster than sqlfluff for linting
======================================================================
```
