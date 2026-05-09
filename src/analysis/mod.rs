mod analyse_expr;
mod analyse_stmt;
mod types;

use crate::{
    analysis::types::{SemErr, SemErrType},
    statement::{Stmt, StmtType},
    value::ValueType,
};
use types::{FuncData, SemanticScope, StructData};

pub use types::UserTypes;

pub struct Analyser {
    user_types: UserTypes,
    symbols: SemanticScope,
    current_return_ty: Option<ValueType>,
    current_use_self: bool,
    return_stmt_found: bool,
    current_struct: Option<String>,
}
impl Analyser {
    fn new() -> Self {
        Self {
            symbols: SemanticScope::new(),
            current_return_ty: None,
            user_types: UserTypes::new(),
            current_struct: None,
            return_stmt_found: false,
            current_use_self: false,
        }
    }
    pub fn analyse_stmts(mut stmts: Vec<Stmt>) -> Option<UserTypes> {
        let mut analyser = Analyser::new();
        if let Err(err) = analyser.init_type_data(&mut stmts) {
            err.print();
            return None;
        }

        for stmt in &mut stmts {
            if let Err(err) = analyser.analyse_stmt(stmt) {
                err.print();
                return None;
            }
        }

        Some(analyser.user_types)
    }

    fn init_type_data(&mut self, stmts: &mut Vec<Stmt>) -> Result<(), SemErr> {
        for stmt in stmts {
            let line = stmt.get_line();
            if let StmtType::Enum { name, variants } = &stmt.stmt {
                if self
                    .user_types
                    .enums
                    .insert(name.to_string(), variants.clone())
                    .is_some()
                {
                    let err_ty = SemErrType::AlreadyDefinedEnum(name.to_string());
                    return Err(SemErr::new(line, err_ty));
                }
            } else if let StmtType::Func {
                name,
                parameters,
                body: _,
                return_ty,
                use_self,
            } = &mut stmt.stmt
            {
                let func_data = FuncData {
                    parameters: parameters.clone(),
                    body: vec![],
                    return_ty: return_ty.clone(),
                    line,
                    use_self: *use_self,
                };

                if self
                    .user_types
                    .funcs
                    .insert(name.to_string(), func_data)
                    .is_some()
                {
                    let err_ty = SemErrType::AlreadyDefinedFunc(name.to_string());
                    return Err(SemErr::new(line, err_ty));
                }
            } else if let StmtType::Struct {
                name,
                fields,
                methods,
            } = &mut stmt.stmt
            {
                if self.current_struct.is_some() {
                    // TODO: this doesn't work for some reason
                    let ty = SemErrType::StructDefInFunc(name.to_string());
                    return Err(SemErr::new(line, ty));
                }
                self.current_struct = Some(name.to_string());
                let struct_data = StructData::new(fields.clone());
                let mut method_data = vec![];

                if self
                    .user_types
                    .structs
                    .insert(name.to_string(), struct_data)
                    .is_some()
                {
                    let err_ty = SemErrType::AlreadyDefinedStruct(name.to_string());
                    return Err(SemErr::new(line, err_ty));
                }

                for method in methods.iter() {
                    if let StmtType::Func {
                        name,
                        parameters,
                        body: _,
                        return_ty,
                        use_self,
                    } = &method.stmt
                    {
                        let func_data = FuncData {
                            parameters: parameters.clone(),
                            body: vec![],
                            return_ty: return_ty.clone(),
                            line,
                            use_self: *use_self,
                        };
                        method_data.push((name.to_string(), func_data));
                    } else {
                        unreachable!()
                    }
                }

                self.user_types.structs.get_mut(name).unwrap().methods = method_data.clone();

                for (i, method) in methods.iter_mut().enumerate() {
                    self.analyse_stmt(method)?;

                    if let StmtType::Func { body, .. } = &method.stmt {
                        method_data[i].1.body = body.clone();
                    } else {
                        unreachable!()
                    }
                }

                self.user_types.structs.get_mut(name).unwrap().methods = method_data;
                self.current_struct = None;
                // self.symbols.declare(Symbol::new("Foo", ValueType::Struct(())), line)
            }
        }

        if !self.user_types.funcs.contains_key("main") {
            let err_ty = SemErrType::NoMainFunc;
            return Err(SemErr::new(0, err_ty));
        }
        Ok(())
    }
}
