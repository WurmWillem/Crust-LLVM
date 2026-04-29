use super::Parser;

use crate::{
    error::ParseErr,
    expression::{Expr, ExprType},
    parse_types::{BinaryOp, FnType, Precedence},
    token::{Literal, TokenType},
};

impl Parser {
    pub(super) fn expression(&mut self) -> Result<Expr, ParseErr> {
        self.parse_precedence(Precedence::Assignment)
    }

    pub(super) fn execute_prefix(
        &mut self,
        fn_type: FnType,
        can_assign: bool,
    ) -> Result<Expr, ParseErr> {
        match fn_type {
            FnType::Grouping => self.grouping(),
            FnType::Array => self.array(),
            FnType::Unary => self.unary(),
            FnType::Number => self.number(),
            FnType::String => self.string(),
            FnType::Literal => self.literal(),
            FnType::Var => self.var(can_assign),
            FnType::This => self.this(),
            _ => unreachable!(),
        }
    }

    pub(super) fn execute_infix(
        &mut self,
        left: Expr,
        fn_type: FnType,
        can_assign: bool,
    ) -> Result<Expr, ParseErr> {
        match fn_type {
            FnType::Binary => self.binary(left),
            FnType::Call => self.call(left),
            FnType::Index => self.index(left, can_assign),
            FnType::Dot => self.dot(left, can_assign),
            FnType::DoubleColon => self.double_colon(left),
            FnType::Cast => self.cast(left),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr, ParseErr> {
        self.advance();
        let (can_assign, mut expr) = self.parse_prefix(precedence)?;

        while self.peek().ty != TokenType::Eof
            && precedence <= self.peek().ty.to_parse_rule().precedence
        {
            self.advance();
            let infix = self.previous().ty.to_parse_rule().infix;
            expr = self.execute_infix(expr, infix, can_assign)?;
        }
        // dbg!(&expr);
        Ok(expr)
    }

    fn parse_prefix(&mut self, precedence: Precedence) -> Result<(bool, Expr), ParseErr> {
        let kind = self.previous().ty;

        let prefix = kind.to_parse_rule().prefix;
        if prefix == FnType::Empty {
            let msg = "Expected expression.";
            let err = ParseErr::new(self.previous().line, msg);
            return Err(err);
        }

        let can_assign = precedence <= Precedence::Assignment;
        let expr = self.execute_prefix(prefix, can_assign)?;
        Ok((can_assign, expr))
    }

    fn this(&mut self) -> Result<Expr, ParseErr> {
        Ok(Expr::new(ExprType::This, self.previous().line))
    }

    fn var(&mut self, can_assign: bool) -> Result<Expr, ParseErr> {
        let name = self.previous().lexeme;
        let line = self.previous().line;

        let ty = if can_assign && self.matches(TokenType::Equal) {
            let value = Box::new(self.expression()?);
            ExprType::Assign {
                name,
                new_value: value,
            }
        } else if can_assign && self.matches(TokenType::PlusEqual) {
            self.get_assign_shorthand(name, line, BinaryOp::Add)?
        } else if can_assign && self.matches(TokenType::MinEqual) {
            self.get_assign_shorthand(name, line, BinaryOp::Sub)?
        } else if can_assign && self.matches(TokenType::MulEqual) {
            self.get_assign_shorthand(name, line, BinaryOp::Mul)?
        } else if can_assign && self.matches(TokenType::DivEqual) {
            self.get_assign_shorthand(name, line, BinaryOp::Div)?
        } else {
            ExprType::Identifier(name)
        };
        let var = Expr::new(ty, line);
        Ok(var)
    }

    fn get_assign_shorthand(
        &mut self,
        name: String,
        line: u32,
        op: BinaryOp,
    ) -> Result<ExprType, ParseErr> {
        let var_ty = ExprType::Identifier(name.clone());
        let var = Box::new(Expr::new(var_ty, line));

        let operand = Box::new(self.expression()?);
        let ty = ExprType::Binary {
            left: var,
            op,
            right: operand,
        };

        let new_value = Box::new(Expr::new(ty, line));
        Ok(ExprType::Assign { name, new_value })
    }

    fn get_assign_shorthand_field(
        &mut self,
        field_name: String,
        line: u32,
        op: BinaryOp,
        inst: Expr,
    ) -> Result<ExprType, ParseErr> {
        let ty = ExprType::Dot {
            inst: Box::new(inst.clone()),
            property: field_name.clone(),
        };
        let left = Box::new(Expr::new(ty, line));

        let operand = Box::new(self.expression()?);
        let ty = ExprType::Binary {
            left,
            op,
            right: operand,
        };

        let new_value = Box::new(Expr::new(ty, line));
        let ty = ExprType::DotAssign {
            inst: Box::new(inst),
            property: field_name,
            new_value,
        };
        Ok(ty)
    }

    fn string(&mut self) -> Result<Expr, ParseErr> {
        let Literal::Str(value) = self.previous().literal else {
            unreachable!();
        };
        let kind = ExprType::Lit(Literal::Str(value));
        Ok(Expr::new(kind, self.previous().line))
    }

    fn grouping(&mut self) -> Result<Expr, ParseErr> {
        let expr = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
        Ok(expr)
    }

    fn double_colon(&mut self, r#type: Expr) -> Result<Expr, ParseErr> {
        self.consume(TokenType::Identifier, "Expected property name after '::'.")?;

        let property = self.previous();
        let ty = ExprType::Colon {
            inst: Box::new(r#type),
            property: property.lexeme,
        };
        Ok(Expr::new(ty, property.line))
    }

    fn dot(&mut self, inst: Expr, can_assign: bool) -> Result<Expr, ParseErr> {
        self.consume(TokenType::Identifier, "Expected property name after '.'.")?;
        let property = self.previous();
        let line = property.line;

        let ty = if self.matches(TokenType::Equal) && can_assign {
            let value = Box::new(self.expression()?);
            ExprType::DotAssign {
                inst: Box::new(inst),
                property: property.lexeme,
                new_value: value,
            }
        } else if can_assign && self.matches(TokenType::PlusEqual) {
            self.get_assign_shorthand_field(property.lexeme, line, BinaryOp::Add, inst)?
        } else if can_assign && self.matches(TokenType::MinEqual) {
            self.get_assign_shorthand_field(property.lexeme, line, BinaryOp::Sub, inst)?
        } else if can_assign && self.matches(TokenType::MulEqual) {
            self.get_assign_shorthand_field(property.lexeme, line, BinaryOp::Mul, inst)?
        } else if can_assign && self.matches(TokenType::DivEqual) {
            self.get_assign_shorthand_field(property.lexeme, line, BinaryOp::Div, inst)?
        } else {
            ExprType::Dot {
                inst: Box::new(inst),
                property: property.lexeme,
            }
        };

        Ok(Expr::new(ty, property.line))
    }
    fn index(&mut self, arr: Expr, can_assign: bool) -> Result<Expr, ParseErr> {
        let index = Box::new(self.expression()?);
        self.consume(TokenType::RightBracket, "Expected ']' after index.")?;

        let arr = Box::new(arr);
        let ty = if can_assign && self.matches(TokenType::Equal) {
            let value = Box::new(self.expression()?);
            ExprType::AssignIndex {
                arr,
                index,
                new_value: value,
            }
        } else {
            ExprType::Index { arr, index }
        };
        let expr = Expr::new(ty, self.previous().line);
        Ok(expr)
    }

    fn cast(&mut self, value: Expr) -> Result<Expr, ParseErr> {
        let line = self.previous().line;

        if let Some(target) = self.peek().as_value_type() {
            self.advance();
            let value = Box::new(value);
            let ty = ExprType::Cast {
                value,
                target_ty: target,
            };

            Ok(Expr::new(ty, line))
        } else {
            Err(ParseErr {
                line,
                msg: "Expected type after 'as' keyword.".to_string(),
            })
        }
    }

    fn call(&mut self, name: Expr) -> Result<Expr, ParseErr> {
        let mut args = Vec::new();
        while !self.check(TokenType::RightParen) {
            args.push(self.expression()?);

            if !self.matches(TokenType::Comma) {
                break;
            }
        }
        self.consume(
            TokenType::RightParen,
            "Expected ')' after function/constructor call.",
        )?;

        let ty = match name.expr {
            ExprType::Identifier(name) => ExprType::FuncCall {
                name,
                args,
                index: None,
            },
            ExprType::Dot { inst, property } => ExprType::MethodCall {
                inst,
                property,
                args,
                is_static: false,
            },
            ExprType::Colon { inst, property } => ExprType::MethodCall {
                inst,
                property,
                args,
                is_static: true,
            },
            _ => unreachable!(),
        };

        let expr = Expr::new(ty, self.previous().line);
        Ok(expr)
    }

    fn binary(&mut self, left: Expr) -> Result<Expr, ParseErr> {
        let left = Box::new(left);
        let op = BinaryOp::from_token_type(self.previous().ty);

        let precedence = op.get_precedency().to_next_precedency();

        let right = Box::new(self.parse_precedence(precedence)?);

        let line = self.previous().line;
        let kind = ExprType::Binary { left, op, right };

        let expr = Expr::new(kind, line);
        Ok(expr)
    }

    fn array(&mut self) -> Result<Expr, ParseErr> {
        let mut values = Vec::new();
        while !self.check(TokenType::RightBracket) {
            values.push(self.expression()?);

            if !self.matches(TokenType::Comma) {
                break;
            }
        }
        self.consume(TokenType::RightBracket, "Expected ']' at end of array.")?;

        let ty = ExprType::Array(values);
        Ok(Expr::new(ty, self.previous().line))
    }

    fn number(&mut self) -> Result<Expr, ParseErr> {
        let kind = match self.previous().literal {
            Literal::F64(n) => ExprType::Lit(Literal::F64(n)),
            Literal::I64(n) => ExprType::Lit(Literal::I64(n)),
            Literal::U64(n) => ExprType::Lit(Literal::U64(n)),
            _ => unreachable!(),
        };
        Ok(Expr::new(kind, self.previous().line))
    }

    fn unary(&mut self) -> Result<Expr, ParseErr> {
        let prefix = self.previous().ty;
        let value = Box::new(self.parse_precedence(Precedence::Unary)?);

        let line = self.previous().line;
        let kind = ExprType::Unary { prefix, value };
        let expr = Expr::new(kind, line);
        Ok(expr)
    }

    fn literal(&mut self) -> Result<Expr, ParseErr> {
        let literal = match self.previous().ty {
            TokenType::True => Literal::True,
            TokenType::False => Literal::False,
            TokenType::Null => Literal::Null,
            _ => unreachable!(),
        };
        let kind = ExprType::Lit(literal);
        Ok(Expr::new(kind, self.previous().line))
    }
}
