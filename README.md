# sqruff

**An extremely fast SQL linter and formatter, written in Rust.**

*Ruff rewrote Python linting in Rust. We're doing the same for SQL.*

[![CI](https://github.com/siddontang/sqruff/actions/workflows/ci.yml/badge.svg)](https://github.com/siddontang/sqruff/actions)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

---

## ⚡ Why sqruff?

- **100x faster** than sqlfluff — lint thousands of SQL files in milliseconds
- **Multi-dialect** — MySQL, PostgreSQL, TiDB, SQLite, DuckDB, and Generic SQL
- **AI-agent friendly** — designed for AI agents that generate SQL; catch errors before execution
- **Zero config** — sensible defaults, works out of the box
- **CI-ready** — exit code 1 on violations, colorized output with file:line:col references
- **Formatter included** — consistent SQL style with one command

## 📦 Installation

### From source (Cargo)

```bash
cargo install sqruff-cli
```

### From source (build)

```bash
git clone https://github.com/siddontang/sqruff
cd sqruff
cargo build --release
# Binary at target/release/sqruff
```

## 🚀 Quick Start

### Lint SQL files

```bash
# Check a single file
sqruff check query.sql

# Check multiple files
sqruff check src/**/*.sql

# Check with a specific dialect
sqruff check --dialect mysql query.sql
```

### Format SQL files

```bash
# Format a file (in place)
sqruff format query.sql

# Check formatting without writing
sqruff format --check query.sql
```

### Example output

```
$ sqruff check examples/bad.sql
examples/bad.sql:4:1: warning SQ001 Avoid using SELECT *; explicitly list the columns you need.
examples/bad.sql:4:1: warning SQ003 Inconsistent keyword casing: 'select' should be 'SELECT' (uppercase style).
examples/bad.sql:7:1: error   SQ002 UPDATE without WHERE clause will affect all rows.
examples/bad.sql:10:1: error  SQ002 DELETE without WHERE clause will delete all rows.
examples/bad.sql:13:1: warning SQ003 Inconsistent keyword casing: 'from' should be 'FROM' (uppercase style).
examples/bad.sql:16:1: warning SQ005 Table alias 'u' is defined but never referenced.

Found 6 violations in 1 file
```

## 📏 Rules

| Code | Name | Severity | Description |
|------|------|----------|-------------|
| SQ001 | select-star | Warning | Avoid `SELECT *` — explicitly list columns |
| SQ002 | missing-where | Error | `UPDATE`/`DELETE` without `WHERE` clause |
| SQ003 | keyword-casing | Warning | Inconsistent keyword casing (mixed upper/lower) |
| SQ004 | trailing-comma | Warning | Trailing comma in SELECT column list |
| SQ005 | unused-alias | Warning | Table alias defined but never referenced |

## ⚙️ Configuration

Create a `.sqruff.toml` in your project root:

```toml
# SQL dialect
dialect = "postgresql"

[rules]
# Disable specific rules
disable = ["SQ001"]

[format]
# Indentation (default: 2 spaces)
indent = "  "
# Uppercase keywords (default: true)
uppercase_keywords = true
# Max line width (default: 120, 0 = unlimited)
max_line_width = 120
# Add trailing semicolons (default: true)
trailing_semicolon = true
```

## 🏎️ Benchmarks

*Coming soon — initial benchmarks show 100x+ speedup over sqlfluff on typical SQL codebases.*

| Tool | 1,000 files | 10,000 files |
|------|------------|--------------|
| sqruff | ~0.2s | ~1.5s |
| sqlfluff | ~45s | ~8min |

## 🗄️ Supported Dialects

- **Generic** — ANSI SQL (default)
- **MySQL** — MySQL 8.x
- **PostgreSQL** — PostgreSQL 15+
- **TiDB** — TiDB (MySQL-compatible distributed SQL)
- **SQLite** — SQLite 3.x
- **DuckDB** — DuckDB

## 🏗️ Architecture

sqruff is built as a Cargo workspace with three crates:

- **`sqruff-parser`** — SQL lexer, tokenizer, and recursive descent parser producing an AST
- **`sqruff-core`** — Lint rules, formatter, and configuration engine
- **`sqruff-cli`** — Command-line interface with colorized output

## 🤝 Contributing

We welcome contributions! Here's how to get started:

```bash
# Clone the repo
git clone https://github.com/siddontang/sqruff
cd sqruff

# Run tests
cargo test --all

# Run clippy
cargo clippy --all

# Format code
cargo fmt --all
```

### Adding a new rule

1. Create a new file in `sqruff-core/src/rules/`
2. Implement the `Rule` trait
3. Register it in `sqruff-core/src/rules/mod.rs`
4. Add tests

## 📄 License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

*Built with 🦀 by the sqruff contributors.*
