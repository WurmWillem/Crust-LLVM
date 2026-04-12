use crate::{
    parse_types::BinaryOp,
    token::{Literal, TokenType},
    value::ValueType,
};

#[derive(Debug, Clone)]
pub struct Expr {
    pub expr: ExprType,
    line: u32,
}
impl Expr {
    pub fn new(expr: ExprType, line: u32) -> Expr {
        Expr { expr, line }
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
        target: ValueType,
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
        inst: Box<Expr>,
        index: u8,
    },
    DotAssign {
        inst: Box<Expr>,
        property: String,
        new_value: Box<Expr>,
    },
    DotAssignResolved {
        inst: Box<Expr>,
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
impl ExprType {
    pub fn to_string_debug(&self) -> String {
        match self {
            ExprType::Lit(literal) => literal.to_string(),
            // ExprType::Array(exprs) => todo!(),
            // ExprType::Identifier(s) => s.to_string(),
            // ExprType::FuncCall { name, args, index } => todo!(),
            // ExprType::Cast { value, target } => todo!(),
            // ExprType::MethodCall {
            //     inst,
            //     property,
            //     args,
            //     is_static,
            // } => todo!(),
            // ExprType::MethodCallResolved {
            //     inst,
            //     index,
            //     args,
            //     use_self,
            // } => todo!(),
            // ExprType::Dot { inst, property } => todo!(),
            // ExprType::Colon { inst, property } => todo!(),
            // ExprType::DotResolved { inst, index } => todo!(),
            // ExprType::DotAssign {
            //     inst,
            //     property,
            //     new_value,
            // } => todo!(),
            // ExprType::DotAssignResolved {
            //     inst,
            //     index,
            //     new_value,
            // } => todo!(),
            // ExprType::Index { arr, index } => todo!(),
            // ExprType::AssignIndex {
            //     arr,
            //     index,
            //     new_value,
            // } => todo!(),
            // ExprType::Assign { name, new_value } => todo!(),
            // ExprType::Unary { prefix, value } => todo!(),
            ExprType::Binary { left, op, right } => format!(
                "{} {} {}",
                left.expr.to_string_debug(),
                op.to_operator().to_string(),
                right.expr.to_string_debug()
            ),
            _ =>  todo!()

            // ExprType::This => todo!(),
        }
    }
}
