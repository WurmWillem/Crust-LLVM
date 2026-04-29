use crate::{
    error::{ParseErr, print_error},
    statement::Stmt,
    token::{Token, TokenType},
};

mod parse_expr;
mod parse_stmt;

use colored::Colorize;

pub struct Parser {
    tokens: Vec<Token>,
    current_token: usize,
}
impl Parser {
    pub fn compile(tokens: Vec<Token>) -> Option<Vec<Stmt>> {
        let mut parser = Parser {
            tokens,
            current_token: 0,
        };

        let mut had_error = false;
        let mut statements = Vec::new();
        while !parser.check(TokenType::Eof) {
            match parser.declaration() {
                Ok(result) => {
                    statements.push(result);
                }
                Err(err) => {
                    print_error(err.line, &err.msg);
                    had_error = true;
                    parser.synchronize();
                }
            }
        }
        parser.current_token += 1;

        if had_error {
            return None;
        }

        if parser.current_token != parser.tokens.len() {
            println!("{}", "Not all tokens were parsed.".red());
        }
        Some(statements)
    }

    fn synchronize(&mut self) {
        // self.advance();

        while self.peek().ty != TokenType::Eof {
            // if we just consumed a semicolon, we probably ended a statement
            if self.previous().ty == TokenType::Semicolon && self.peek().ty != TokenType::RightBrace
            {
                return;
            }

            // check if next token looks like the start of a new statement
            match self.peek().ty {
                TokenType::Struct
                | TokenType::Fn
                | TokenType::F64
                | TokenType::Bool
                | TokenType::Str
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => (),
            }

            self.advance();
        }
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<(), ParseErr> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(ParseErr {
                line: self.previous().line,
                msg: msg.to_string(),
            })
        }
    }

    fn matches(&mut self, kind: TokenType) -> bool {
        if !self.check(kind) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&self, kind: TokenType) -> bool {
        self.peek().ty == kind
    }

    fn advance(&mut self) -> Token {
        if self.peek().ty != TokenType::Eof {
            self.current_token += 1;
        }
        self.previous()
    }

    fn regress(&mut self) {
        self.current_token -= 1;
    }

    fn peek(&self) -> Token {
        self.tokens[self.current_token].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current_token - 1].clone()
    }
}
