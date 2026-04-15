use inkwell::types::BasicType;
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

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type MainFunc = unsafe extern "C" fn() -> i64;

#[unsafe(no_mangle)]
pub extern "C" fn print_i64(x: i64) {
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

        let print_fn = codegen.module.get_function("print_i64").unwrap();
        codegen
            .execution_engine
            .add_global_mapping(&print_fn, print_i64 as usize);

        let main: JitFunction<MainFunc> =
            unsafe { codegen.execution_engine.get_function("main").ok() }
                .ok_or("Unable to get JIT function")?;

        codegen.module.print_to_stderr();

        unsafe {
            println!("main returns '{}'", main.call());
        }

        Ok(())
    }

    fn build_main(
        &mut self,
        mut stmts: Vec<Stmt>,
    ) -> Result<(), Box<dyn Error>> {
        // Declare external print_i64
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();
        let print_type = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("print_i64", print_type, None);


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
        // dbg!(stmt);
        match &stmt.stmt {
            StmtType::Expr(expr) => {
                let _ = self.emit_expr(expr);
                // self.builder.build_return(Some(&e)).unwrap();
            }
            StmtType::VarDecl { name, value, ty } => {
                let ty = self.context.i64_type();
                let ptr = self.alloc_builder.build_alloca(ty, &name).unwrap();

                let value = self.emit_expr(value);
                self.alloc_builder.build_store(ptr, value).unwrap();
                self.declared_vars.insert(name.to_string(), ptr);
            }
            StmtType::Println(expr) => {
                let value = self.emit_expr(expr);
                let print_fn = self.module.get_function("print_i64").unwrap();

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
        match &expr.expr {
            ExprType::Identifier(name) => {
                let ptr = self.declared_vars.get(name).unwrap();
                self.builder.build_load(self.context.i64_type(), *ptr, name).unwrap()
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
                let left = self.emit_expr(left).into_int_value();
                let right = self.emit_expr(right).into_int_value();

                use crate::parse_types::BinaryOp;
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
                    BinaryOp::And => self
                        .builder
                        .build_and(left, right, "andtmp")
                        .unwrap()
                        .into(),
                    BinaryOp::Or => self.builder.build_or(left, right, "ortmp").unwrap().into(),
                }
            }
            _ => todo!(),
        }
    }
}
