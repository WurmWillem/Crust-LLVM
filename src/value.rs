use std::fmt::{self};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ValueType {
    Null,
    // TODO: consider reworking/removing this
    Any, // useful as generic type for functions like println()
    Bool,
    F64,
    I64,
    U64,
    Str,
    Arr(Box<ValueType>),
    Struct(String),
    Enum(String),
    UnknownType(String),
}
impl ValueType {
    pub fn is_num(&self) -> bool {
        matches!(
            self,
            ValueType::F64 | ValueType::I64 | ValueType::U64 | ValueType::Enum(_)
        )
    }
}
impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // dbg!(self);
        match self {
            ValueType::Arr(ty) => write!(f, "[{ty}]"),
            ValueType::Any => write!(f, "Any"),
            ValueType::Null => write!(f, "Null"),
            ValueType::Bool => write!(f, "Bool"),
            ValueType::F64 => write!(f, "Double"),
            ValueType::I64 => write!(f, "Int"),
            ValueType::U64 => write!(f, "Uint"),
            ValueType::Str => write!(f, "String"),
            ValueType::Struct(s) => write!(f, "struct {s}"),
            ValueType::Enum(e) => write!(f, "enum {e}"),
            ValueType::UnknownType(t) => write!(f, "type {t}"),
        }
    }
}
