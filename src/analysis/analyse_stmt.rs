use super::types::Symbol;
use crate::{
    analysis::types::{SemErr, SemErrType},
    expression::ExprType,
    statement::{Stmt, StmtType},
    token::Literal,
    value::ValueType,
};

use super::Analyser;

impl Analyser {
    pub fn analyse_stmt(&mut self, stmt: &mut Stmt) -> Result<(), SemErr> {
        let line = stmt.get_line();
        match &mut stmt.stmt {
            StmtType::Expr(expr) => {
                self.analyse_expr(expr)?;
            }
            StmtType::VarDecl { name, value, ty } => {
                if let ValueType::UnknownType(name) = ty {
                    if !self.user_types.structs.contains_key(name as &str)
                        && !self.user_types.enums.contains_key(name as &str)
                    {
                        let err = SemErrType::UndefinedType(name.clone());
                        return Err(SemErr::new(line, err));
                    }
                }

                self.user_types.resolve_value_ty(ty);
                let value_ty = self.analyse_expr(value)?;

                if value_ty != *ty
                    && value_ty != ValueType::Null
                    && value_ty != ValueType::Any
                    && !try_coerce(&mut value.expr, ty)
                {
                    let err_ty = SemErrType::VarDeclTypeMismatch(ty.clone(), value_ty);
                    return Err(SemErr::new(line, err_ty));
                }

                self.symbols
                    .declare(Symbol::new(name.to_string(), ty.clone()), line)?;
            }
            StmtType::Println(expr) => {
                self.analyse_expr(expr)?;
            }
            StmtType::Return(expr) => {
                self.return_stmt_found = true;
                let return_ty = self.analyse_expr(expr)?;

                if let Some(expected_return_ty) = &self.current_return_ty {
                    if return_ty != *expected_return_ty
                        && return_ty != ValueType::Null
                        && !try_coerce(&mut expr.expr, expected_return_ty)
                    {
                        let err_ty =
                            SemErrType::IncorrectReturnTy(expected_return_ty.clone(), return_ty);
                        return Err(SemErr::new(line, err_ty));
                    }
                }
            }
            StmtType::Block(stmts) => {
                self.symbols.begin_scope();
                for stmt in stmts {
                    self.analyse_stmt(stmt)?;
                }
                self.symbols.end_scope();
            }
            StmtType::If {
                condition,
                body,
                final_else,
            } => {
                let condition_ty = self.analyse_expr(condition)?;
                if condition_ty != ValueType::Bool {
                    let err_ty = SemErrType::InvalidIfCondition(condition_ty);
                    return Err(SemErr::new(line, err_ty));
                }

                self.analyse_stmt(body)?;
                if let Some(final_else) = final_else {
                    self.analyse_stmt(final_else)?;
                }
            }
            StmtType::While { condition, body } => {
                let condition_ty = self.analyse_expr(condition)?;
                if condition_ty != ValueType::Bool {
                    let err_ty = SemErrType::InvalidWhileCondition(condition_ty);
                    return Err(SemErr::new(line, err_ty));
                }

                self.analyse_stmt(body)?;
            }
            StmtType::For {
                var,
                condition,
                body,
            } => {
                self.symbols.begin_scope();
                self.analyse_stmt(var)?;
                self.analyse_expr(condition)?;
                self.analyse_stmt(body)?;
                self.symbols.end_scope();
            }
            StmtType::Func {
                name,
                parameters,
                body,
                return_ty,
                use_self,
            } => {
                self.analyse_func_stmt(return_ty.clone(), parameters, line, body, name, *use_self)?;
            }
            StmtType::Break => (),
            StmtType::Continue => (),
            StmtType::Struct { .. } => (),
            StmtType::Enum { .. } => (),
        };
        Ok(())
    }

    fn analyse_func_stmt(
        &mut self,
        return_ty: ValueType,
        parameters: &mut Vec<(ValueType, String)>,
        line: u32,
        body: &mut [Stmt],
        name: &str,
        use_self: bool,
    ) -> Result<(), SemErr> {
        if self.current_return_ty.is_some() {
            let ty = SemErrType::FuncDefInFunc(name.to_string());
            return Err(SemErr::new(line, ty));
        }

        let prev_use_self = self.current_use_self;
        let return_ty_is_null = return_ty == ValueType::Null;

        self.current_return_ty = Some(return_ty);
        self.current_use_self = use_self;

        self.symbols.begin_scope();

        for (ty, name) in parameters {
            self.user_types.resolve_value_ty(ty);
            self.symbols
                .declare(Symbol::new(name.to_string(), ty.clone()), line)?;
        }
        self.return_stmt_found = false;

        for stmt in body.iter_mut() {
            self.analyse_stmt(stmt)?;
        }

        if let Some(func) = self.user_types.funcs.get_mut(name) {
            func.body = body.to_owned();
        }

        if !return_ty_is_null && !self.return_stmt_found {
            let ty =
                SemErrType::NoReturnTy(name.to_string(), self.current_return_ty.clone().unwrap());
            return Err(SemErr::new(line, ty));
        }

        self.symbols.end_scope();
        self.current_return_ty = None;
        self.current_use_self = prev_use_self;

        Ok(())
    }
}

// TODO: remove coercing
fn try_coerce(expr: &mut ExprType, target: &ValueType) -> bool {
    match expr {
        ExprType::Lit(lit) => match (&lit, target) {
            (Literal::I64(n), ValueType::U64) => {
                *lit = Literal::U64(*n as u64);
                true
            }
            _ => false,
        },
        ExprType::Binary { left, right, .. } => {
            try_coerce(&mut left.expr, target) && try_coerce(&mut right.expr, target)
        }
        _ => false,
    }
}
