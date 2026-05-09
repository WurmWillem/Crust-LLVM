use std::collections::HashMap;

use colored::Colorize;

use crate::{binary_op::BinaryOp, error::print_error, statement::Stmt, value::ValueType};

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
impl BinaryOp {
    pub fn to_operator(self) -> Operator {
        match self {
            BinaryOp::Add => Operator::Add,
            BinaryOp::Sub => Operator::Sub,
            BinaryOp::Mul => Operator::Mul,
            BinaryOp::Div => Operator::Div,
            BinaryOp::Equal => Operator::Equal,
            BinaryOp::NotEqual => Operator::NotEqual,
            BinaryOp::Less => Operator::Less,
            BinaryOp::LessEqual => Operator::LessEqual,
            BinaryOp::Greater => Operator::Greater,
            BinaryOp::GreaterEqual => Operator::GreaterEqual,
            BinaryOp::And => Operator::And,
            BinaryOp::Or => Operator::Or,
        }
    }
}

#[derive(Debug)]
pub struct SemErr {
    ty: SemErrType,
    line: u32,
}
impl SemErr {
    pub fn new(line: u32, ty: SemErrType) -> Self {
        Self { ty, line }
    }
}
#[derive(Debug)]
pub enum SemErrType {
    NoMainFunc,
    InvalidInfixOp(ValueType, Operator),
    InvalidPrefixOp,
    SelfOutsideStruct,
    SelfAsStaticStruct,
    InvalidStaticAccess,
    SelfInMethodWithoutSelfParam,
    UndefinedVar(String),
    FuncDefInFunc(String),
    UndefinedFunc(String),
    IndexNonArr(ValueType),
    StructDefInFunc(String),
    UndefinedType(String),
    AlreadyDefinedVar(String),
    AlreadyDefinedFunc(String),
    AlreadyDefinedEnum(String),
    AlreadyDefinedStruct(String),
    NatParamTypeMismatch(String),
    StaticMethodOnInstance(String),
    SelfOnStaticMethod,
    NoSelfOnMethod,
    InvalidIfCondition(ValueType),
    InvalidWhileCondition(ValueType),
    InvalidTypeFieldAccess(ValueType),
    InvalidTypeMethodAccess(ValueType),
    NoReturnTy(String, ValueType),
    InvalidMethod(String, String),
    InvalidPubField(String, String),
    InvalidCast(ValueType, ValueType),
    InvalidVariant(String, String),
    IncorrectReturnTy(ValueType, ValueType),
    FieldTypeMismatch(ValueType, ValueType),
    ArrElTypeMismatch(ValueType, ValueType),
    VarDeclTypeMismatch(ValueType, ValueType),
    AssignArrTypeMismatch(ValueType, ValueType),
    IncorrectArity(String, u8, u8),
    OpTypeMismatch(ValueType, Operator, ValueType),
    ParamTypeMismatch(String, ValueType, ValueType),
}
impl SemErr {
    pub fn print(&self) {
        //dbg!(&self.ty);
        let msg = match &self.ty {
            SemErrType::InvalidPrefixOp => "invalid prefix.".to_string(),
            SemErrType::InvalidInfixOp(ty, op) => format!("Cannot apply operator '{}' to types {}.", op.to_string(), ty.to_string()),
            SemErrType::InvalidStaticAccess => "You can only use the '::' syntax for static methods.".to_string(),
            SemErrType::FuncDefInFunc(name) => format!("You attempted to define the function '{}' inside another function, which is illegal.", name.green()),
            SemErrType::StructDefInFunc(name) => format!("You attempted to define the struct '{}' inside a function, which is illegal.", name.green()),
            SemErrType::InvalidCast(expected, found) => format!("You can't cast an expression of type '{found}' to type '{expected}'."),
            SemErrType::InvalidIfCondition(found) => format!("If statement only accepts condition of type 'bool', found '{found}'."),
            SemErrType::InvalidWhileCondition(found) => format!("While statement only accepts condition of type 'bool', found '{found}'."),
            SemErrType::NoMainFunc => {
                "You have to define a function with the name 'main' as entry point for the program."
                    .to_string()
            }
            SemErrType::SelfOutsideStruct => {
                "'self.property' can only be used inside methods of structs.".to_string()
            }
            SemErrType::SelfInMethodWithoutSelfParam => {
                "'self.property' can only be used inside methods with 'self' as parameter.".to_string()
            }
            SemErrType::SelfAsStaticStruct => {
                "'self::property' is invalid syntax as self is not static. Did you mean 'self.property'?".to_string()
            }
            SemErrType::StaticMethodOnInstance(inst_name) => format!("You cannot use a static method on an instance ({}).", inst_name.green()),
            SemErrType::SelfOnStaticMethod => "'struct::property' can only be used for static methods which don't have self as parameter.".to_string(),
            SemErrType::NoSelfOnMethod => "'instance.property' can only be used for non-static methods which have self as parameter.".to_string(),
            SemErrType::InvalidTypeMethodAccess(ty) => {
                format!(
                    "You can only access methods of instances, but you tried to access a method of type '{ty}'."
                )
            }
            SemErrType::InvalidTypeFieldAccess(ty) => {
                format!(
                    "You can only access fields of instances, but you tried to access a field of type '{ty}'."
                )
            }
            SemErrType::InvalidPubField(name, property) => {
                format!("Struct '{name}' has no field named '{property}'.")
            }
            SemErrType::InvalidMethod(name, property) => {
                format!("Struct '{name}' has no method named '{property}'.")
            }
            SemErrType::InvalidVariant(name, property) => {
                format!("Enum '{name}' has no variant named '{property}'.")
            }
            SemErrType::IndexNonArr(ty) => format!(
                "You can only index arrays, but you tried to index the type '{ty}'."
            ),

            SemErrType::AssignArrTypeMismatch(expected, found) => {
                format!(
                    "Array is of type '[{expected}]', but you tried to assign a value of type '{found}' to one of its elements."
                )
            }
            SemErrType::IncorrectReturnTy(expected, found) => {
                format!(
                    "Function expected return type '{expected}', but found type '{found}'."
                )
            }
            SemErrType::NoReturnTy(name, return_ty) => {
                format!(
                    "Function '{name}' has return type '{return_ty}', but no return statement was found."
                )
            }
            SemErrType::IncorrectArity(name, expected, found) => {
                format!(
                    "Function '{}' expected {} argument(s), but found {}.",
                    name.green(),
                    expected,
                    found
                )
            }

            SemErrType::UndefinedFunc(name) => {
                format!("Function '{}' has not been defined.", name.green())
            }
            SemErrType::UndefinedType(name) => {
                format!("Type '{}' has not been defined.", name.green())
            }
            SemErrType::UndefinedVar(name) => {
                format!(
                    "Variable '{}' has not been defined in this scope.",
                    name.green()
                )
            }
            SemErrType::AlreadyDefinedVar(name) => {
                format!(
                    "Variable with name '{}' has already been defined in this scope.",
                    name.green()
                )
            }
            SemErrType::AlreadyDefinedFunc(name) => {
                format!(
                    "Function with name '{}' has already been defined.",
                    name.green()
                )
            }
            SemErrType::AlreadyDefinedEnum(name) => {
                format!(
                    "Enum with name '{}' has already been defined.",
                    name.green()
                )
            }
            SemErrType::AlreadyDefinedStruct(name) => {
                format!(
                    "Struct with name '{}' has already been defined.",
                    name.green()
                )
            }

            SemErrType::OpTypeMismatch(expected, op, found) => {
                format!(
                    "Operator '{op}' Expects type '{expected}', but found type '{found}'."
                )
            }
            SemErrType::VarDeclTypeMismatch(expected, found) => {
                format!(
                    "Variable was given type '{expected}', but found type '{found}'."
                )
            }
            SemErrType::ParamTypeMismatch(name, expected, found) => {
                format!(
                    "Parameter of function '{name}' has type '{expected}', but found type '{found}'."
                )
            }
            SemErrType::NatParamTypeMismatch(name) => {
                format!(
                    "The types of the parameters of function '{}' and the types of the given arguments don't match.",
                    name.green()
                )
            }
            SemErrType::FieldTypeMismatch(expected, found) => {
                format!(
                    "Field was given type '{expected}', but found type '{found}'."
                )
            }
            SemErrType::ArrElTypeMismatch(expected, found) => {
                format!(
                    "Not all elements in the array are of the same type. Array expected type '{expected}', but found type '{found}'."
                )
            }
        };
        print_error(self.line, &msg);
    }
}
