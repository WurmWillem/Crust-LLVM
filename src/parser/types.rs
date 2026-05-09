use crate::{binary_op::BinaryOp, token::TokenType};

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
}

impl TokenType {
    pub fn to_parse_rule(self) -> ParseRule {
        use FnType as F;
        use Precedence as P;
        use TokenType as TT;

        match self {
            TT::LeftParen => ParseRule::new(F::Grouping, F::Call, P::Call),
            TT::LeftBracket => ParseRule::new(F::Array, F::Index, P::Call),
            TT::Dot => ParseRule::new(F::Empty, F::Dot, P::Call),
            TT::DoubleColon => ParseRule::new(F::Empty, F::DoubleColon, P::Call),
            TT::Minus => ParseRule::new(F::Unary, F::Binary, P::Term),
            TT::Plus => ParseRule::new(F::Empty, F::Binary, P::Term),
            TT::Slash | TT::Star => ParseRule::new(F::Empty, F::Binary, P::Factor),
            TT::Bang => ParseRule::new(F::Unary, F::Empty, P::Factor),
            TT::EqualEqual
            | TT::BangEqual
            | TT::Greater
            | TT::GreaterEqual
            | TT::Less
            | TT::LessEqual => ParseRule::new(F::Empty, F::Binary, P::Comparison),
            TT::Identifier => ParseRule::new(F::Var, F::Empty, P::Primary),
            TT::StringLit => ParseRule::new(F::String, F::Empty, P::Primary),
            TT::Num => ParseRule::new(F::Number, F::Empty, P::Primary),
            TT::As => ParseRule::new(F::Number, F::Cast, P::Cast),
            TT::And => ParseRule::new(F::Empty, F::Binary, P::And),
            TT::Or => ParseRule::new(F::Empty, F::Binary, P::Or),
            TT::False | TT::True | TT::Null => ParseRule::new(F::Literal, F::Empty, P::Primary),
            TT::This => ParseRule::new(F::This, F::Empty, P::Primary),
            _ => ParseRule::new(F::Empty, F::Empty, P::Primary),
        }
    }
}
