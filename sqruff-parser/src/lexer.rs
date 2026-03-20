use crate::token::{Token, TokenKind};

/// SQL lexer — converts source text into a stream of tokens.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Tokenize the entire input, excluding whitespace and comments
    /// (but preserving them internally for formatting).
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            if tok.kind == TokenKind::Eof {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        Ok(tokens)
    }

    /// Tokenize keeping all tokens including whitespace/comments.
    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, LexError> {
        self.tokenize()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.input.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        // Skip whitespace
        self.skip_whitespace();

        let line = self.line;
        let col = self.col;

        let Some(ch) = self.peek() else {
            return Ok(Token {
                kind: TokenKind::Eof,
                text: String::new(),
                line,
                col,
            });
        };

        // Line comment: -- ...
        if ch == '-' && self.peek_ahead(1) == Some('-') {
            return self.lex_line_comment(line, col);
        }

        // Block comment: /* ... */
        if ch == '/' && self.peek_ahead(1) == Some('*') {
            return self.lex_block_comment(line, col);
        }

        // String literal: 'text'
        if ch == '\'' {
            return self.lex_string_literal(line, col);
        }

        // Quoted identifier: "name" or `name`
        if ch == '"' || ch == '`' {
            return self.lex_quoted_identifier(line, col);
        }

        // Number
        if ch.is_ascii_digit() {
            return self.lex_number(line, col);
        }

        // Identifier or keyword
        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.lex_identifier_or_keyword(line, col);
        }

        // Operators and punctuation
        self.lex_operator(line, col)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn lex_line_comment(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let mut text = String::new();
        // consume --
        text.push(self.advance().unwrap());
        text.push(self.advance().unwrap());
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            text.push(self.advance().unwrap());
        }
        let comment = text[2..].trim().to_string();
        Ok(Token {
            kind: TokenKind::LineComment(comment),
            text,
            line,
            col,
        })
    }

    fn lex_block_comment(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let mut text = String::new();
        text.push(self.advance().unwrap()); // /
        text.push(self.advance().unwrap()); // *
        loop {
            match self.advance() {
                Some('*') if self.peek() == Some('/') => {
                    text.push('*');
                    text.push(self.advance().unwrap());
                    break;
                }
                Some(ch) => text.push(ch),
                None => {
                    return Err(LexError {
                        message: "unterminated block comment".to_string(),
                        line,
                        col,
                    })
                }
            }
        }
        let comment = text[2..text.len() - 2].trim().to_string();
        Ok(Token {
            kind: TokenKind::BlockComment(comment),
            text,
            line,
            col,
        })
    }

    fn lex_string_literal(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let mut text = String::new();
        let quote = self.advance().unwrap(); // '
        text.push(quote);
        let mut value = String::new();
        loop {
            match self.advance() {
                Some('\'') if self.peek() == Some('\'') => {
                    // escaped quote ''
                    text.push('\'');
                    text.push(self.advance().unwrap());
                    value.push('\'');
                }
                Some('\'') => {
                    text.push('\'');
                    break;
                }
                Some(ch) => {
                    text.push(ch);
                    value.push(ch);
                }
                None => {
                    return Err(LexError {
                        message: "unterminated string literal".to_string(),
                        line,
                        col,
                    })
                }
            }
        }
        Ok(Token {
            kind: TokenKind::StringLiteral(value),
            text,
            line,
            col,
        })
    }

    fn lex_quoted_identifier(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let mut text = String::new();
        let quote = self.advance().unwrap();
        text.push(quote);
        let close = if quote == '`' { '`' } else { '"' };
        let mut name = String::new();
        loop {
            match self.advance() {
                Some(ch) if ch == close => {
                    text.push(ch);
                    break;
                }
                Some(ch) => {
                    text.push(ch);
                    name.push(ch);
                }
                None => {
                    return Err(LexError {
                        message: format!("unterminated quoted identifier starting with {}", quote),
                        line,
                        col,
                    })
                }
            }
        }
        Ok(Token {
            kind: TokenKind::QuotedIdentifier(name),
            text,
            line,
            col,
        })
    }

    fn lex_number(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                text.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Ok(Token {
            kind: TokenKind::Number(text.clone()),
            text,
            line,
            col,
        })
    }

    fn lex_identifier_or_keyword(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        let kind = match text.to_uppercase().as_str() {
            "SELECT" => TokenKind::Select,
            "FROM" => TokenKind::From,
            "WHERE" => TokenKind::Where,
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            "NOT" => TokenKind::Not,
            "AS" => TokenKind::As,
            "ON" => TokenKind::On,
            "JOIN" => TokenKind::Join,
            "INNER" => TokenKind::Inner,
            "LEFT" => TokenKind::Left,
            "RIGHT" => TokenKind::Right,
            "FULL" => TokenKind::Full,
            "OUTER" => TokenKind::Outer,
            "CROSS" => TokenKind::Cross,
            "INSERT" => TokenKind::Insert,
            "INTO" => TokenKind::Into,
            "VALUES" => TokenKind::Values,
            "UPDATE" => TokenKind::Update,
            "SET" => TokenKind::Set,
            "DELETE" => TokenKind::Delete,
            "CREATE" => TokenKind::Create,
            "TABLE" => TokenKind::Table,
            "DROP" => TokenKind::Drop,
            "ALTER" => TokenKind::Alter,
            "INDEX" => TokenKind::Index,
            "IF" => TokenKind::If,
            "EXISTS" => TokenKind::Exists,
            "PRIMARY" => TokenKind::Primary,
            "KEY" => TokenKind::Key,
            "UNIQUE" => TokenKind::Unique,
            "DEFAULT" => TokenKind::Default,
            "NULL" => TokenKind::Null,
            "IN" => TokenKind::In,
            "BETWEEN" => TokenKind::Between,
            "LIKE" => TokenKind::Like,
            "IS" => TokenKind::Is,
            "ORDER" => TokenKind::Order,
            "BY" => TokenKind::By,
            "ASC" => TokenKind::Asc,
            "DESC" => TokenKind::Desc,
            "GROUP" => TokenKind::Group,
            "HAVING" => TokenKind::Having,
            "LIMIT" => TokenKind::Limit,
            "OFFSET" => TokenKind::Offset,
            "UNION" => TokenKind::Union,
            "ALL" => TokenKind::All,
            "DISTINCT" => TokenKind::Distinct,
            "CASE" => TokenKind::Case,
            "WHEN" => TokenKind::When,
            "THEN" => TokenKind::Then,
            "ELSE" => TokenKind::Else,
            "END" => TokenKind::End,
            "INT" => TokenKind::Int,
            "INTEGER" => TokenKind::Integer,
            "VARCHAR" => TokenKind::Varchar,
            "TEXT" => TokenKind::Text,
            "BOOLEAN" => TokenKind::Boolean,
            "FLOAT" => TokenKind::Float,
            "DOUBLE" => TokenKind::Double,
            "DECIMAL" => TokenKind::Decimal,
            "DATE" => TokenKind::Date,
            "TIMESTAMP" => TokenKind::Timestamp,
            "BLOB" => TokenKind::Blob,
            "BIGINT" => TokenKind::BigInt,
            "SMALLINT" => TokenKind::SmallInt,
            "CHAR" => TokenKind::Char,
            "AUTO_INCREMENT" => TokenKind::AutoIncrement,
            "REFERENCES" => TokenKind::References,
            "FOREIGN" => TokenKind::Foreign,
            "CONSTRAINT" => TokenKind::Constraint,
            "CHECK" => TokenKind::Check,
            "COUNT" => TokenKind::Count,
            "SUM" => TokenKind::Sum,
            "AVG" => TokenKind::Avg,
            "MIN" => TokenKind::Min,
            "MAX" => TokenKind::Max,
            _ => TokenKind::Identifier(text.clone()),
        };
        Ok(Token {
            kind,
            text,
            line,
            col,
        })
    }

    fn lex_operator(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let ch = self.advance().unwrap();
        let (kind, text) = match ch {
            '*' => (TokenKind::Star, "*".to_string()),
            ',' => (TokenKind::Comma, ",".to_string()),
            '.' => (TokenKind::Dot, ".".to_string()),
            ';' => (TokenKind::Semicolon, ";".to_string()),
            '(' => (TokenKind::LeftParen, "(".to_string()),
            ')' => (TokenKind::RightParen, ")".to_string()),
            '+' => (TokenKind::Plus, "+".to_string()),
            '-' => (TokenKind::Minus, "-".to_string()),
            '/' => (TokenKind::Slash, "/".to_string()),
            '%' => (TokenKind::Percent, "%".to_string()),
            '=' => (TokenKind::Equals, "=".to_string()),
            '!' if self.peek() == Some('=') => {
                self.advance();
                (TokenKind::NotEquals, "!=".to_string())
            }
            '<' if self.peek() == Some('>') => {
                self.advance();
                (TokenKind::NotEquals, "<>".to_string())
            }
            '<' if self.peek() == Some('=') => {
                self.advance();
                (TokenKind::LessEqual, "<=".to_string())
            }
            '<' => (TokenKind::LessThan, "<".to_string()),
            '>' if self.peek() == Some('=') => {
                self.advance();
                (TokenKind::GreaterEqual, ">=".to_string())
            }
            '>' => (TokenKind::GreaterThan, ">".to_string()),
            _ => {
                return Err(LexError {
                    message: format!("unexpected character: '{}'", ch),
                    line,
                    col,
                })
            }
        };
        Ok(Token { kind, text, line, col })
    }
}

/// Error from the lexer.
#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let mut lexer = Lexer::new("SELECT id, name FROM users WHERE id = 1;");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Select);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("id".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Comma);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("name".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::From);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("users".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::Where);
        assert_eq!(tokens[7].kind, TokenKind::Identifier("id".to_string()));
        assert_eq!(tokens[8].kind, TokenKind::Equals);
        assert_eq!(tokens[9].kind, TokenKind::Number("1".to_string()));
        assert_eq!(tokens[10].kind, TokenKind::Semicolon);
        assert_eq!(tokens[11].kind, TokenKind::Eof);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("SELECT 'hello world'");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens[1].kind,
            TokenKind::StringLiteral("hello world".to_string())
        );
    }

    #[test]
    fn test_escaped_string() {
        let mut lexer = Lexer::new("SELECT 'it''s'");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens[1].kind,
            TokenKind::StringLiteral("it's".to_string())
        );
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("-- this is a comment\nSELECT 1");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::LineComment(_)));
        assert_eq!(tokens[1].kind, TokenKind::Select);
    }

    #[test]
    fn test_block_comment() {
        let mut lexer = Lexer::new("/* block */ SELECT 1");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::BlockComment(_)));
        assert_eq!(tokens[1].kind, TokenKind::Select);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("<> != <= >= < > =");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::NotEquals);
        assert_eq!(tokens[1].kind, TokenKind::NotEquals);
        assert_eq!(tokens[2].kind, TokenKind::LessEqual);
        assert_eq!(tokens[3].kind, TokenKind::GreaterEqual);
        assert_eq!(tokens[4].kind, TokenKind::LessThan);
        assert_eq!(tokens[5].kind, TokenKind::GreaterThan);
        assert_eq!(tokens[6].kind, TokenKind::Equals);
    }

    #[test]
    fn test_quoted_identifier() {
        let mut lexer = Lexer::new("SELECT \"my column\" FROM `table`");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens[1].kind,
            TokenKind::QuotedIdentifier("my column".to_string())
        );
        assert_eq!(tokens[3].kind, TokenKind::QuotedIdentifier("table".to_string()));
    }

    #[test]
    fn test_case_insensitive_keywords() {
        let mut lexer = Lexer::new("select FROM Where");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Select);
        assert_eq!(tokens[0].text, "select");
        assert_eq!(tokens[1].kind, TokenKind::From);
        assert_eq!(tokens[2].kind, TokenKind::Where);
    }

    #[test]
    fn test_line_numbers() {
        let mut lexer = Lexer::new("SELECT\n  id\nFROM users");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].line, 1); // SELECT
        assert_eq!(tokens[1].line, 2); // id
        assert_eq!(tokens[2].line, 3); // FROM
    }
}
