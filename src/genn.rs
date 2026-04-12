use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::BasicValueEnum;

use std::error::Error;

use crate::expression::{Expr, ExprType};
use crate::statement::{Stmt, StmtType};

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;
type MainFunc = unsafe extern "C" fn() -> i64;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn compile(stmts: Vec<Stmt>) -> Result<(), Box<dyn Error>> {
        let context = Context::create();
        let module = context.create_module("program");
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
        let codegen = CodeGen {
            context: &context,
            module,
            builder: context.create_builder(),
            execution_engine,
        };

        // let sum = codege
        //     .jit_compile_sum()
        //     .ok_or("Unable to JIT compile `sum`")?;

        let main = codegen
            .compile_main(stmts)
            .ok_or("Unable to JIT compile `main`")?;

        let x = 1u64;
        let y = 2u64;
        let z = 3u64;

        unsafe {
            println!("main = {}", main.call());
            // assert_eq!(sum.call(x, y, z), x + y + z);
        }

        // unsafe {
        //     println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
        //     assert_eq!(sum.call(x, y, z), x + y + z);
        // }

        codegen.module.print_to_stderr();

        Ok(())
    }
    fn compile_main(&self, stmts: Vec<Stmt>) -> Option<JitFunction<'_, MainFunc>> {
        // fn compile_main(&self)  {
        let fn_type = self.context.i64_type().fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);

        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        for stmt in &stmts {
            self.emit_stmt(stmt);
        }

        // self.builder
        //     .build_return(Some(&self.context.i64_type().const_int(0, false)))
        //     .unwrap();
        unsafe { self.execution_engine.get_function("main").ok() }
    }

    fn emit_stmt(&self, stmt: &Stmt) {
        // dbg!(stmt);
        match &stmt.stmt {
            StmtType::Expr(expr) => {
                let e = self.emit_expr(expr);
                self.builder
                    .build_return(Some(&e))
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
        match &expr.expr {
            ExprType::Lit(literal) => match literal {
                crate::token::Literal::I64(n) => {
                    self.context.i64_type().const_int(*n as u64, false).into()
                }
                _ => todo!(),
            },
            ExprType::Binary { left, op, right } => {
                let left = self.emit_expr(left).into_int_value();
                let right = self.emit_expr(right).into_int_value();
                self.builder
                    .build_int_add(left, right, "addtmp")
                    .unwrap()
                    .into()
            }
            _ => todo!(),
        }
    }

    fn jit_compile_sum(&self) -> Option<JitFunction<'_, SumFunc>> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();
        let z = function.get_nth_param(2)?.into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum").unwrap();
        let sum = self.builder.build_int_add(sum, z, "sum").unwrap();

        self.builder.build_return(Some(&sum)).unwrap();

        unsafe { self.execution_engine.get_function("sum").ok() }
    }
}
