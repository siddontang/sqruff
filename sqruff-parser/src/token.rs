/// Token kinds produced by the SQL lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Select,
    From,
    Where,
    And,
    Or,
    Not,
    As,
    On,
    Join,
    Inner,
    Left,
    Right,
    Full,
    Outer,
    Cross,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    Drop,
    Alter,
    Index,
    If,
    Exists,
    Primary,
    Key,
    Unique,
    Default,
    Null,
    In,
    Between,
    Like,
    Is,
    Order,
    By,
    Asc,
    Desc,
    Group,
    Having,
    Limit,
    Offset,
    Union,
    All,
    Distinct,
    Case,
    When,
    Then,
    Else,
    End,
    Int,
    Integer,
    Varchar,
    Text,
    Boolean,
    Float,
    Double,
    Decimal,
    Date,
    Timestamp,
    Blob,
    BigInt,
    SmallInt,
    Char,
    AutoIncrement,
    NotNull,
    References,
    Foreign,
    Constraint,
    Check,
    Count,
    Sum,
    Avg,
    Min,
    Max,

    // Literals
    Number(String),
    StringLiteral(String),

    // Identifiers
    Identifier(String),
    QuotedIdentifier(String),

    // Operators & punctuation
    Star,         // *
    Comma,        // ,
    Dot,          // .
    Semicolon,    // ;
    LeftParen,    // (
    RightParen,   // )
    Equals,       // =
    NotEquals,    // != or <>
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Plus,         // +
    Minus,        // -
    Slash,        // /
    Percent,      // %

    // Comments
    LineComment(String),
    BlockComment(String),

    // Whitespace (preserved for formatting)
    Whitespace(String),
    Newline,

    // End of input
    Eof,
}

impl TokenKind {
    /// Returns true if this token is a SQL keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Select
                | TokenKind::From
                | TokenKind::Where
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Not
                | TokenKind::As
                | TokenKind::On
                | TokenKind::Join
                | TokenKind::Inner
                | TokenKind::Left
                | TokenKind::Right
                | TokenKind::Full
                | TokenKind::Outer
                | TokenKind::Cross
                | TokenKind::Insert
                | TokenKind::Into
                | TokenKind::Values
                | TokenKind::Update
                | TokenKind::Set
                | TokenKind::Delete
                | TokenKind::Create
                | TokenKind::Table
                | TokenKind::Drop
                | TokenKind::Alter
                | TokenKind::Index
                | TokenKind::If
                | TokenKind::Exists
                | TokenKind::Primary
                | TokenKind::Key
                | TokenKind::Unique
                | TokenKind::Default
                | TokenKind::Null
                | TokenKind::In
                | TokenKind::Between
                | TokenKind::Like
                | TokenKind::Is
                | TokenKind::Order
                | TokenKind::By
                | TokenKind::Asc
                | TokenKind::Desc
                | TokenKind::Group
                | TokenKind::Having
                | TokenKind::Limit
                | TokenKind::Offset
                | TokenKind::Union
                | TokenKind::All
                | TokenKind::Distinct
                | TokenKind::Case
                | TokenKind::When
                | TokenKind::Then
                | TokenKind::Else
                | TokenKind::End
                | TokenKind::Int
                | TokenKind::Integer
                | TokenKind::Varchar
                | TokenKind::Text
                | TokenKind::Boolean
                | TokenKind::Float
                | TokenKind::Double
                | TokenKind::Decimal
                | TokenKind::Date
                | TokenKind::Timestamp
                | TokenKind::Blob
                | TokenKind::BigInt
                | TokenKind::SmallInt
                | TokenKind::Char
                | TokenKind::AutoIncrement
                | TokenKind::NotNull
                | TokenKind::References
                | TokenKind::Foreign
                | TokenKind::Constraint
                | TokenKind::Check
                | TokenKind::Count
                | TokenKind::Sum
                | TokenKind::Avg
                | TokenKind::Min
                | TokenKind::Max
        )
    }

    /// The canonical uppercase keyword text for this token, if it is a keyword.
    pub fn keyword_str(&self) -> Option<&'static str> {
        match self {
            TokenKind::Select => Some("SELECT"),
            TokenKind::From => Some("FROM"),
            TokenKind::Where => Some("WHERE"),
            TokenKind::And => Some("AND"),
            TokenKind::Or => Some("OR"),
            TokenKind::Not => Some("NOT"),
            TokenKind::As => Some("AS"),
            TokenKind::On => Some("ON"),
            TokenKind::Join => Some("JOIN"),
            TokenKind::Inner => Some("INNER"),
            TokenKind::Left => Some("LEFT"),
            TokenKind::Right => Some("RIGHT"),
            TokenKind::Full => Some("FULL"),
            TokenKind::Outer => Some("OUTER"),
            TokenKind::Cross => Some("CROSS"),
            TokenKind::Insert => Some("INSERT"),
            TokenKind::Into => Some("INTO"),
            TokenKind::Values => Some("VALUES"),
            TokenKind::Update => Some("UPDATE"),
            TokenKind::Set => Some("SET"),
            TokenKind::Delete => Some("DELETE"),
            TokenKind::Create => Some("CREATE"),
            TokenKind::Table => Some("TABLE"),
            TokenKind::Drop => Some("DROP"),
            TokenKind::Alter => Some("ALTER"),
            TokenKind::Index => Some("INDEX"),
            TokenKind::If => Some("IF"),
            TokenKind::Exists => Some("EXISTS"),
            TokenKind::Primary => Some("PRIMARY"),
            TokenKind::Key => Some("KEY"),
            TokenKind::Unique => Some("UNIQUE"),
            TokenKind::Default => Some("DEFAULT"),
            TokenKind::Null => Some("NULL"),
            TokenKind::In => Some("IN"),
            TokenKind::Between => Some("BETWEEN"),
            TokenKind::Like => Some("LIKE"),
            TokenKind::Is => Some("IS"),
            TokenKind::Order => Some("ORDER"),
            TokenKind::By => Some("BY"),
            TokenKind::Asc => Some("ASC"),
            TokenKind::Desc => Some("DESC"),
            TokenKind::Group => Some("GROUP"),
            TokenKind::Having => Some("HAVING"),
            TokenKind::Limit => Some("LIMIT"),
            TokenKind::Offset => Some("OFFSET"),
            TokenKind::Union => Some("UNION"),
            TokenKind::All => Some("ALL"),
            TokenKind::Distinct => Some("DISTINCT"),
            TokenKind::Case => Some("CASE"),
            TokenKind::When => Some("WHEN"),
            TokenKind::Then => Some("THEN"),
            TokenKind::Else => Some("ELSE"),
            TokenKind::End => Some("END"),
            TokenKind::Int => Some("INT"),
            TokenKind::Integer => Some("INTEGER"),
            TokenKind::Varchar => Some("VARCHAR"),
            TokenKind::Text => Some("TEXT"),
            TokenKind::Boolean => Some("BOOLEAN"),
            TokenKind::Float => Some("FLOAT"),
            TokenKind::Double => Some("DOUBLE"),
            TokenKind::Decimal => Some("DECIMAL"),
            TokenKind::Date => Some("DATE"),
            TokenKind::Timestamp => Some("TIMESTAMP"),
            TokenKind::Blob => Some("BLOB"),
            TokenKind::BigInt => Some("BIGINT"),
            TokenKind::SmallInt => Some("SMALLINT"),
            TokenKind::Char => Some("CHAR"),
            TokenKind::AutoIncrement => Some("AUTO_INCREMENT"),
            TokenKind::NotNull => Some("NOT NULL"),
            TokenKind::References => Some("REFERENCES"),
            TokenKind::Foreign => Some("FOREIGN"),
            TokenKind::Constraint => Some("CONSTRAINT"),
            TokenKind::Check => Some("CHECK"),
            TokenKind::Count => Some("COUNT"),
            TokenKind::Sum => Some("SUM"),
            TokenKind::Avg => Some("AVG"),
            TokenKind::Min => Some("MIN"),
            TokenKind::Max => Some("MAX"),
            _ => None,
        }
    }
}

/// A token with its position in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    /// The original text from the source.
    pub text: String,
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column number.
    pub col: usize,
}
