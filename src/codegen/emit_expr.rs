use inkwell::builder::BuilderError;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

use crate::expression::{Expr, ExprType};
use crate::value::ValueType;

use super::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    pub(super) fn emit_expr(&self, expr: &Expr) -> Result<BasicValueEnum, BuilderError> {
        dbg!(&expr.expr);
        use crate::token::Literal;
        match &expr.expr {
            ExprType::Cast { value, target_ty } => self.emit_cast(value, target_ty),
            ExprType::Identifier(name) => {
                let (ptr, ty) = self.find_var(name).unwrap();
                let x = self.to_llvm_type(ty);
                self.builder.build_load(x, *ptr, name)
            }
            ExprType::Assign { name, new_value } => {
                let (ptr, _) = self.find_var(name).unwrap();
                let val = self.emit_expr(new_value)?;

                self.builder.build_store(*ptr, val)?;

                Ok(val)
            }
            ExprType::Lit(literal) => {
                let result = match literal {
                    Literal::I64(n) => self.context.i64_type().const_int(*n as u64, true).into(),
                    Literal::U64(n) => self.context.i64_type().const_int(*n, false).into(),
                    Literal::F64(n) => self.context.f64_type().const_float(*n).into(),
                    Literal::True => self.context.bool_type().const_int(1, false).into(),
                    Literal::False => self.context.bool_type().const_int(0, false).into(),
                    Literal::Str(string) => self.emit_str_expr(string)?,
                    Literal::Null => self.context.ptr_type(AddressSpace::default()).const_null().into(),
                    _ => todo!(),
                };
                Ok(result)
            }
            ExprType::Binary { left, op, right } => self.emit_binary_expr(left, op, right),
            ExprType::Unary { prefix: _, value } => self.emit_unary_expr(value),
            ExprType::FuncCall {
                name,
                args,
                index: _,
            } => {
                let func = self.module.get_function(name).unwrap();

                let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum> = args
                    .iter()
                    .map(|expr| self.emit_expr(expr).map(|v| v.into()))
                    .collect::<Result<Vec<_>, _>>()?;

                let call_site = self.builder.build_call(func, &llvm_args, "calltmp")?;

                // if function returns void
                // TODO: void functions crash i think
                if func.get_type().get_return_type().is_none() {
                    unreachable!()
                } else {
                    Ok(call_site
                        .try_as_basic_value()
                        .expect_basic("to basic value"))
                }
            }
            _ => todo!(),
        }
    }

    fn emit_str_expr(&self, string: &String) -> Result<BasicValueEnum<'_>, BuilderError> {
        let global = self
            .builder
            .build_global_string_ptr(&string, "string_lit")?;

        let len = self
            .context
            .i64_type()
            .const_int(string.len() as u64, false);

        let str_ty = self.string_type();

        Ok(str_ty
            .const_named_struct(&[global.as_pointer_value().into(), len.into()])
            .into())
    }

    fn emit_unary_expr(&self, value: &Box<Expr>) -> Result<BasicValueEnum<'_>, BuilderError> {
        let result = match value.end_ty {
            ValueType::Bool => {
                let value = self.emit_expr(value)?.into_int_value();
                self.builder.build_not(value, "nottmp")?.into()
            }
            ValueType::I64 => {
                let value = self.emit_expr(value)?.into_int_value();
                self.builder.build_int_neg(value, "negtmp")?.into()
            }
            ValueType::F64 => {
                let value = self.emit_expr(value)?.into_float_value();
                self.builder.build_float_neg(value, "fnegtmp")?.into()
            }
            _ => unreachable!(),
        };
        Ok(result)
    }

    fn emit_binary_expr(
        &self,
        left: &Box<Expr>,
        op: &crate::binary_op::BinaryOp,
        right: &Box<Expr>,
    ) -> Result<BasicValueEnum<'_>, BuilderError> {
        use crate::binary_op::BinaryOp;

        let result = match left.end_ty {
            ValueType::Bool => {
                let left = self.emit_expr(left)?.into_int_value();
                let right = self.emit_expr(right)?.into_int_value();
                use inkwell::IntPredicate::*;

                match op {
                    BinaryOp::Equal => self
                        .builder
                        .build_int_compare(EQ, left, right, "eqtmp")?
                        .into(),
                    BinaryOp::NotEqual => self
                        .builder
                        .build_int_compare(NE, left, right, "neqtmp")?
                        .into(),
                    BinaryOp::And => self.builder.build_and(left, right, "andtmp")?.into(),
                    BinaryOp::Or => self.builder.build_or(left, right, "ortmp")?.into(),
                    _ => unreachable!(),
                }
            }
            ValueType::I64 => {
                let left = self.emit_expr(left)?.into_int_value();
                let right = self.emit_expr(right)?.into_int_value();
                use inkwell::IntPredicate::*;

                match op {
                    BinaryOp::Add => self.builder.build_int_add(left, right, "addtmp")?.into(),
                    BinaryOp::Sub => self.builder.build_int_sub(left, right, "subtmp")?.into(),
                    BinaryOp::Mul => self.builder.build_int_mul(left, right, "multmp")?.into(),
                    BinaryOp::Div => self
                        .builder
                        .build_int_signed_div(left, right, "divtmp")?
                        .into(),
                    BinaryOp::Equal => self
                        .builder
                        .build_int_compare(EQ, left, right, "eqtmp")?
                        .into(),
                    BinaryOp::NotEqual => self
                        .builder
                        .build_int_compare(NE, left, right, "neqtmp")?
                        .into(),
                    BinaryOp::Less => self
                        .builder
                        .build_int_compare(SLT, left, right, "slttmp")?
                        .into(),
                    BinaryOp::LessEqual => self
                        .builder
                        .build_int_compare(SLE, left, right, "sletmp")?
                        .into(),
                    BinaryOp::Greater => self
                        .builder
                        .build_int_compare(SGT, left, right, "sgttmp")?
                        .into(),
                    BinaryOp::GreaterEqual => self
                        .builder
                        .build_int_compare(SGE, left, right, "sgetmp")?
                        .into(),
                    _ => unreachable!(),
                }
            }
            ValueType::F64 => {
                let left = self.emit_expr(left)?.into_float_value();
                let right = self.emit_expr(right)?.into_float_value();
                use inkwell::FloatPredicate::*;

                match op {
                    BinaryOp::Add => self.builder.build_float_add(left, right, "addtmp")?.into(),
                    BinaryOp::Sub => self.builder.build_float_sub(left, right, "subtmp")?.into(),
                    BinaryOp::Mul => self.builder.build_float_mul(left, right, "multmp")?.into(),
                    BinaryOp::Div => self.builder.build_float_div(left, right, "divtmp")?.into(),
                    BinaryOp::Equal => self
                        .builder
                        .build_float_compare(OEQ, left, right, "eqtmp")?
                        .into(),
                    BinaryOp::NotEqual => self
                        .builder
                        .build_float_compare(ONE, left, right, "neqtmp")?
                        .into(),
                    BinaryOp::Less => self
                        .builder
                        .build_float_compare(OLT, left, right, "slttmp")?
                        .into(),
                    BinaryOp::LessEqual => self
                        .builder
                        .build_float_compare(OLE, left, right, "sletmp")?
                        .into(),
                    BinaryOp::Greater => self
                        .builder
                        .build_float_compare(OGT, left, right, "sgttmp")?
                        .into(),
                    BinaryOp::GreaterEqual => self
                        .builder
                        .build_float_compare(OGE, left, right, "sgetmp")?
                        .into(),
                    _ => unreachable!(),
                }
            }
            ValueType::U64 => {
                let left = self.emit_expr(left)?.into_int_value();
                let right = self.emit_expr(right)?.into_int_value();
                use inkwell::IntPredicate::*;

                match op {
                    BinaryOp::Add => self.builder.build_int_add(left, right, "addtmp")?.into(),
                    BinaryOp::Sub => self.builder.build_int_sub(left, right, "subtmp")?.into(),
                    BinaryOp::Mul => self.builder.build_int_mul(left, right, "multmp")?.into(),
                    BinaryOp::Div => self
                        .builder
                        .build_int_unsigned_div(left, right, "divtmp")?
                        .into(),
                    BinaryOp::Equal => self
                        .builder
                        .build_int_compare(EQ, left, right, "eqtmp")?
                        .into(),
                    BinaryOp::NotEqual => self
                        .builder
                        .build_int_compare(NE, left, right, "neqtmp")?
                        .into(),
                    BinaryOp::Less => self
                        .builder
                        .build_int_compare(ULT, left, right, "ulttmp")?
                        .into(),
                    BinaryOp::LessEqual => self
                        .builder
                        .build_int_compare(ULE, left, right, "uletmp")?
                        .into(),
                    BinaryOp::Greater => self
                        .builder
                        .build_int_compare(UGT, left, right, "ugttmp")?
                        .into(),
                    BinaryOp::GreaterEqual => self
                        .builder
                        .build_int_compare(UGE, left, right, "ugetmp")?
                        .into(),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };
        Ok(result)
    }

    fn emit_cast(
        &self,
        value: &Box<Expr>,
        target_ty: &ValueType,
    ) -> Result<BasicValueEnum<'_>, BuilderError> {
        let source_ty = value.end_ty.clone();
        let val = self.emit_expr(value)?;

        let result = match (source_ty, target_ty) {
            (ValueType::I64, ValueType::F64) => self
                .builder
                .build_signed_int_to_float(
                    val.into_int_value(),
                    self.context.f64_type(),
                    "cast_i64_f64",
                )?
                .into(),

            (ValueType::U64, ValueType::F64) => self
                .builder
                .build_unsigned_int_to_float(
                    val.into_int_value(),
                    self.context.f64_type(),
                    "cast_u64_f64",
                )?
                .into(),

            (ValueType::Bool, ValueType::F64) => self
                .builder
                .build_unsigned_int_to_float(
                    val.into_int_value(),
                    self.context.f64_type(),
                    "cast_bool_f64",
                )?
                .into(),

            (ValueType::F64, ValueType::I64) => self
                .builder
                .build_float_to_signed_int(
                    val.into_float_value(),
                    self.context.i64_type(),
                    "cast_f64_i64",
                )?
                .into(),

            (ValueType::F64, ValueType::U64) => self
                .builder
                .build_float_to_unsigned_int(
                    val.into_float_value(),
                    self.context.i64_type(),
                    "cast_f64_u64",
                )?
                .into(),

            (ValueType::F64, ValueType::Bool) => {
                let zero = self.context.f64_type().const_float(0.0);
                self.builder
                    .build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        val.into_float_value(),
                        zero,
                        "cast_f64_bool",
                    )?
                    .into()
            }

            // between integer types (I64, U64, Bool), same bit width, just reinterpret
            (ValueType::I64, ValueType::U64) | (ValueType::U64, ValueType::I64) => val,

            (ValueType::I64, ValueType::Bool) | (ValueType::U64, ValueType::Bool) => {
                let zero = self.context.i64_type().const_int(0, false);
                self.builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        val.into_int_value(),
                        zero,
                        "cast_int_bool",
                    )?
                    .into()
            }

            (ValueType::Bool, ValueType::I64) | (ValueType::Bool, ValueType::U64) => self
                .builder
                .build_int_z_extend(
                    val.into_int_value(),
                    self.context.i64_type(),
                    "cast_bool_int",
                )?
                .into(),

            _ => unimplemented!(),
        };
        Ok(result)
    }
}
