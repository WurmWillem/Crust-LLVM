use super::CodeGen;
use inkwell::builder::BuilderError;

use std::collections::HashMap;

use crate::expression::Expr;
use crate::statement::{Stmt, StmtType};
use crate::value::ValueType;

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn emit_stmt(&mut self, stmt: &Stmt) -> Result<(), BuilderError> {
        // dbg!(&stmt.stmt);
        match &stmt.stmt {
            StmtType::While { condition, body } => {
                self.emit_while_stmt(condition, body)?;
            }
            StmtType::For {
                var,
                condition,
                body,
            } => {
                self.emit_for_stmt(var, condition, body)?;
            }
            StmtType::If {
                condition,
                body,
                final_else,
            } => {
                self.emit_if_stmt(condition, body, final_else)?;
            }
            StmtType::Block(stmts) => {
                self.declared_vars.push(HashMap::new());
                for stmt in stmts {
                    self.emit_stmt(stmt)?;
                }
                self.declared_vars.pop().unwrap();
            }
            StmtType::Expr(expr) => {
                let _ = self.emit_expr(expr);
            }
            StmtType::VarDecl { name, value, ty } => {
                let llvm_ty = self.to_llvm_type(ty);
                let ptr = self.alloc_builder.build_alloca(llvm_ty, &name)?;

                let value = self.emit_expr(value)?;
                self.builder.build_store(ptr, value)?;
                self.declared_vars
                    .last_mut()
                    .unwrap()
                    .insert(name.to_string(), (ptr, ty.clone()));
            }
            StmtType::Println(expr) => {
                let value = self.emit_expr(expr)?;
                let print_fn = match expr.end_ty {
                    ValueType::Null => todo!(),
                    ValueType::Bool => self.module.get_function("print_i64").unwrap(),
                    ValueType::F64 => self.module.get_function("print_f64").unwrap(),
                    ValueType::I64 => self.module.get_function("print_i64").unwrap(),
                    ValueType::U64 => self.module.get_function("print_u64").unwrap(),
                    ValueType::Str => self.module.get_function("print_str").unwrap(),
                    _ => unreachable!(),
                };

                self.builder
                    .build_call(print_fn, &[value.into()], "printtmp")?;
            }

            StmtType::Return(expr) => {
                let value = self.emit_expr(expr)?;
                self.builder.build_return(Some(&value))?;
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    fn emit_if_stmt(
        &mut self,
        condition: &Expr,
        body: &Box<Stmt>,
        final_else: &Option<Box<Stmt>>,
    ) -> Result<(), BuilderError> {
        let condition = self.emit_expr(condition)?.into_int_value();
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let end_block = self.context.append_basic_block(function, "if_end");

        self.builder
            .build_conditional_branch(condition, then_block, else_block)?;

        self.builder.position_at_end(then_block);
        self.emit_stmt(&body)?;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(end_block)?;
        }

        self.builder.position_at_end(else_block);

        if let Some(else_stmt) = final_else {
            self.emit_stmt(else_stmt)?;
        }

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(end_block)?;
        }

        self.builder.position_at_end(end_block);
        Ok(())
    }

    fn emit_for_stmt(
        &mut self,
        var: &Box<Stmt>,
        condition: &Expr,
        body: &Box<Stmt>,
    ) -> Result<(), BuilderError> {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        self.emit_stmt(var)?;

        let condition_block = self.context.append_basic_block(function, "for_condition");
        let body_block = self.context.append_basic_block(function, "for_body");
        let end_block = self.context.append_basic_block(function, "for_end");

        self.builder.build_unconditional_branch(condition_block)?;

        // condition block
        self.builder.position_at_end(condition_block);
        let condition = self.emit_expr(condition)?.into_int_value();
        self.builder
            .build_conditional_branch(condition, body_block, end_block)?;

        // body block
        self.builder.position_at_end(body_block);
        self.emit_stmt(&body)?;

        let name = if let StmtType::VarDecl { name, .. } = &var.stmt {
            name
        } else {
            unreachable!();
        };
        let (i_ptr, _) = self.declared_vars.last().unwrap().get(name).unwrap();
        let x = self
            .builder
            .build_load(self.context.i64_type(), *i_ptr, "load_i")?
            .into_int_value();
        let one = self.context.i64_type().const_int(1, false);
        let new_i = self.builder.build_int_add(x, one, "add_i_tmp")?;
        self.builder.build_store(*i_ptr, new_i)?;

        // let stmt = Sm

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(condition_block)?;
        }

        // end block
        self.builder.position_at_end(end_block);
        Ok(())
    }

    fn emit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) -> Result<(), BuilderError> {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        let body_block = self.context.append_basic_block(function, "while_body");
        let condition_block = self.context.append_basic_block(function, "while_condition");
        let end_block = self.context.append_basic_block(function, "while_end");

        self.builder.build_unconditional_branch(condition_block)?;

        // condition block
        self.builder.position_at_end(condition_block);
        let condition = self.emit_expr(condition)?.into_int_value();

        self.builder
            .build_conditional_branch(condition, body_block, end_block)?;

        // body block
        self.builder.position_at_end(body_block);
        self.emit_stmt(&body)?;

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(condition_block)?;
        }

        // end block
        self.builder.position_at_end(end_block);
        Ok(())
    }
}
