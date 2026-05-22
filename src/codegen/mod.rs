mod c_funcs;
mod emit_expr;
mod emit_stmt;

use crate::analysis::UserTypes;
use crate::codegen::c_funcs::*;
use crate::value::ValueType;

use inkwell::builder::{Builder, BuilderError};
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, StructType};
use inkwell::values::PointerValue;
use inkwell::{AddressSpace, OptimizationLevel};

use std::collections::HashMap;
use std::error::Error;

/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
pub type MainFunc = unsafe extern "C" fn();

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,

    alloc_builder: Builder<'ctx>,
    declared_vars: Vec<HashMap<String, (PointerValue<'ctx>, ValueType)>>,
    // struct_data: Vec<HashMap<String, BasicTypeEnum<'ctx>>>,
}
impl<'ctx> CodeGen<'ctx> {
    pub fn compile(user_types: UserTypes) -> Result<(), Box<dyn Error>> {
        let context = Context::create();
        let module = context.create_module("program");
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;

        let mut codegen = CodeGen {
            context: &context,
            module,
            builder: context.create_builder(),
            alloc_builder: context.create_builder(),
            execution_engine,
            declared_vars: vec![],
            // struct_data: vec![],
        };

        codegen.declare_structs(&user_types);
        codegen.define_structs(&user_types);

        codegen.declare_funcs(&user_types);
        codegen.define_funcs(user_types)?;

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

        if let Err(err) = codegen.module.verify() {
            codegen.module.print_to_stderr();
            panic!("LLVM verification failed: {err}");
        }
        codegen.module.print_to_stderr();

        // let start = std::time::Instant::now();
        unsafe {
            main.call();
        }
        // println!("{:?}", start.elapsed());

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

    fn declare_structs(&mut self, user_types: &UserTypes) {
        for (name, _) in user_types.structs.iter() {
            self.context.opaque_struct_type(name);
        }
    }

    fn define_structs(&mut self, user_types: &UserTypes) {
        for (name, data) in user_types.structs.iter() {
            let struct_ty = self.module.get_struct_type(name).unwrap();
            let fields: Vec<BasicTypeEnum> = data
                .fields
                .iter()
                .map(|(ty, _)| self.to_llvm_type(ty))
                .collect();
            struct_ty.set_body(&fields, false);
        }
    }

    fn declare_funcs(&mut self, user_types: &UserTypes) {
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

        for (name, data) in &user_types.funcs {
            // dbg!(&data.return_ty);
            let fn_type = self.make_fn_type(&data.return_ty, &data.parameters);
            if self.module.get_function(name).is_none() {
                self.module.add_function(name, fn_type, None);
            }
        }
    }

    fn define_funcs(&mut self, mut user_types: UserTypes) -> Result<(), BuilderError> {
        for (name, data) in user_types.funcs.iter_mut() {
            // dbg!(&data.return_ty);
            let func = self.module.get_function(name).unwrap();
            let block = self.context.append_basic_block(func, "entry");

            self.builder.position_at_end(block);
            self.alloc_builder.position_at_end(block);

            self.declared_vars.push(HashMap::new());
            // add parameters
            for (i, (ty, name)) in data.parameters.iter().enumerate() {
                let param = func.get_nth_param(i as u32).unwrap();

                let ptr = self
                    .alloc_builder
                    .build_alloca(self.to_llvm_type(ty), name)?;

                self.builder.build_store(ptr, param)?;

                self.declared_vars
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), (ptr, ty.clone()));
            }

            for stmt in &mut data.body {
                self.emit_stmt(&stmt)?;
            }

            self.declared_vars.pop().unwrap();

            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.builder.build_return(None)?;
            }
        }

        Ok(())
    }

    fn find_var(&self, name: &String) -> Option<&(PointerValue<'ctx>, ValueType)> {
        self.declared_vars
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
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
            ValueType::Struct(name) => self.context.get_struct_type(name).unwrap().into(),
            // TODO: remove
            ValueType::UnknownType(name) => self.context.get_struct_type(name).unwrap().into(),
            _ => todo!(),
        }
    }
}
