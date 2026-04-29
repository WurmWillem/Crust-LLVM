use crate::{
    error::ParseErr,
    expression::{Expr, ExprType},
    parse_types::BinaryOp,
    statement::{Stmt, StmtType},
    token::{Literal, TokenType},
    value::ValueType,
};

use super::Parser;

const EXPECTED_SEMICOLON_MSG: &str = "Expected ';' at end of statement.";

impl Parser {
    pub(super) fn declaration(&mut self) -> Result<Stmt, ParseErr> {
        if let Some(var_type) = self.peek().as_value_type() {
            self.advance();
            if self.peek().ty != TokenType::Identifier && self.peek().ty != TokenType::LeftBracket {
                self.regress();
                return self.statement();
            }

            self.advance();
            if self.peek().ty == TokenType::Num {
                self.regress();
                self.regress();
                return self.statement();
            }
            self.regress();
            self.var_decl(var_type)
        } else if self.matches(TokenType::Fn) {
            self.func_decl()
        } else if self.matches(TokenType::Struct) {
            self.struct_decl()
        } else if self.matches(TokenType::Enum) {
            self.enum_decl()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseErr> {
        if self.matches(TokenType::Print) {
            self.print_statement()
        } else if self.matches(TokenType::LeftBrace) {
            self.block()
        } else if self.matches(TokenType::If) {
            self.if_stmt()
        } else if self.matches(TokenType::While) {
            self.while_stmt()
        } else if self.matches(TokenType::For) {
            self.for_stmt()
        } else if self.matches(TokenType::Break) {
            self.break_stmt()
        } else if self.matches(TokenType::Continue) {
            self.continue_stmt()
        } else if self.matches(TokenType::Return) {
            self.return_stmt()
        } else {
            self.expr_stmt()
        }
    }

    fn var_decl(&mut self, mut ty: ValueType) -> Result<Stmt, ParseErr> {
        while self.matches(TokenType::LeftBracket) {
            self.consume(TokenType::RightBracket, "Expected ']' after left bracket.")?;
            ty = ValueType::Arr(Box::new(ty));
        }

        self.consume(TokenType::Identifier, "Expected variable name after type.")?;
        let name = self.previous().lexeme;
        let line = self.previous().line;

        let value = if self.matches(TokenType::Equal) {
            let value = self.expression()?;
            self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;
            value
        } else {
            self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;
            Expr::new(ExprType::Lit(Literal::Null), line)
        };

        let kind = StmtType::VarDecl { name, value, ty };
        let var = Stmt::new(kind, line);
        Ok(var)
    }

    fn enum_decl(&mut self) -> Result<Stmt, ParseErr> {
        self.consume(
            TokenType::Identifier,
            "Expected enum name after 'enum' keyword.",
        )?;
        let name = self.previous().lexeme;
        let line = self.previous().line;

        self.consume(TokenType::LeftBrace, "Expected '{' after enum name.")?;

        let mut variants = Vec::new();
        while !self.check(TokenType::RightBrace) {
            self.consume(TokenType::Identifier, "Expected variant name.")?;
            let variant_name = self.previous().lexeme;

            variants.push(variant_name);
            self.consume(TokenType::Comma, "Expected ',' after enum variant.")?;
        }
        self.consume(TokenType::RightBrace, "Expected '}' after struct body.")?;

        let ty = StmtType::Enum { name, variants };
        Ok(Stmt::new(ty, line))
    }

    fn struct_decl(&mut self) -> Result<Stmt, ParseErr> {
        self.consume(
            TokenType::Identifier,
            "Expected struct name after 'struct' keyword.",
        )?;
        let name = self.previous().lexeme;
        let line = self.previous().line;

        self.consume(TokenType::LeftBrace, "Expected '{' after struct name.")?;

        let mut fields = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Fn) {
            let mut field_ty = match self.advance().as_value_type() {
                Some(ty) => ty,
                None => {
                    let msg = "Expected type for field declaration in struct body.";
                    return Err(ParseErr::new(line, msg));
                }
            };
            while self.matches(TokenType::LeftBracket) {
                self.consume(TokenType::RightBracket, "Expected ']' after left bracket.")?;
                field_ty = ValueType::Arr(Box::new(field_ty));
            }

            self.consume(TokenType::Identifier, "Expected variable name after type.")?;
            let field_name = self.previous().lexeme;

            fields.push((field_ty, field_name));

            self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;
        }
        let mut methods = vec![];
        while self.matches(TokenType::Fn) {
            methods.push(self.func_decl()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after struct body.")?;

        let ty = StmtType::Struct {
            name,
            fields,
            methods,
        };
        Ok(Stmt::new(ty, line))
    }

    fn parse_parameter(&mut self) -> Result<(ValueType, String), ParseErr> {
        let var_ty = match self.advance().as_value_type() {
            Some(mut var_type) => {
                while self.matches(TokenType::LeftBracket) {
                    self.consume(TokenType::RightBracket, "Expected ']' after left bracket.")?;
                    var_type = ValueType::Arr(Box::new(var_type));
                }
                var_type
            }
            _ => {
                return Err(ParseErr::new(
                    self.previous().line,
                    "Expected type for parameter.",
                ));
            }
        };

        self.consume(TokenType::Identifier, "Expected parameter name.")?;
        let name = self.previous().lexeme;

        Ok((var_ty, name))
    }

    fn func_decl(&mut self) -> Result<Stmt, ParseErr> {
        self.consume(
            TokenType::Identifier,
            "Expected function name after 'fn' keyword.",
        )?;
        let name = self.previous().lexeme;
        let line = self.previous().line;

        self.consume(TokenType::LeftParen, "Expected '(' after function name.")?;

        let mut parameters = Vec::new();
        let mut use_self = false;
        if !self.check(TokenType::RightParen) {
            if self.matches(TokenType::This) {
                use_self = true;
                self.matches(TokenType::Comma);
            }

            if !self.check(TokenType::RightParen) {
                parameters.push(self.parse_parameter()?);
                while self.matches(TokenType::Comma) {
                    parameters.push(self.parse_parameter()?);
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after function name.")?;

        let mut return_ty = ValueType::Null;
        if self.matches(TokenType::Colon) {
            return_ty = match self.advance().as_value_type() {
                Some(return_ty) => return_ty,
                _ => {
                    return Err(ParseErr::new(
                        self.previous().line,
                        "Expected return type after finding ':'.",
                    ));
                }
            };
        }

        self.consume(
            TokenType::LeftBrace,
            "Expected '{' at begin of function body.",
        )?;

        let mut body = vec![];
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            body.push(self.declaration()?);
        }

        if self.peek().ty != TokenType::Eof {
            self.consume(
                TokenType::RightBrace,
                "Expected '}' at end of function body.",
            )?;
        }

        let fn_ty = StmtType::Func {
            name,
            parameters,
            body,
            return_ty,
            use_self,
        };
        let func = Stmt::new(fn_ty, line);
        Ok(func)
    }

    fn block(&mut self) -> Result<Stmt, ParseErr> {
        let mut stmts = vec![];
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            stmts.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' at end of block.")?;

        let ty = StmtType::Block(stmts);
        let block = Stmt::new(ty, self.previous().line);
        Ok(block)
    }

    fn continue_stmt(&mut self) -> Result<Stmt, ParseErr> {
        self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;
        let stmt = Stmt::new(StmtType::Continue, self.previous().line);
        Ok(stmt)
    }

    fn break_stmt(&mut self) -> Result<Stmt, ParseErr> {
        self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;
        let stmt = Stmt::new(StmtType::Break, self.previous().line);
        Ok(stmt)
    }

    fn return_stmt(&mut self) -> Result<Stmt, ParseErr> {
        let value_ty = ExprType::Lit(Literal::Null);
        let mut value = Expr::new(value_ty, self.previous().line);

        if !self.check(TokenType::Semicolon) {
            value = self.expression()?;
        }

        self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;

        let stmt_ty = StmtType::Return(value);
        let stmt = Stmt::new(stmt_ty, self.previous().line);
        Ok(stmt)
    }

    fn for_stmt(&mut self) -> Result<Stmt, ParseErr> {
        self.consume(TokenType::Identifier, "Expected variable name after 'for'.")?;
        let var = self.previous();
        let line = var.line;
        let name = var.lexeme;

        self.consume(TokenType::In, "Expected 'in' after 'for identifier'.")?;

        // declare var
        let value = self.expression()?;
        let ty = ValueType::I64;
        let kind = StmtType::VarDecl {
            name: name.clone(),
            value,
            ty,
        };
        let var = Box::new(Stmt::new(kind, line));

        // condition
        self.consume(TokenType::To, "Expected 'to' after 'for identifier'.")?;
        let end = Box::new(self.expression()?);
        let cast = ExprType::Cast {
            value: end,
            target_ty: ValueType::I64,
        };
        let cast = Expr::new(cast, line);

        let get_var_ty = ExprType::Identifier(name);
        let get_var = Box::new(Expr::new(get_var_ty, line));
        let condition_ty = ExprType::Binary {
            left: get_var,
            op: BinaryOp::Less,
            right: Box::new(cast),
        };
        let condition = Expr::new(condition_ty, line);

        // produce for stmt
        let body = Box::new(self.statement()?);
        let for_ty = StmtType::For {
            condition,
            body,
            var,
        };
        let stmt = Stmt::new(for_ty, line);

        Ok(stmt)
    }

    fn while_stmt(&mut self) -> Result<Stmt, ParseErr> {
        let condition = self.expression()?;
        let body = Box::new(self.statement()?);

        let ty = StmtType::While { condition, body };
        let stmt = Stmt::new(ty, self.previous().line);
        Ok(stmt)
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParseErr> {
        let line = self.previous().line;

        let condition = self.expression()?;
        let body = Box::new(self.statement()?);

        let mut final_else = None;
        if self.matches(TokenType::Else) {
            final_else = Some(Box::new(self.statement()?));
        }

        let ty = StmtType::If {
            condition,
            body,
            final_else,
        };
        Ok(Stmt::new(ty, line))
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseErr> {
        let kind = StmtType::Println(self.expression()?);
        self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;

        let stmt = Stmt::new(kind, self.previous().line);
        Ok(stmt)
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParseErr> {
        let kind = StmtType::Expr(self.expression()?);
        let stmt = Stmt::new(kind, self.previous().line);
        self.consume(TokenType::Semicolon, EXPECTED_SEMICOLON_MSG)?;
        Ok(stmt)
    }
}
