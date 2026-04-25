use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, StructType};
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::{AddressSpace, OptimizationLevel};

use std::collections::HashMap;
use std::error::Error;

use crate::analysis_types::UserTypes;
use crate::c_funcs::*;
use crate::expression::{Expr, ExprType};
use crate::statement::{Stmt, StmtType};
use crate::value::ValueType;

/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.

pub type MainFunc = unsafe extern "C" fn() -> i64;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,

    alloc_builder: Builder<'ctx>,
    declared_vars: HashMap<String, (PointerValue<'ctx>, ValueType)>,

    user_types: UserTypes,
}
impl<'ctx> CodeGen<'ctx> {
    pub fn compile(stmts: Vec<Stmt>, user_types: UserTypes) -> Result<(), Box<dyn Error>> {
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
            user_types,
        };

        codegen.build_main(stmts)?;

        let print_i64_fn = codegen.module.get_function("print_i64").unwrap();
        codegen
            .execution_engine
            .add_global_mapping(&print_i64_fn, print_i64 as usize);

        let print_u64_fn = codegen.module.get_function("print_u64").unwrap();
        codegen
            .execution_engine
            .add_global_mapping(&print_u64_fn, print_u64 as usize);

        let print_f64_fn = codegen.module.get_function("print_f64").unwrap();
        codegen
            .execution_engine
            .add_global_mapping(&print_f64_fn, print_f64 as usize);

        let print_str_fn = codegen.module.get_function("print_str").unwrap();
        codegen
            .execution_engine
            .add_global_mapping(&print_str_fn, print_str as usize);

        let main: JitFunction<MainFunc> =
            unsafe { codegen.execution_engine.get_function("main").ok() }
                .ok_or("Unable to get JIT function")?;

        codegen.module.print_to_stderr();

        let start = std::time::Instant::now();
        unsafe {
            main.call();
        }
        println!("{:?}", start.elapsed());
        // unsafe {
        //     println!("main returns '{}'", main.call());
        // }

        Ok(())
    }

    fn make_fn_type(
        &self,
        ret: &ValueType,
        params: &Vec<(ValueType, String)>,
    ) -> FunctionType<'ctx> {
        let params: Vec<BasicMetadataTypeEnum<'ctx>> = params
            .iter()
            .map(|(ty, _)| self.to_llvm_type(ty).into())
            .collect();

        match ret {
            ValueType::Null => self.context.void_type().fn_type(&params, false),

            _ => match self.to_llvm_type(ret) {
                BasicTypeEnum::IntType(t) => t.fn_type(&params, false),
                BasicTypeEnum::FloatType(t) => t.fn_type(&params, false),
                BasicTypeEnum::PointerType(t) => t.fn_type(&params, false),
                BasicTypeEnum::StructType(t) => t.fn_type(&params, false),
                BasicTypeEnum::ArrayType(t) => t.fn_type(&params, false),
                BasicTypeEnum::VectorType(t) => t.fn_type(&params, false),
                BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&params, false),
            },
        }
    }

    fn build_funcs(&mut self) -> Result<(), Box<dyn Error>> {
        let mut funcs = std::mem::take(&mut self.user_types.funcs);

        for (name, data) in funcs.iter_mut() {
            // data.return_ty.
            let fn_type = self.make_fn_type(&data.return_ty, &data.parameters);

            let func = self.module.add_function(&name, fn_type, None);

            let block = self.context.append_basic_block(func, "entry");
            self.builder.position_at_end(block);
            self.alloc_builder.position_at_end(block);

            for stmt in &mut data.body {
                self.emit_stmt(&stmt);
            }
            self.builder
                .build_return(Some(&self.context.i64_type().const_int(0, false)))?;
        }

        self.user_types.funcs = funcs;
        Ok(())
    }

    fn build_main(&mut self, mut stmts: Vec<Stmt>) -> Result<(), Box<dyn Error>> {
        // declare external print_i64
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();
        let ptr_ty = self.context.ptr_type(AddressSpace::default());

        let print_i64_type = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("print_i64", print_i64_type, None);
        let print_u64_type = void_type.fn_type(&[self.context.i64_type().into()], false);
        self.module.add_function("print_u64", print_u64_type, None);
        let print_f64_type = void_type.fn_type(&[self.context.f64_type().into()], false);
        self.module.add_function("print_f64", print_f64_type, None);

        let print_str_type = void_type.fn_type(&[ptr_ty.into(), i64_type.into()], false);
        self.module.add_function("print_str", print_str_type, None);

        // create main function
        self.build_funcs()?;
        // let fn_type = self.context.i64_type().fn_type(&[], false);
        // let function = self.module.add_function("main", fn_type, None);
        //
        // let basic_block = self.context.append_basic_block(function, "entry");
        // self.builder.position_at_end(basic_block);
        // self.alloc_builder.position_at_end(basic_block);

        // for stmt in &mut stmts {
        //     self.emit_stmt(stmt);
        // }
        //
        // self.builder
        //     .build_return(Some(&self.context.i64_type().const_int(0, false)))?;

        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        // dbg!(stmt);
        match &stmt.stmt {
            StmtType::While { condition, body } => {
                self.emit_while_stmt(condition, body);
            }
            StmtType::For {
                var,
                condition,
                body,
            } => {
                self.emit_for_stmt(var, condition, body);
            }
            StmtType::If {
                condition,
                body,
                final_else,
            } => {
                self.emit_if_stmt(condition, body, final_else);
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
            StmtType::VarDecl { name, value, ty } => {
                let llvm_ty = self.to_llvm_type(ty);
                let ptr = self.alloc_builder.build_alloca(llvm_ty, &name).unwrap();

                let value = self.emit_expr(value);
                self.alloc_builder.build_store(ptr, value).unwrap();
                self.declared_vars
                    .insert(name.to_string(), (ptr, ty.clone()));
            }
            StmtType::Println(expr) => {
                let value = self.emit_expr(expr);
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
                todo!();
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
            ExprType::Cast { value, target: _ } => {
                // TODO: make this actually cast
                self.emit_expr(value)
            }
            ExprType::Identifier(name) => {
                let (ptr, ty) = self.declared_vars.get(name).unwrap();
                let x = self.to_llvm_type(ty);
                self.builder.build_load(x, *ptr, name).unwrap()
            }
            ExprType::Assign { name, new_value } => {
                let (ptr, _) = self.declared_vars.get(name).unwrap();
                let val = self.emit_expr(new_value).into_int_value();

                self.builder.build_store(*ptr, val).unwrap();

                val.into()
            }
            ExprType::Lit(literal) => match literal {
                Literal::I64(n) => self.context.i64_type().const_int(*n as u64, true).into(),
                Literal::U64(n) => self.context.i64_type().const_int(*n, false).into(),
                Literal::F64(n) => self.context.f64_type().const_float(*n).into(),
                Literal::True => self.context.bool_type().const_int(1, false).into(),
                Literal::False => self.context.bool_type().const_int(0, false).into(),
                Literal::Str(string) => {
                    // self.emit_string_literal(text).into()
                    self.emit_str_expr(string)
                }
                _ => todo!(),
            },
            ExprType::Binary { left, op, right } => self.emit_binary_expr(left, op, right),
            _ => todo!(),
        }
    }

    fn string_type(&self) -> StructType<'ctx> {
        let i8_ptr = self.context.ptr_type(AddressSpace::default());
        self.context
            .struct_type(&[i8_ptr.into(), self.context.i64_type().into()], false)
    }

    fn to_llvm_type(&self, ty: &ValueType) -> BasicTypeEnum<'ctx> {
        match ty {
            ValueType::I64 => self.context.i64_type().into(),
            ValueType::U64 => self.context.i64_type().into(),
            ValueType::Bool => self.context.bool_type().into(),
            ValueType::F64 => self.context.f64_type().into(),
            ValueType::Str => self.string_type().into(),
            _ => todo!(),
        }
    }

    fn emit_str_expr(&self, string: &String) -> BasicValueEnum<'_> {
        let global = self
            .builder
            .build_global_string_ptr(&string, "string_lit")
            .unwrap();

        let len = self
            .context
            .i64_type()
            .const_int(string.len() as u64, false);

        let str_ty = self.string_type();

        str_ty
            .const_named_struct(&[global.as_pointer_value().into(), len.into()])
            .into()
    }

    fn emit_if_stmt(&mut self, condition: &Expr, body: &Box<Stmt>, final_else: &Option<Box<Stmt>>) {
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

    fn emit_for_stmt(&mut self, var: &Box<Stmt>, condition: &Expr, body: &Box<Stmt>) {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        self.emit_stmt(var);

        let condition_block = self.context.append_basic_block(function, "for_condition");
        let body_block = self.context.append_basic_block(function, "for_body");
        let end_block = self.context.append_basic_block(function, "for_end");

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

        let name = if let StmtType::VarDecl { name, .. } = &var.stmt {
            name
        } else {
            unreachable!();
        };
        let (i_ptr, _) = self.declared_vars.get(name).unwrap();
        let x = self
            .builder
            .build_load(self.context.i64_type(), *i_ptr, "load_i")
            .unwrap()
            .into_int_value();
        let one = self.context.i64_type().const_int(1, false);
        let new_i = self.builder.build_int_add(x, one, "add_i_tmp").unwrap();
        self.builder.build_store(*i_ptr, new_i).unwrap();

        // let stmt = Sm

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

    fn emit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) {
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

    fn emit_binary_expr(
        &self,
        left: &Box<Expr>,
        op: &crate::parse_types::BinaryOp,
        right: &Box<Expr>,
    ) -> BasicValueEnum<'_> {
        use crate::parse_types::BinaryOp;

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
                    BinaryOp::Or => self.builder.build_or(left, right, "ortmp").unwrap().into(),
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
            ValueType::U64 => {
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
                        .build_int_unsigned_div(left, right, "divtmp")
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
                        .build_int_compare(ULT, left, right, "ulttmp")
                        .unwrap()
                        .into(),
                    BinaryOp::LessEqual => self
                        .builder
                        .build_int_compare(ULE, left, right, "uletmp")
                        .unwrap()
                        .into(),
                    BinaryOp::Greater => self
                        .builder
                        .build_int_compare(UGT, left, right, "ugttmp")
                        .unwrap()
                        .into(),
                    BinaryOp::GreaterEqual => self
                        .builder
                        .build_int_compare(UGE, left, right, "ugetmp")
                        .unwrap()
                        .into(),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };
        result
    }
}
