/// Supported SQL dialects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dialect {
    #[default]
    Generic,
    MySQL,
    PostgreSQL,
    TiDB,
    SQLite,
    DuckDB,
}

impl Dialect {
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mysql" => Dialect::MySQL,
            "postgres" | "postgresql" => Dialect::PostgreSQL,
            "tidb" => Dialect::TiDB,
            "sqlite" => Dialect::SQLite,
            "duckdb" => Dialect::DuckDB,
            _ => Dialect::Generic,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Dialect::Generic => "generic",
            Dialect::MySQL => "mysql",
            Dialect::PostgreSQL => "postgresql",
            Dialect::TiDB => "tidb",
            Dialect::SQLite => "sqlite",
            Dialect::DuckDB => "duckdb",
        }
    }
}

impl std::fmt::Display for Dialect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
