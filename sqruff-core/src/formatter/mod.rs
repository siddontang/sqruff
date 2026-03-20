use crate::config::Config;
use sqruff_parser::*;

/// SQL formatter that pretty-prints AST back to formatted SQL.
pub struct Formatter<'a> {
    config: &'a Config,
}

impl<'a> Formatter<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn format(&self, file: &SqlFile) -> String {
        let mut output = String::new();
        for (i, stmt) in file.statements.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&self.format_statement(stmt));
            if self.config.format.trailing_semicolon {
                output.push(';');
            }
        }
        output.push('\n');
        output
    }

    fn format_statement(&self, stmt: &Statement) -> String {
        match &stmt.kind {
            StatementKind::Select(s) => self.format_select(s),
            StatementKind::Insert(s) => self.format_insert(s),
            StatementKind::Update(s) => self.format_update(s),
            StatementKind::Delete(s) => self.format_delete(s),
            StatementKind::CreateTable(s) => self.format_create_table(s),
        }
    }

    fn format_select(&self, sel: &SelectStatement) -> String {
        let indent = &self.config.format.indent;
        let mut parts = Vec::new();

        // SELECT [DISTINCT]
        let mut select_line = self.kw("SELECT").to_string();
        if sel.distinct {
            select_line.push(' ');
            select_line.push_str(&self.kw("DISTINCT"));
        }
        parts.push(select_line);

        // Columns
        let cols: Vec<String> = sel
            .columns
            .iter()
            .map(|c| {
                let mut s = format!("{}{}", indent, self.format_expr(&c.expr));
                if let Some(alias) = &c.alias {
                    s.push(' ');
                    s.push_str(&self.kw("AS"));
                    s.push(' ');
                    s.push_str(alias);
                }
                s
            })
            .collect();
        parts.push(cols.join(",\n"));

        // FROM
        if let Some(from) = &sel.from {
            parts.push(format!("{}", self.kw("FROM")));
            let mut table_str = format!("{}{}", indent, from.table.name);
            if let Some(alias) = &from.table.alias {
                table_str.push(' ');
                table_str.push_str(&self.kw("AS"));
                table_str.push(' ');
                table_str.push_str(alias);
            }
            parts.push(table_str);

            for join in &from.joins {
                let jt = match join.join_type {
                    JoinType::Inner => self.kw("INNER JOIN"),
                    JoinType::Left => self.kw("LEFT JOIN"),
                    JoinType::Right => self.kw("RIGHT JOIN"),
                    JoinType::Full => self.kw("FULL JOIN"),
                    JoinType::Cross => self.kw("CROSS JOIN"),
                };
                let mut join_str = format!("{}{} {}", indent, jt, join.table.name);
                if let Some(alias) = &join.table.alias {
                    join_str.push(' ');
                    join_str.push_str(&self.kw("AS"));
                    join_str.push(' ');
                    join_str.push_str(alias);
                }
                if let Some(on) = &join.on {
                    join_str.push(' ');
                    join_str.push_str(&self.kw("ON"));
                    join_str.push(' ');
                    join_str.push_str(&self.format_expr(on));
                }
                parts.push(join_str);
            }
        }

        // WHERE
        if let Some(wc) = &sel.where_clause {
            parts.push(format!("{}", self.kw("WHERE")));
            parts.push(format!("{}{}", indent, self.format_expr(wc)));
        }

        // GROUP BY
        if !sel.group_by.is_empty() {
            parts.push(format!("{} {}", self.kw("GROUP"), self.kw("BY")));
            let exprs: Vec<String> = sel
                .group_by
                .iter()
                .map(|e| format!("{}{}", indent, self.format_expr(e)))
                .collect();
            parts.push(exprs.join(",\n"));
        }

        // HAVING
        if let Some(h) = &sel.having {
            parts.push(format!("{}", self.kw("HAVING")));
            parts.push(format!("{}{}", indent, self.format_expr(h)));
        }

        // ORDER BY
        if !sel.order_by.is_empty() {
            parts.push(format!("{} {}", self.kw("ORDER"), self.kw("BY")));
            let items: Vec<String> = sel
                .order_by
                .iter()
                .map(|o| {
                    let dir = if o.descending {
                        format!(" {}", self.kw("DESC"))
                    } else {
                        String::new()
                    };
                    format!("{}{}{}", indent, self.format_expr(&o.expr), dir)
                })
                .collect();
            parts.push(items.join(",\n"));
        }

        // LIMIT
        if let Some(l) = &sel.limit {
            parts.push(format!("{} {}", self.kw("LIMIT"), self.format_expr(l)));
        }

        // OFFSET
        if let Some(o) = &sel.offset {
            parts.push(format!("{} {}", self.kw("OFFSET"), self.format_expr(o)));
        }

        parts.join("\n")
    }

    fn format_insert(&self, ins: &InsertStatement) -> String {
        let mut s = format!("{} {} {}", self.kw("INSERT"), self.kw("INTO"), ins.table);

        if !ins.columns.is_empty() {
            s.push_str(&format!(" ({})", ins.columns.join(", ")));
        }

        s.push('\n');
        s.push_str(&self.kw("VALUES"));

        for (i, row) in ins.values.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push('\n');
            let indent = &self.config.format.indent;
            let vals: Vec<String> = row.iter().map(|e| self.format_expr(e)).collect();
            s.push_str(&format!("{}({})", indent, vals.join(", ")));
        }

        s
    }

    fn format_update(&self, upd: &UpdateStatement) -> String {
        let indent = &self.config.format.indent;
        let mut s = format!("{} {}\n{}", self.kw("UPDATE"), upd.table, self.kw("SET"));

        for (i, a) in upd.assignments.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push('\n');
            s.push_str(&format!(
                "{}{} = {}",
                indent,
                a.column,
                self.format_expr(&a.value)
            ));
        }

        if let Some(wc) = &upd.where_clause {
            s.push('\n');
            s.push_str(&self.kw("WHERE"));
            s.push('\n');
            s.push_str(&format!("{}{}", indent, self.format_expr(wc)));
        }

        s
    }

    fn format_delete(&self, del: &DeleteStatement) -> String {
        let indent = &self.config.format.indent;
        let mut s = format!("{} {} {}", self.kw("DELETE"), self.kw("FROM"), del.table);

        if let Some(wc) = &del.where_clause {
            s.push('\n');
            s.push_str(&self.kw("WHERE"));
            s.push('\n');
            s.push_str(&format!("{}{}", indent, self.format_expr(wc)));
        }

        s
    }

    fn format_create_table(&self, ct: &CreateTableStatement) -> String {
        let indent = &self.config.format.indent;
        let mut s = format!("{} {}", self.kw("CREATE"), self.kw("TABLE"));

        if ct.if_not_exists {
            s.push_str(&format!(
                " {} {} {}",
                self.kw("IF"),
                self.kw("NOT"),
                self.kw("EXISTS")
            ));
        }

        s.push_str(&format!(" {} (\n", ct.name));

        let mut entries: Vec<String> = Vec::new();

        for col in &ct.columns {
            let mut col_str = format!("{}{} {}", indent, col.name, col.data_type);
            if col.primary_key {
                col_str.push_str(&format!(" {} {}", self.kw("PRIMARY"), self.kw("KEY")));
            }
            if col.unique {
                col_str.push_str(&format!(" {}", self.kw("UNIQUE")));
            }
            if let Some(false) = col.nullable {
                col_str.push_str(&format!(" {} {}", self.kw("NOT"), self.kw("NULL")));
            }
            if col.auto_increment {
                col_str.push_str(&format!(" {}", self.kw("AUTO_INCREMENT")));
            }
            if let Some(def) = &col.default {
                col_str.push_str(&format!(" {} {}", self.kw("DEFAULT"), self.format_expr(def)));
            }
            entries.push(col_str);
        }

        for constraint in &ct.constraints {
            match constraint {
                TableConstraint::PrimaryKey(cols) => {
                    entries.push(format!(
                        "{}{} {} ({})",
                        indent,
                        self.kw("PRIMARY"),
                        self.kw("KEY"),
                        cols.join(", ")
                    ));
                }
                TableConstraint::Unique(cols) => {
                    entries.push(format!(
                        "{}{} ({})",
                        indent,
                        self.kw("UNIQUE"),
                        cols.join(", ")
                    ));
                }
                TableConstraint::ForeignKey {
                    columns,
                    ref_table,
                    ref_columns,
                } => {
                    entries.push(format!(
                        "{}{} {} ({}) {} {} ({})",
                        indent,
                        self.kw("FOREIGN KEY"),
                        self.kw(""),
                        columns.join(", "),
                        self.kw("REFERENCES"),
                        ref_table,
                        ref_columns.join(", ")
                    ));
                }
            }
        }

        s.push_str(&entries.join(",\n"));
        s.push_str("\n)");
        s
    }

    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Star => "*".to_string(),
            Expr::Identifier(name) => name.clone(),
            Expr::QualifiedIdentifier(table, col) => format!("{}.{}", table, col),
            Expr::Number(n) => n.clone(),
            Expr::StringLiteral(s) => format!("'{}'", s.replace('\'', "''")),
            Expr::Null => self.kw("NULL"),
            Expr::BinaryOp { left, op, right } => {
                let op_str = match op {
                    BinOp::Eq => "=",
                    BinOp::NotEq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => return format!(
                        "{}\n{}{} {}",
                        self.format_expr(left),
                        &self.config.format.indent,
                        self.kw("AND"),
                        self.format_expr(right)
                    ),
                    BinOp::Or => return format!(
                        "{}\n{}{} {}",
                        self.format_expr(left),
                        &self.config.format.indent,
                        self.kw("OR"),
                        self.format_expr(right)
                    ),
                    BinOp::Plus => "+",
                    BinOp::Minus => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Like => return format!(
                        "{} {} {}",
                        self.format_expr(left),
                        self.kw("LIKE"),
                        self.format_expr(right)
                    ),
                };
                format!(
                    "{} {} {}",
                    self.format_expr(left),
                    op_str,
                    self.format_expr(right)
                )
            }
            Expr::UnaryOp { op, expr } => match op {
                UnaryOp::Not => format!("{} {}", self.kw("NOT"), self.format_expr(expr)),
                UnaryOp::Minus => format!("-{}", self.format_expr(expr)),
            },
            Expr::FunctionCall { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.format_expr(a)).collect();
                format!(
                    "{}({})",
                    if self.config.format.uppercase_keywords {
                        name.to_uppercase()
                    } else {
                        name.to_lowercase()
                    },
                    args_str.join(", ")
                )
            }
            Expr::IsNull(e) => format!("{} {} {}", self.format_expr(e), self.kw("IS"), self.kw("NULL")),
            Expr::IsNotNull(e) => format!(
                "{} {} {} {}",
                self.format_expr(e),
                self.kw("IS"),
                self.kw("NOT"),
                self.kw("NULL")
            ),
            Expr::InList { expr, list } => {
                let items: Vec<String> = list.iter().map(|e| self.format_expr(e)).collect();
                format!(
                    "{} {} ({})",
                    self.format_expr(expr),
                    self.kw("IN"),
                    items.join(", ")
                )
            }
            Expr::Between { expr, low, high } => format!(
                "{} {} {} {} {}",
                self.format_expr(expr),
                self.kw("BETWEEN"),
                self.format_expr(low),
                self.kw("AND"),
                self.format_expr(high)
            ),
            Expr::Nested(e) => format!("({})", self.format_expr(e)),
        }
    }

    /// Convert a keyword to the configured case.
    fn kw(&self, keyword: &str) -> String {
        if self.config.format.uppercase_keywords {
            keyword.to_uppercase()
        } else {
            keyword.to_lowercase()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqruff_parser::{Dialect, Parser};

    #[test]
    fn test_format_simple_select() {
        let sql = "select id,name from users where id=1";
        let config = Config::default();
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let fmt = Formatter::new(&config);
        let result = fmt.format(&file);
        assert!(result.contains("SELECT"));
        assert!(result.contains("FROM"));
        assert!(result.contains("WHERE"));
    }

    #[test]
    fn test_format_preserves_semicolon() {
        let sql = "SELECT 1";
        let config = Config::default();
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let fmt = Formatter::new(&config);
        let result = fmt.format(&file);
        assert!(result.trim().ends_with(';'));
    }

    #[test]
    fn test_format_create_table() {
        let sql = "create table users (id int primary key, name varchar(255) not null)";
        let config = Config::default();
        let file = Parser::parse_str(sql, Dialect::Generic).unwrap();
        let fmt = Formatter::new(&config);
        let result = fmt.format(&file);
        assert!(result.contains("CREATE TABLE"));
        assert!(result.contains("PRIMARY KEY"));
        assert!(result.contains("NOT NULL"));
    }
}
