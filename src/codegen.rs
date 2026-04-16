use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, PointerValue};

use std::collections::HashMap;
use std::error::Error;

use crate::expression::{Expr, ExprType};
use crate::statement::{Stmt, StmtType};
use crate::value::ValueType;

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type MainFunc = unsafe extern "C" fn() -> i64;

#[unsafe(no_mangle)]
pub extern "C" fn print_i64(x: i64) {
    println!("{}", x);
}
#[unsafe(no_mangle)]
pub extern "C" fn print_f64(x: f64) {
    println!("{}", x);
}

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,

    alloc_builder: Builder<'ctx>,
    declared_vars: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn compile(stmts: Vec<Stmt>) -> Result<(), Box<dyn Error>> {
        let context = Context::create();
        let module = context.create_module("program");
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
        let mut codegen = CodeGen {
            context: &context,
            module,
            builder: context.create_builder(),
            alloc_builder: context.create_builder(),
            execution_engine,
            declared_vars: HashMap::new(),
        };

        codegen.build_main(stmts)?;

        let print_i64_fn = codegen.module.get_function("print_i64").unwrap();
        codegen
            .execution_engine
            .add_global_mapping(&print_i64_fn, print_i64 as usize);

        let print_f64_fn = codegen.module.get_function("print_f64").unwrap();
        codegen
            .execution_engine
            .add_global_mapping(&print_f64_fn, print_f64 as usize);

        let main: JitFunction<MainFunc> =
            unsafe { codegen.execution_engine.get_function("main").ok() }
                .ok_or("Unable to get JIT function")?;

        codegen.module.print_to_stderr();

        unsafe {
            println!("main returns '{}'", main.call());
        }

        Ok(())
    }

    fn build_main(&mut self, mut stmts: Vec<Stmt>) -> Result<(), Box<dyn Error>> {
        // Declare external print_i64
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();

        let print_type = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("print_i64", print_type, None);
        let print_f64_type = void_type.fn_type(&[self.context.f64_type().into()], false);
        self.module.add_function("print_f64", print_f64_type, None);

        // Create main function
        let fn_type = self.context.i64_type().fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);

        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);
        self.alloc_builder.position_at_end(basic_block);

        for stmt in &mut stmts {
            self.emit_stmt(stmt);
        }

        self.builder
            .build_return(Some(&self.context.i64_type().const_int(0, false)))?;

        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.stmt {
            StmtType::While { condition, body } => {
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let body_block = self.context.append_basic_block(function, "while_body");
                let condition_block = self.context.append_basic_block(function, "while_condition");
                let end_block = self.context.append_basic_block(function, "while_end");

                self.builder
                    .build_unconditional_branch(condition_block)
                    .unwrap();

                // condition block
                self.builder.position_at_end(condition_block);
                let condition = self.emit_expr(condition).into_int_value();

                self.builder
                    .build_conditional_branch(condition, body_block, end_block)
                    .unwrap();

                // body block
                self.builder.position_at_end(body_block);
                self.emit_stmt(&body);

                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder
                        .build_unconditional_branch(condition_block)
                        .unwrap();
                }

                // end block
                self.builder.position_at_end(end_block);
            }
            StmtType::If {
                condition,
                body,
                final_else,
            } => {
                let condition = self.emit_expr(condition).into_int_value();
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
                    .build_conditional_branch(condition, then_block, else_block)
                    .unwrap();

                self.builder.position_at_end(then_block);
                self.emit_stmt(&body);
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(end_block).unwrap();
                }

                self.builder.position_at_end(else_block);

                if let Some(else_stmt) = final_else {
                    self.emit_stmt(else_stmt);
                }

                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(end_block).unwrap();
                }

                self.builder.position_at_end(end_block);
            }
            StmtType::Block(stmts) => {
                for stmt in stmts {
                    self.emit_stmt(stmt);
                }
            }
            StmtType::Expr(expr) => {
                let _ = self.emit_expr(expr);
                // self.builder.build_return(Some(&e)).unwrap();
            }
            StmtType::VarDecl { name, value, ty: _ } => {
                let ty = self.context.i64_type();
                let ptr = self.alloc_builder.build_alloca(ty, &name).unwrap();

                let value = self.emit_expr(value);
                self.alloc_builder.build_store(ptr, value).unwrap();
                self.declared_vars.insert(name.to_string(), ptr);
            }
            StmtType::Println(expr) => {
                let value = self.emit_expr(expr);
                let print_fn = match expr.end_ty {
                    ValueType::Null => todo!(),
                    ValueType::Bool => self.module.get_function("print_i64").unwrap(),
                    ValueType::F64 => self.module.get_function("print_f64").unwrap(),
                    ValueType::I64 => self.module.get_function("print_i64").unwrap(),
                    ValueType::U64 => todo!(),
                    ValueType::Str => todo!(),
                    _ => unreachable!(),
                };
                self.module.get_function("print_i64").unwrap();

                self.builder
                    .build_call(print_fn, &[value.into()], "printtmp")
                    .unwrap();
            }

            StmtType::Func {
                name: _,
                parameters: _,
                body,
                return_ty: _,
                use_self: _,
            } => {
                for stmt in body {
                    self.emit_stmt(stmt);
                }
            }
            _ => todo!(),
        }
    }
    fn emit_expr(&self, expr: &Expr) -> BasicValueEnum {
        // dbg!(&expr.expr);
        use crate::token::Literal;
        // let end_ty = expr.end_ty.clone();
        match &expr.expr {
            ExprType::Identifier(name) => {
                let ptr = self.declared_vars.get(name).unwrap();
                self.builder
                    .build_load(self.context.i64_type(), *ptr, name)
                    .unwrap()
            }
            ExprType::Assign { name, new_value } => {
                let ptr = self.declared_vars.get(name).unwrap();
                let val = self.emit_expr(new_value).into_int_value();

                self.builder.build_store(*ptr, val).unwrap();

                val.into()
            }
            ExprType::Lit(literal) => match literal {
                Literal::I64(n) => self.context.i64_type().const_int(*n as u64, false).into(),
                // Literal::U64(n) => self.context.u64_type().const_int(*n as u64, false).into(),
                Literal::F64(n) => self.context.f64_type().const_float(*n).into(),
                Literal::True => self.context.bool_type().const_int(1, false).into(),
                Literal::False => self.context.bool_type().const_int(0, false).into(),
                _ => todo!(),
            },
            ExprType::Binary { left, op, right } => {
                use crate::parse_types::BinaryOp;
                // use inkwell::IntPredicate::*;
                // use inkwell::FloatPredicate::*;

                let result: BasicValueEnum = match left.end_ty {
                    ValueType::Bool => {
                        let left = self.emit_expr(left).into_int_value();
                        let right = self.emit_expr(right).into_int_value();
                        use inkwell::IntPredicate::*;

                        match op {
                            BinaryOp::Equal => self
                                .builder
                                .build_int_compare(EQ, left, right, "eqtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::NotEqual => self
                                .builder
                                .build_int_compare(NE, left, right, "neqtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::And => self
                                .builder
                                .build_and(left, right, "andtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Or => {
                                self.builder.build_or(left, right, "ortmp").unwrap().into()
                            }
                            _ => unreachable!(),
                        }
                    }
                    ValueType::I64 => {
                        let left = self.emit_expr(left).into_int_value();
                        let right = self.emit_expr(right).into_int_value();
                        use inkwell::IntPredicate::*;

                        match op {
                            BinaryOp::Add => self
                                .builder
                                .build_int_add(left, right, "addtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Sub => self
                                .builder
                                .build_int_sub(left, right, "subtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Mul => self
                                .builder
                                .build_int_mul(left, right, "multmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Div => self
                                .builder
                                .build_int_signed_div(left, right, "divtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Equal => self
                                .builder
                                .build_int_compare(EQ, left, right, "eqtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::NotEqual => self
                                .builder
                                .build_int_compare(NE, left, right, "neqtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Less => self
                                .builder
                                .build_int_compare(SLT, left, right, "slttmp")
                                .unwrap()
                                .into(),
                            BinaryOp::LessEqual => self
                                .builder
                                .build_int_compare(SLE, left, right, "sletmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Greater => self
                                .builder
                                .build_int_compare(SGT, left, right, "sgttmp")
                                .unwrap()
                                .into(),
                            BinaryOp::GreaterEqual => self
                                .builder
                                .build_int_compare(SGE, left, right, "sgetmp")
                                .unwrap()
                                .into(),
                            _ => unreachable!(),
                        }
                    }
                    ValueType::F64 => {
                        let left = self.emit_expr(left).into_float_value();
                        let right = self.emit_expr(right).into_float_value();
                        use inkwell::FloatPredicate::*;

                        match op {
                            BinaryOp::Add => self
                                .builder
                                .build_float_add(left, right, "addtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Sub => self
                                .builder
                                .build_float_sub(left, right, "subtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Mul => self
                                .builder
                                .build_float_mul(left, right, "multmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Div => self
                                .builder
                                .build_float_div(left, right, "divtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Equal => self
                                .builder
                                .build_float_compare(OEQ, left, right, "eqtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::NotEqual => self
                                .builder
                                .build_float_compare(ONE, left, right, "neqtmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Less => self
                                .builder
                                .build_float_compare(OLT, left, right, "slttmp")
                                .unwrap()
                                .into(),
                            BinaryOp::LessEqual => self
                                .builder
                                .build_float_compare(OLE, left, right, "sletmp")
                                .unwrap()
                                .into(),
                            BinaryOp::Greater => self
                                .builder
                                .build_float_compare(OGT, left, right, "sgttmp")
                                .unwrap()
                                .into(),
                            BinaryOp::GreaterEqual => self
                                .builder
                                .build_float_compare(OGE, left, right, "sgetmp")
                                .unwrap()
                                .into(),
                            _ => unreachable!(),
                        }
                    }
                    crate::value::ValueType::U64 => todo!(),
                    _ => unreachable!(),
                };
                result
            }
            _ => todo!(),
        }
    }
}
