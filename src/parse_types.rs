use crate::{analysis_types::Operator, token::TokenType};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Precedence {
    Primary,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Cast,       // as
    Unary,      // ! -
    Call,       // . ()
}
impl Precedence {
    pub fn to_next_precedency(self) -> Self {
        use Precedence::*;
        match self {
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Cast,
            Cast => Unary,
            Unary => Call,
            Call => Primary,
            Primary => Primary,
        }
    }
}
impl std::convert::From<u8> for Precedence {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Primary,
            1 => Self::Assignment,
            2 => Self::Or,
            3 => Self::And,
            4 => Self::Equality,
            5 => Self::Comparison,
            6 => Self::Term,
            7 => Self::Factor,
            8 => Self::Cast,
            9 => Self::Unary,
            10 => Self::Call,
            _ => panic!("Not a valid value for Precedence."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FnType {
    Empty,
    Grouping,
    Array,
    Unary,
    Binary,
    Cast,
    Number,
    String,
    Literal,
    Var,
    Call,
    Index,
    Dot,
    DoubleColon,
    This,
}

#[derive(Clone, Copy)]
pub struct ParseRule {
    pub prefix: FnType, // stores in what way can it be used as prefix (if used at all)
    pub infix: FnType,
    pub precedence: Precedence,
}
impl ParseRule {
    pub fn new(prefix: FnType, infix: FnType, precedence: Precedence) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
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
}
impl BinaryOp {
    pub fn get_precedency(self) -> Precedence {
        match self {
            BinaryOp::Add | BinaryOp::Sub => Precedence::Term,
            BinaryOp::Mul | BinaryOp::Div => Precedence::Factor,
            BinaryOp::Equal | BinaryOp::NotEqual => Precedence::Equality,
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                Precedence::Comparison
            }
            BinaryOp::And => Precedence::And,
            BinaryOp::Or => Precedence::Or,
        }
    }

    pub fn from_token_type(ty: TokenType) -> Self {
        match ty {
            TokenType::Plus => BinaryOp::Add,
            TokenType::Minus => BinaryOp::Sub,
            TokenType::Star => BinaryOp::Mul,
            TokenType::Slash => BinaryOp::Div,
            TokenType::EqualEqual => BinaryOp::Equal,
            TokenType::BangEqual => BinaryOp::NotEqual,
            TokenType::Less => BinaryOp::Less,
            TokenType::LessEqual => BinaryOp::LessEqual,
            TokenType::Greater => BinaryOp::Greater,
            TokenType::GreaterEqual => BinaryOp::GreaterEqual,
            TokenType::And => BinaryOp::And,
            TokenType::Or => BinaryOp::Or,
            _ => unreachable!(),
        }
    }

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
