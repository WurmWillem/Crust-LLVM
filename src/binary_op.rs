use crate::analysis_types::Operator;
use crate::token::TokenType;

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
