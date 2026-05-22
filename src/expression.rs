use crate::binary_op::BinaryOp;
use crate::{
    token::{Literal, TokenType},
    value::ValueType,
};

#[derive(Debug, Clone)]
pub struct Expr {
    pub expr: ExprType,
    line: u32,
    pub end_ty: ValueType,
}
impl Expr {
    pub fn new(expr: ExprType, line: u32) -> Expr {
        Expr {
            expr,
            line,
            end_ty: ValueType::Any,
        }
    }
    pub fn get_line(&self) -> u32 {
        self.line
    }
}

#[derive(Debug, Clone)]
pub enum ExprType {
    Lit(Literal),
    Array(Vec<Expr>),
    Identifier(String),
    FuncCall {
        name: String,
        args: Vec<Expr>,
        index: Option<usize>,
    },
    Cast {
        value: Box<Expr>,
        target_ty: ValueType,
    },
    MethodCall {
        inst: Box<Expr>,
        property: String,
        args: Vec<Expr>,
        is_static: bool,
    },
    MethodCallResolved {
        inst: Box<Expr>,
        index: u8,
        args: Vec<Expr>,
        use_self: bool,
    },
    Dot {
        inst: Box<Expr>,
        property: String,
    },
    Colon {
        inst: Box<Expr>,
        property: String,
    },
    DotResolved {
        inst_name: String,
        index: u8,
    },
    DotAssign {
        inst: Box<Expr>,
        property: String,
        new_value: Box<Expr>,
    },
    // TODO: this should not be an expression
    DotAssignResolved {
        inst_name: String,
        index: u8,
        new_value: Box<Expr>,
    },
    Index {
        arr: Box<Expr>,
        index: Box<Expr>,
    },
    AssignIndex {
        arr: Box<Expr>,
        index: Box<Expr>,
        new_value: Box<Expr>,
    },
    Assign {
        name: String,
        new_value: Box<Expr>,
    },
    Unary {
        prefix: TokenType,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    This,
}
