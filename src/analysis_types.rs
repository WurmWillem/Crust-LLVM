use std::collections::HashMap;

use crate::{
    error::{SemErr, SemErrType},
    statement::Stmt,
    value::ValueType,
};

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    // binary
    Add,
    Sub,
    Mul,
    Div,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    And,
    Or,

    //unary
    Minus,
    Bang,
}
impl core::fmt::Display for Operator {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "=="),
            Operator::Less => write!(f, "<"),
            Operator::LessEqual => write!(f, "<="),
            Operator::Greater => write!(f, ">"),
            Operator::GreaterEqual => write!(f, ">="),
            Operator::And => write!(f, "&&"),
            Operator::Or => write!(f, "||"),
            Operator::Minus => write!(f, "-"),
            Operator::Bang => write!(f, "!"),
        }
    }
}

// TODO: rethink encapsulation
#[derive(Debug, Clone)]
pub struct FuncData {
    pub parameters: Vec<(ValueType, String)>,
    pub body: Vec<Stmt>,
    pub return_ty: ValueType,
    pub line: u32,
    pub use_self: bool,
}
#[derive(Debug)]
pub struct NatFuncData {
    pub parameters: Vec<ValueType>,
    pub return_ty: ValueType,
    pub use_self: bool,
}
#[derive(Debug)]
pub struct NatStructData {
    pub fields: Vec<(ValueType, String)>,
    pub methods: Vec<(String, NatFuncData)>,
}
impl NatStructData {
    pub fn get_method_data(
        &self,
        name: &str,
        property: &str,
        line: u32,
    ) -> Result<(u8, ValueType, bool, Vec<ValueType>), SemErr> {
        for (index, (method_name, data)) in self.methods.iter().enumerate() {
            if *method_name == property {
                let params = data.parameters.to_vec();
                return Ok((index as u8, data.return_ty.clone(), data.use_self, params));
            }
        }
        let ty = SemErrType::InvalidMethod(name.to_string(), property.to_string());
        Err(SemErr::new(line, ty))
    }
}
#[derive(Debug)]
pub struct StructData {
    pub fields: Vec<(ValueType, String)>,
    pub methods: Vec<(String, FuncData)>,
}
impl StructData {
    pub fn new(fields: Vec<(ValueType, String)>) -> Self {
        Self {
            fields,
            methods: vec![],
        }
    }

    pub fn get_method_data(
        &self,
        name: &str,
        property: &str,
        line: u32,
    ) -> Result<(u8, ValueType, bool, Vec<ValueType>), SemErr> {
        for (index, (method_name, data)) in self.methods.iter().enumerate() {
            if *method_name == property {
                let params = data.parameters.iter().map(|p| p.0.clone()).collect();
                return Ok((index as u8, data.return_ty.clone(), data.use_self, params));
            }
        }
        let ty = SemErrType::InvalidMethod(name.to_string(), property.to_string());
        Err(SemErr::new(line, ty))
    }

    pub fn get_field_index(&self, name: String, property: &str, line: u32) -> Result<u8, SemErr> {
        let index = match self
            .fields
            .iter()
            .position(|(_, field_name)| *field_name == property)
        {
            Some(index) => index as u8,
            None => {
                let ty = SemErrType::InvalidPubField(name, property.to_string());
                return Err(SemErr::new(line, ty));
            }
        };
        Ok(index)
    }
}

pub struct UserTypes {
    // TODO: maybe change to private
    pub funcs: HashMap<String, FuncData>,
    pub structs: HashMap<String, StructData>,
    pub enums: HashMap<String, Vec<String>>,
}
impl UserTypes {
    pub fn new() -> Self {
        Self {
            funcs: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub fn resolve_value_ty(&self, ty: &mut ValueType) {
        if let ValueType::UnknownType(name) = ty {
            if self.structs.contains_key(name as &str) {
                *ty = ValueType::Struct(name.clone())
            } else if self.enums.contains_key(name as &str) {
                *ty = ValueType::Enum(name.clone())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    name: String,
    pub ty: ValueType,
}
impl Symbol {
    pub fn new(name: String, ty: ValueType) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
pub struct SemanticScope {
    stack: Vec<HashMap<String, Symbol>>,
}
impl SemanticScope {
    pub fn new() -> Self {
        Self {
            stack: vec![HashMap::new()],
        }
    }

    pub fn begin_scope(&mut self) {
        self.stack.push(HashMap::new());
    }
    pub fn end_scope(&mut self) {
        self.stack.pop();
    }

    pub fn declare(&mut self, symbol: Symbol, line: u32) -> Result<(), SemErr> {
        let current = self.stack.last_mut().unwrap();
        if current.contains_key(&symbol.name) {
            return Err(SemErr::new(
                line,
                SemErrType::AlreadyDefinedVar(symbol.name.to_string()),
            ));
        }
        current.insert(symbol.name.clone(), symbol);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        for scope in self.stack.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym.clone());
            }
        }
        None
    }
}
