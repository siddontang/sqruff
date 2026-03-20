use crate::ast::*;
use crate::dialect::Dialect;
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

/// Recursive descent SQL parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    dialect: Dialect,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for ParseError {}

impl Parser {
    pub fn new(tokens: Vec<Token>, dialect: Dialect) -> Self {
        // Filter out comments for parsing (we already tracked casing in tokens)
        let tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::LineComment(_) | TokenKind::BlockComment(_)))
            .collect();
        Self {
            tokens,
            pos: 0,
            dialect,
        }
    }

    /// Parse SQL source text into a SqlFile.
    pub fn parse_str(input: &str, dialect: Dialect) -> Result<SqlFile, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().map_err(|e| ParseError {
            message: e.message,
            line: e.line,
            col: e.col,
        })?;
        let mut parser = Parser::new(tokens, dialect);
        parser.parse()
    }

    pub fn parse(&mut self) -> Result<SqlFile, ParseError> {
        let mut statements = Vec::new();
        while !self.at_end() {
            // skip stray semicolons
            if self.check(&TokenKind::Semicolon) {
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
            // optional trailing semicolon
            if self.check(&TokenKind::Semicolon) {
                self.advance();
            }
        }
        Ok(SqlFile {
            statements,
            dialect: self.dialect,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let tok = self.peek_token();
        let line = tok.line;
        let col = tok.col;

        let kind = match &tok.kind {
            TokenKind::Select => StatementKind::Select(self.parse_select()?),
            TokenKind::Insert => StatementKind::Insert(self.parse_insert()?),
            TokenKind::Update => StatementKind::Update(self.parse_update()?),
            TokenKind::Delete => StatementKind::Delete(self.parse_delete()?),
            TokenKind::Create => StatementKind::CreateTable(self.parse_create_table()?),
            _ => {
                return Err(ParseError {
                    message: format!("unexpected token: {:?}", tok.kind),
                    line: tok.line,
                    col: tok.col,
                })
            }
        };

        Ok(Statement { kind, line, col })
    }

    // ── SELECT ──────────────────────────────────────

    fn parse_select(&mut self) -> Result<SelectStatement, ParseError> {
        let mut keyword_cases = Vec::new();
        self.expect_keyword_tracking(&TokenKind::Select, &mut keyword_cases)?;

        let distinct = if self.check(&TokenKind::Distinct) {
            self.expect_keyword_tracking(&TokenKind::Distinct, &mut keyword_cases)?;
            true
        } else {
            false
        };

        let columns = self.parse_select_columns()?;

        let from = if self.check(&TokenKind::From) {
            self.expect_keyword_tracking(&TokenKind::From, &mut keyword_cases)?;
            Some(self.parse_from_clause(&mut keyword_cases)?)
        } else {
            None
        };

        let where_clause = if self.check(&TokenKind::Where) {
            self.expect_keyword_tracking(&TokenKind::Where, &mut keyword_cases)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        let group_by = if self.check(&TokenKind::Group) {
            self.expect_keyword_tracking(&TokenKind::Group, &mut keyword_cases)?;
            self.expect_keyword_tracking(&TokenKind::By, &mut keyword_cases)?;
            self.parse_expr_list()?
        } else {
            Vec::new()
        };

        let having = if self.check(&TokenKind::Having) {
            self.expect_keyword_tracking(&TokenKind::Having, &mut keyword_cases)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        let order_by = if self.check(&TokenKind::Order) {
            self.expect_keyword_tracking(&TokenKind::Order, &mut keyword_cases)?;
            self.expect_keyword_tracking(&TokenKind::By, &mut keyword_cases)?;
            self.parse_order_by_items(&mut keyword_cases)?
        } else {
            Vec::new()
        };

        let limit = if self.check(&TokenKind::Limit) {
            self.expect_keyword_tracking(&TokenKind::Limit, &mut keyword_cases)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        let offset = if self.check(&TokenKind::Offset) {
            self.expect_keyword_tracking(&TokenKind::Offset, &mut keyword_cases)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(SelectStatement {
            distinct,
            columns,
            from,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
            keyword_cases,
        })
    }

    fn parse_select_columns(&mut self) -> Result<Vec<SelectColumn>, ParseError> {
        let mut cols = Vec::new();
        loop {
            let tok = self.peek_token();
            let line = tok.line;
            let col = tok.col;

            if self.check(&TokenKind::Star) {
                self.advance();
                cols.push(SelectColumn {
                    expr: Expr::Star,
                    alias: None,
                    line,
                    col,
                });
            } else {
                let expr = self.parse_expr()?;
                let alias = if self.check(&TokenKind::As) {
                    self.advance();
                    Some(self.expect_identifier()?)
                } else if self.check_identifier() && !self.check(&TokenKind::From) {
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                cols.push(SelectColumn {
                    expr,
                    alias,
                    line,
                    col,
                });
            }

            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(cols)
    }

    fn parse_from_clause(
        &mut self,
        keyword_cases: &mut Vec<KeywordCase>,
    ) -> Result<FromClause, ParseError> {
        let table = self.parse_table_ref()?;
        let mut joins = Vec::new();

        loop {
            let join_type = if self.check(&TokenKind::Inner) {
                self.expect_keyword_tracking(&TokenKind::Inner, keyword_cases)?;
                self.expect_keyword_tracking(&TokenKind::Join, keyword_cases)?;
                Some(JoinType::Inner)
            } else if self.check(&TokenKind::Left) {
                self.expect_keyword_tracking(&TokenKind::Left, keyword_cases)?;
                if self.check(&TokenKind::Outer) {
                    self.expect_keyword_tracking(&TokenKind::Outer, keyword_cases)?;
                }
                self.expect_keyword_tracking(&TokenKind::Join, keyword_cases)?;
                Some(JoinType::Left)
            } else if self.check(&TokenKind::Right) {
                self.expect_keyword_tracking(&TokenKind::Right, keyword_cases)?;
                if self.check(&TokenKind::Outer) {
                    self.expect_keyword_tracking(&TokenKind::Outer, keyword_cases)?;
                }
                self.expect_keyword_tracking(&TokenKind::Join, keyword_cases)?;
                Some(JoinType::Right)
            } else if self.check(&TokenKind::Full) {
                self.expect_keyword_tracking(&TokenKind::Full, keyword_cases)?;
                if self.check(&TokenKind::Outer) {
                    self.expect_keyword_tracking(&TokenKind::Outer, keyword_cases)?;
                }
                self.expect_keyword_tracking(&TokenKind::Join, keyword_cases)?;
                Some(JoinType::Full)
            } else if self.check(&TokenKind::Cross) {
                self.expect_keyword_tracking(&TokenKind::Cross, keyword_cases)?;
                self.expect_keyword_tracking(&TokenKind::Join, keyword_cases)?;
                Some(JoinType::Cross)
            } else if self.check(&TokenKind::Join) {
                self.expect_keyword_tracking(&TokenKind::Join, keyword_cases)?;
                Some(JoinType::Inner)
            } else {
                None
            };

            match join_type {
                Some(jt) => {
                    let table = self.parse_table_ref()?;
                    let on = if self.check(&TokenKind::On) {
                        self.expect_keyword_tracking(&TokenKind::On, keyword_cases)?;
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    joins.push(JoinClause {
                        join_type: jt,
                        table,
                        on,
                    });
                }
                None => break,
            }
        }

        Ok(FromClause { table, joins })
    }

    fn parse_table_ref(&mut self) -> Result<TableRef, ParseError> {
        let tok = self.peek_token();
        let line = tok.line;
        let col = tok.col;
        let name = self.expect_identifier()?;
        let alias = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else if self.check_identifier()
            && !self.check(&TokenKind::On)
            && !self.check(&TokenKind::Inner)
            && !self.check(&TokenKind::Left)
            && !self.check(&TokenKind::Right)
            && !self.check(&TokenKind::Full)
            && !self.check(&TokenKind::Cross)
            && !self.check(&TokenKind::Join)
            && !self.check(&TokenKind::Where)
            && !self.check(&TokenKind::Group)
            && !self.check(&TokenKind::Order)
            && !self.check(&TokenKind::Limit)
            && !self.check(&TokenKind::Set)
        {
            Some(self.expect_identifier()?)
        } else {
            None
        };
        Ok(TableRef { name, alias, line, col })
    }

    fn parse_order_by_items(
        &mut self,
        keyword_cases: &mut Vec<KeywordCase>,
    ) -> Result<Vec<OrderByItem>, ParseError> {
        let mut items = Vec::new();
        loop {
            let expr = self.parse_expr()?;
            let descending = if self.check(&TokenKind::Desc) {
                self.expect_keyword_tracking(&TokenKind::Desc, keyword_cases)?;
                true
            } else if self.check(&TokenKind::Asc) {
                self.expect_keyword_tracking(&TokenKind::Asc, keyword_cases)?;
                false
            } else {
                false
            };
            items.push(OrderByItem { expr, descending });
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(items)
    }

    // ── INSERT ──────────────────────────────────────

    fn parse_insert(&mut self) -> Result<InsertStatement, ParseError> {
        let mut keyword_cases = Vec::new();
        self.expect_keyword_tracking(&TokenKind::Insert, &mut keyword_cases)?;
        self.expect_keyword_tracking(&TokenKind::Into, &mut keyword_cases)?;
        let table = self.expect_identifier()?;

        let columns = if self.check(&TokenKind::LeftParen) {
            self.advance();
            let cols = self.parse_identifier_list()?;
            self.expect(&TokenKind::RightParen)?;
            cols
        } else {
            Vec::new()
        };

        self.expect_keyword_tracking(&TokenKind::Values, &mut keyword_cases)?;

        let mut values = Vec::new();
        loop {
            self.expect(&TokenKind::LeftParen)?;
            let row = self.parse_expr_list()?;
            self.expect(&TokenKind::RightParen)?;
            values.push(row);
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(InsertStatement {
            table,
            columns,
            values,
            keyword_cases,
        })
    }

    // ── UPDATE ──────────────────────────────────────

    fn parse_update(&mut self) -> Result<UpdateStatement, ParseError> {
        let mut keyword_cases = Vec::new();
        self.expect_keyword_tracking(&TokenKind::Update, &mut keyword_cases)?;
        let table = self.expect_identifier()?;
        self.expect_keyword_tracking(&TokenKind::Set, &mut keyword_cases)?;

        let mut assignments = Vec::new();
        loop {
            let column = self.expect_identifier()?;
            self.expect(&TokenKind::Equals)?;
            let value = self.parse_expr()?;
            assignments.push(Assignment { column, value });
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        let where_clause = if self.check(&TokenKind::Where) {
            self.expect_keyword_tracking(&TokenKind::Where, &mut keyword_cases)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(UpdateStatement {
            table,
            assignments,
            where_clause,
            keyword_cases,
        })
    }

    // ── DELETE ──────────────────────────────────────

    fn parse_delete(&mut self) -> Result<DeleteStatement, ParseError> {
        let mut keyword_cases = Vec::new();
        self.expect_keyword_tracking(&TokenKind::Delete, &mut keyword_cases)?;
        self.expect_keyword_tracking(&TokenKind::From, &mut keyword_cases)?;
        let table = self.expect_identifier()?;

        let where_clause = if self.check(&TokenKind::Where) {
            self.expect_keyword_tracking(&TokenKind::Where, &mut keyword_cases)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(DeleteStatement {
            table,
            where_clause,
            keyword_cases,
        })
    }

    // ── CREATE TABLE ────────────────────────────────

    fn parse_create_table(&mut self) -> Result<CreateTableStatement, ParseError> {
        let mut keyword_cases = Vec::new();
        self.expect_keyword_tracking(&TokenKind::Create, &mut keyword_cases)?;
        self.expect_keyword_tracking(&TokenKind::Table, &mut keyword_cases)?;

        let if_not_exists = if self.check(&TokenKind::If) {
            self.expect_keyword_tracking(&TokenKind::If, &mut keyword_cases)?;
            self.expect_keyword_tracking(&TokenKind::Not, &mut keyword_cases)?;
            self.expect_keyword_tracking(&TokenKind::Exists, &mut keyword_cases)?;
            true
        } else {
            false
        };

        let name = self.expect_identifier()?;
        self.expect(&TokenKind::LeftParen)?;

        let mut columns = Vec::new();
        let mut constraints = Vec::new();

        loop {
            if self.check(&TokenKind::RightParen) {
                break;
            }

            // Check for table-level constraints
            if self.check(&TokenKind::Primary) || self.check(&TokenKind::Unique) || self.check(&TokenKind::Foreign) || self.check(&TokenKind::Constraint) {
                constraints.push(self.parse_table_constraint(&mut keyword_cases)?);
            } else {
                columns.push(self.parse_column_def(&mut keyword_cases)?);
            }

            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(CreateTableStatement {
            if_not_exists,
            name,
            columns,
            constraints,
            keyword_cases,
        })
    }

    fn parse_column_def(&mut self, keyword_cases: &mut Vec<KeywordCase>) -> Result<ColumnDef, ParseError> {
        let name = self.expect_identifier()?;
        let data_type = self.parse_data_type(keyword_cases)?;

        let mut nullable = None;
        let mut default = None;
        let mut primary_key = false;
        let mut unique = false;
        let mut auto_increment = false;

        // Parse column constraints
        loop {
            if self.check(&TokenKind::Not) {
                self.expect_keyword_tracking(&TokenKind::Not, keyword_cases)?;
                self.expect_keyword_tracking(&TokenKind::Null, keyword_cases)?;
                nullable = Some(false);
            } else if self.check(&TokenKind::Null) {
                self.expect_keyword_tracking(&TokenKind::Null, keyword_cases)?;
                nullable = Some(true);
            } else if self.check(&TokenKind::Primary) {
                self.expect_keyword_tracking(&TokenKind::Primary, keyword_cases)?;
                self.expect_keyword_tracking(&TokenKind::Key, keyword_cases)?;
                primary_key = true;
            } else if self.check(&TokenKind::Unique) {
                self.expect_keyword_tracking(&TokenKind::Unique, keyword_cases)?;
                unique = true;
            } else if self.check(&TokenKind::Default) {
                self.expect_keyword_tracking(&TokenKind::Default, keyword_cases)?;
                default = Some(self.parse_primary_expr()?);
            } else if self.check(&TokenKind::AutoIncrement) {
                self.expect_keyword_tracking(&TokenKind::AutoIncrement, keyword_cases)?;
                auto_increment = true;
            } else {
                break;
            }
        }

        Ok(ColumnDef {
            name,
            data_type,
            nullable,
            default,
            primary_key,
            unique,
            auto_increment,
        })
    }

    fn parse_data_type(&mut self, keyword_cases: &mut Vec<KeywordCase>) -> Result<DataType, ParseError> {
        let tok = self.peek_token();
        let dt = match &tok.kind {
            TokenKind::Int => {
                self.expect_keyword_tracking(&TokenKind::Int, keyword_cases)?;
                DataType::Int
            }
            TokenKind::Integer => {
                self.expect_keyword_tracking(&TokenKind::Integer, keyword_cases)?;
                DataType::Integer
            }
            TokenKind::BigInt => {
                self.expect_keyword_tracking(&TokenKind::BigInt, keyword_cases)?;
                DataType::BigInt
            }
            TokenKind::SmallInt => {
                self.expect_keyword_tracking(&TokenKind::SmallInt, keyword_cases)?;
                DataType::SmallInt
            }
            TokenKind::Varchar => {
                self.expect_keyword_tracking(&TokenKind::Varchar, keyword_cases)?;
                let len = self.parse_optional_paren_number()?;
                DataType::Varchar(len)
            }
            TokenKind::Char => {
                self.expect_keyword_tracking(&TokenKind::Char, keyword_cases)?;
                let len = self.parse_optional_paren_number()?;
                DataType::Char(len)
            }
            TokenKind::Text => {
                self.expect_keyword_tracking(&TokenKind::Text, keyword_cases)?;
                DataType::Text
            }
            TokenKind::Boolean => {
                self.expect_keyword_tracking(&TokenKind::Boolean, keyword_cases)?;
                DataType::Boolean
            }
            TokenKind::Float => {
                self.expect_keyword_tracking(&TokenKind::Float, keyword_cases)?;
                DataType::Float
            }
            TokenKind::Double => {
                self.expect_keyword_tracking(&TokenKind::Double, keyword_cases)?;
                DataType::Double
            }
            TokenKind::Decimal => {
                self.expect_keyword_tracking(&TokenKind::Decimal, keyword_cases)?;
                if self.check(&TokenKind::LeftParen) {
                    self.advance();
                    let p = self.expect_number()?;
                    let s = if self.check(&TokenKind::Comma) {
                        self.advance();
                        Some(self.expect_number()?)
                    } else {
                        None
                    };
                    self.expect(&TokenKind::RightParen)?;
                    DataType::Decimal(Some(p), s)
                } else {
                    DataType::Decimal(None, None)
                }
            }
            TokenKind::Date => {
                self.expect_keyword_tracking(&TokenKind::Date, keyword_cases)?;
                DataType::Date
            }
            TokenKind::Timestamp => {
                self.expect_keyword_tracking(&TokenKind::Timestamp, keyword_cases)?;
                DataType::Timestamp
            }
            TokenKind::Blob => {
                self.expect_keyword_tracking(&TokenKind::Blob, keyword_cases)?;
                DataType::Blob
            }
            TokenKind::Identifier(_) => {
                let name = self.expect_identifier()?;
                DataType::Custom(name)
            }
            _ => {
                return Err(ParseError {
                    message: format!("expected data type, got {:?}", tok.kind),
                    line: tok.line,
                    col: tok.col,
                })
            }
        };
        Ok(dt)
    }

    fn parse_optional_paren_number(&mut self) -> Result<Option<u32>, ParseError> {
        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let n = self.expect_number()?;
            self.expect(&TokenKind::RightParen)?;
            Ok(Some(n))
        } else {
            Ok(None)
        }
    }

    fn parse_table_constraint(&mut self, keyword_cases: &mut Vec<KeywordCase>) -> Result<TableConstraint, ParseError> {
        // Optional CONSTRAINT name
        if self.check(&TokenKind::Constraint) {
            self.expect_keyword_tracking(&TokenKind::Constraint, keyword_cases)?;
            let _ = self.expect_identifier()?; // constraint name, ignored for now
        }

        if self.check(&TokenKind::Primary) {
            self.expect_keyword_tracking(&TokenKind::Primary, keyword_cases)?;
            self.expect_keyword_tracking(&TokenKind::Key, keyword_cases)?;
            self.expect(&TokenKind::LeftParen)?;
            let cols = self.parse_identifier_list()?;
            self.expect(&TokenKind::RightParen)?;
            Ok(TableConstraint::PrimaryKey(cols))
        } else if self.check(&TokenKind::Unique) {
            self.expect_keyword_tracking(&TokenKind::Unique, keyword_cases)?;
            self.expect(&TokenKind::LeftParen)?;
            let cols = self.parse_identifier_list()?;
            self.expect(&TokenKind::RightParen)?;
            Ok(TableConstraint::Unique(cols))
        } else if self.check(&TokenKind::Foreign) {
            self.expect_keyword_tracking(&TokenKind::Foreign, keyword_cases)?;
            self.expect_keyword_tracking(&TokenKind::Key, keyword_cases)?;
            self.expect(&TokenKind::LeftParen)?;
            let columns = self.parse_identifier_list()?;
            self.expect(&TokenKind::RightParen)?;
            self.expect_keyword_tracking(&TokenKind::References, keyword_cases)?;
            let ref_table = self.expect_identifier()?;
            self.expect(&TokenKind::LeftParen)?;
            let ref_columns = self.parse_identifier_list()?;
            self.expect(&TokenKind::RightParen)?;
            Ok(TableConstraint::ForeignKey {
                columns,
                ref_table,
                ref_columns,
            })
        } else {
            let tok = self.peek_token();
            Err(ParseError {
                message: format!("expected table constraint, got {:?}", tok.kind),
                line: tok.line,
                col: tok.col,
            })
        }
    }

    // ── Expression parsing ──────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.check(&TokenKind::Or) {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expr()?;
        while self.check(&TokenKind::And) {
            self.advance();
            let right = self.parse_not_expr()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_not_expr(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::Not) {
            self.advance();
            let expr = self.parse_not_expr()?;
            Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            })
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_addition()?;

        // IS NULL / IS NOT NULL
        if self.check(&TokenKind::Is) {
            self.advance();
            if self.check(&TokenKind::Not) {
                self.advance();
                self.expect(&TokenKind::Null)?;
                return Ok(Expr::IsNotNull(Box::new(left)));
            } else {
                self.expect(&TokenKind::Null)?;
                return Ok(Expr::IsNull(Box::new(left)));
            }
        }

        // IN (...)
        if self.check(&TokenKind::In) {
            self.advance();
            self.expect(&TokenKind::LeftParen)?;
            let list = self.parse_expr_list()?;
            self.expect(&TokenKind::RightParen)?;
            return Ok(Expr::InList {
                expr: Box::new(left),
                list,
            });
        }

        // BETWEEN ... AND ...
        if self.check(&TokenKind::Between) {
            self.advance();
            let low = self.parse_addition()?;
            self.expect(&TokenKind::And)?;
            let high = self.parse_addition()?;
            return Ok(Expr::Between {
                expr: Box::new(left),
                low: Box::new(low),
                high: Box::new(high),
            });
        }

        // LIKE
        if self.check(&TokenKind::Like) {
            self.advance();
            let right = self.parse_addition()?;
            return Ok(Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Like,
                right: Box::new(right),
            });
        }

        // Comparison operators
        let op = match self.peek_token().kind {
            TokenKind::Equals => Some(BinOp::Eq),
            TokenKind::NotEquals => Some(BinOp::NotEq),
            TokenKind::LessThan => Some(BinOp::Lt),
            TokenKind::GreaterThan => Some(BinOp::Gt),
            TokenKind::LessEqual => Some(BinOp::LtEq),
            TokenKind::GreaterEqual => Some(BinOp::GtEq),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let right = self.parse_addition()?;
            Ok(Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.peek_token().kind {
                TokenKind::Plus => BinOp::Plus,
                TokenKind::Minus => BinOp::Minus,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek_token().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_primary_expr()?;
            Ok(Expr::UnaryOp {
                op: UnaryOp::Minus,
                expr: Box::new(expr),
            })
        } else {
            self.parse_primary_expr()
        }
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek_token();

        match &tok.kind {
            TokenKind::Number(n) => {
                let n = n.clone();
                self.advance();
                Ok(Expr::Number(n))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            TokenKind::Star => {
                self.advance();
                Ok(Expr::Star)
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(Expr::Nested(Box::new(expr)))
            }
            // Aggregate functions
            TokenKind::Count | TokenKind::Sum | TokenKind::Avg | TokenKind::Min | TokenKind::Max => {
                let name = tok.text.clone();
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let args = if self.check(&TokenKind::Star) {
                    self.advance();
                    vec![Expr::Star]
                } else {
                    self.parse_expr_list()?
                };
                self.expect(&TokenKind::RightParen)?;
                Ok(Expr::FunctionCall { name, args })
            }
            TokenKind::Identifier(_) | TokenKind::QuotedIdentifier(_) => {
                let name = self.expect_identifier()?;

                // Check for function call
                if self.check(&TokenKind::LeftParen) {
                    self.advance();
                    let args = if self.check(&TokenKind::RightParen) {
                        Vec::new()
                    } else {
                        self.parse_expr_list()?
                    };
                    self.expect(&TokenKind::RightParen)?;
                    Ok(Expr::FunctionCall { name, args })
                }
                // Check for qualified identifier: table.column
                else if self.check(&TokenKind::Dot) {
                    self.advance();
                    if self.check(&TokenKind::Star) {
                        self.advance();
                        Ok(Expr::QualifiedIdentifier(name, "*".to_string()))
                    } else {
                        let col = self.expect_identifier()?;
                        Ok(Expr::QualifiedIdentifier(name, col))
                    }
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            _ => Err(ParseError {
                message: format!("unexpected token in expression: {:?}", tok.kind),
                line: tok.line,
                col: tok.col,
            }),
        }
    }

    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = Vec::new();
        exprs.push(self.parse_expr()?);
        while self.check(&TokenKind::Comma) {
            self.advance();
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut ids = Vec::new();
        ids.push(self.expect_identifier()?);
        while self.check(&TokenKind::Comma) {
            self.advance();
            ids.push(self.expect_identifier()?);
        }
        Ok(ids)
    }

    // ── Helpers ─────────────────────────────────────

    fn peek_token(&self) -> Token {
        self.tokens
            .get(self.pos)
            .cloned()
            .unwrap_or(Token {
                kind: TokenKind::Eof,
                text: String::new(),
                line: 0,
                col: 0,
            })
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek_token();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].kind == TokenKind::Eof
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek_token().kind) == std::mem::discriminant(kind)
    }

    fn check_identifier(&self) -> bool {
        matches!(
            self.peek_token().kind,
            TokenKind::Identifier(_) | TokenKind::QuotedIdentifier(_)
        )
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        let tok = self.peek_token();
        if std::mem::discriminant(&tok.kind) == std::mem::discriminant(kind) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: format!("expected {:?}, got {:?}", kind, tok.kind),
                line: tok.line,
                col: tok.col,
            })
        }
    }

    fn expect_keyword_tracking(
        &mut self,
        kind: &TokenKind,
        cases: &mut Vec<KeywordCase>,
    ) -> Result<Token, ParseError> {
        let tok = self.expect(kind)?;
        if let Some(canonical) = tok.kind.keyword_str() {
            cases.push(KeywordCase {
                keyword: canonical.to_string(),
                original: tok.text.clone(),
                line: tok.line,
                col: tok.col,
            });
        }
        Ok(tok)
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let tok = self.peek_token();
        match &tok.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            TokenKind::QuotedIdentifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError {
                message: format!("expected identifier, got {:?}", tok.kind),
                line: tok.line,
                col: tok.col,
            }),
        }
    }

    fn expect_number(&mut self) -> Result<u32, ParseError> {
        let tok = self.peek_token();
        if let TokenKind::Number(n) = &tok.kind {
            let val = n.parse::<u32>().map_err(|_| ParseError {
                message: format!("invalid number: {}", n),
                line: tok.line,
                col: tok.col,
            })?;
            self.advance();
            Ok(val)
        } else {
            Err(ParseError {
                message: format!("expected number, got {:?}", tok.kind),
                line: tok.line,
                col: tok.col,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_select() {
        let sql = "SELECT id, name FROM users WHERE id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        assert_eq!(file.statements.len(), 1);
        match &file.statements[0].kind {
            StatementKind::Select(sel) => {
                assert_eq!(sel.columns.len(), 2);
                assert!(sel.from.is_some());
                assert!(sel.where_clause.is_some());
            }
            _ => panic!("expected SELECT"),
        }
    }

    #[test]
    fn test_parse_select_star() {
        let sql = "SELECT * FROM users";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Select(sel) => {
                assert_eq!(sel.columns.len(), 1);
                assert!(matches!(sel.columns[0].expr, Expr::Star));
            }
            _ => panic!("expected SELECT"),
        }
    }

    #[test]
    fn test_parse_insert() {
        let sql = "INSERT INTO users (id, name) VALUES (1, 'Alice')";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Insert(ins) => {
                assert_eq!(ins.table, "users");
                assert_eq!(ins.columns, vec!["id", "name"]);
                assert_eq!(ins.values.len(), 1);
                assert_eq!(ins.values[0].len(), 2);
            }
            _ => panic!("expected INSERT"),
        }
    }

    #[test]
    fn test_parse_update() {
        let sql = "UPDATE users SET name = 'Bob' WHERE id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Update(upd) => {
                assert_eq!(upd.table, "users");
                assert_eq!(upd.assignments.len(), 1);
                assert!(upd.where_clause.is_some());
            }
            _ => panic!("expected UPDATE"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let sql = "DELETE FROM users WHERE id = 1";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Delete(del) => {
                assert_eq!(del.table, "users");
                assert!(del.where_clause.is_some());
            }
            _ => panic!("expected DELETE"),
        }
    }

    #[test]
    fn test_parse_delete_no_where() {
        let sql = "DELETE FROM users";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Delete(del) => {
                assert!(del.where_clause.is_none());
            }
            _ => panic!("expected DELETE"),
        }
    }

    #[test]
    fn test_parse_create_table() {
        let sql = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255) NOT NULL, email TEXT)";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::CreateTable(ct) => {
                assert_eq!(ct.name, "users");
                assert_eq!(ct.columns.len(), 3);
                assert!(ct.columns[0].primary_key);
                assert_eq!(ct.columns[1].nullable, Some(false));
            }
            _ => panic!("expected CREATE TABLE"),
        }
    }

    #[test]
    fn test_parse_select_with_join() {
        let sql = "SELECT u.id, o.total FROM users u INNER JOIN orders o ON u.id = o.user_id";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Select(sel) => {
                let from = sel.from.as_ref().unwrap();
                assert_eq!(from.table.name, "users");
                assert_eq!(from.table.alias.as_deref(), Some("u"));
                assert_eq!(from.joins.len(), 1);
            }
            _ => panic!("expected SELECT"),
        }
    }

    #[test]
    fn test_parse_multiple_statements() {
        let sql = "SELECT 1; SELECT 2;";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        assert_eq!(file.statements.len(), 2);
    }

    #[test]
    fn test_parse_select_with_alias() {
        let sql = "SELECT id AS user_id FROM users";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Select(sel) => {
                assert_eq!(sel.columns[0].alias.as_deref(), Some("user_id"));
            }
            _ => panic!("expected SELECT"),
        }
    }

    #[test]
    fn test_parse_aggregate_functions() {
        let sql = "SELECT COUNT(*), SUM(amount) FROM orders";
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        match &file.statements[0].kind {
            StatementKind::Select(sel) => {
                assert_eq!(sel.columns.len(), 2);
                assert!(matches!(&sel.columns[0].expr, Expr::FunctionCall { name, .. } if name == "COUNT"));
            }
            _ => panic!("expected SELECT"),
        }
    }
}
