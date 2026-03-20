use crate::dialect::Dialect;

/// Top-level parsed SQL — a sequence of statements.
#[derive(Debug, Clone)]
pub struct SqlFile {
    pub statements: Vec<Statement>,
    pub dialect: Dialect,
}

/// A single SQL statement.
#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
}

// ── SELECT ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub distinct: bool,
    pub columns: Vec<SelectColumn>,
    pub from: Option<FromClause>,
    pub where_clause: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<OrderByItem>,
    pub limit: Option<Expr>,
    pub offset: Option<Expr>,
    /// Original keyword casing as found in source.
    pub keyword_cases: Vec<KeywordCase>,
}

#[derive(Debug, Clone)]
pub struct SelectColumn {
    pub expr: Expr,
    pub alias: Option<String>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct FromClause {
    pub table: TableRef,
    pub joins: Vec<JoinClause>,
}

#[derive(Debug, Clone)]
pub struct TableRef {
    pub name: String,
    pub alias: Option<String>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: TableRef,
    pub on: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug, Clone)]
pub struct OrderByItem {
    pub expr: Expr,
    pub descending: bool,
}

// ── INSERT ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expr>>,
    pub keyword_cases: Vec<KeywordCase>,
}

// ── UPDATE ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UpdateStatement {
    pub table: String,
    pub assignments: Vec<Assignment>,
    pub where_clause: Option<Expr>,
    pub keyword_cases: Vec<KeywordCase>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub column: String,
    pub value: Expr,
}

// ── DELETE ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DeleteStatement {
    pub table: String,
    pub where_clause: Option<Expr>,
    pub keyword_cases: Vec<KeywordCase>,
}

// ── CREATE TABLE ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CreateTableStatement {
    pub if_not_exists: bool,
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub constraints: Vec<TableConstraint>,
    pub keyword_cases: Vec<KeywordCase>,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: Option<bool>,
    pub default: Option<Expr>,
    pub primary_key: bool,
    pub unique: bool,
    pub auto_increment: bool,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Int,
    BigInt,
    SmallInt,
    Integer,
    Varchar(Option<u32>),
    Char(Option<u32>),
    Text,
    Boolean,
    Float,
    Double,
    Decimal(Option<u32>, Option<u32>),
    Date,
    Timestamp,
    Blob,
    Custom(String),
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Int => write!(f, "INT"),
            DataType::BigInt => write!(f, "BIGINT"),
            DataType::SmallInt => write!(f, "SMALLINT"),
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Varchar(Some(n)) => write!(f, "VARCHAR({})", n),
            DataType::Varchar(None) => write!(f, "VARCHAR"),
            DataType::Char(Some(n)) => write!(f, "CHAR({})", n),
            DataType::Char(None) => write!(f, "CHAR"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Decimal(Some(p), Some(s)) => write!(f, "DECIMAL({}, {})", p, s),
            DataType::Decimal(Some(p), None) => write!(f, "DECIMAL({})", p),
            DataType::Decimal(None, _) => write!(f, "DECIMAL"),
            DataType::Date => write!(f, "DATE"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Blob => write!(f, "BLOB"),
            DataType::Custom(s) => write!(f, "{}", s.to_uppercase()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TableConstraint {
    PrimaryKey(Vec<String>),
    Unique(Vec<String>),
    ForeignKey {
        columns: Vec<String>,
        ref_table: String,
        ref_columns: Vec<String>,
    },
}

// ── Expressions ─────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Star,
    Identifier(String),
    QualifiedIdentifier(String, String), // table.column
    Number(String),
    StringLiteral(String),
    Null,
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    IsNull(Box<Expr>),
    IsNotNull(Box<Expr>),
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
    },
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
    },
    Nested(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Like,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
    Minus,
}

// ── Keyword casing tracking ─────────────────────────

/// Tracks the original casing of a keyword for lint rules.
#[derive(Debug, Clone)]
pub struct KeywordCase {
    pub keyword: String,     // canonical uppercase
    pub original: String,    // as written in source
    pub line: usize,
    pub col: usize,
}
