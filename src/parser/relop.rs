use crate::parser::tokens::Symbol;
use crate::parser::tokens::Token;

use std::fmt;

#[derive(Copy, Clone, PartialEq)]
pub enum RelOp {
    NotEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

impl RelOp {
    pub fn flip(&self) -> RelOp {
        match self {
            RelOp::NotEqual => RelOp::EqualEqual,
            RelOp::EqualEqual => RelOp::NotEqual,
            RelOp::Less => RelOp::GreaterEqual,
            RelOp::LessEqual => RelOp::Greater,
            RelOp::Greater => RelOp::LessEqual,
            RelOp::GreaterEqual => RelOp::Less,
        }
    }
}

impl From<&Token> for RelOp {
    fn from(token: &Token) -> Self {
        match token.sym {
            Symbol::NotEqual => RelOp::NotEqual,
            Symbol::EqualEqual => RelOp::EqualEqual,
            Symbol::Greater => RelOp::Greater,
            Symbol::GreaterEqual => RelOp::GreaterEqual,
            Symbol::Less => RelOp::Less,
            Symbol::LessEqual => RelOp::LessEqual,
            _ => panic!("Invalid RelOp"),
        }
    }
}

impl fmt::Debug for RelOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            RelOp::NotEqual => write!(f, "!="),
            RelOp::EqualEqual => write!(f, "=="),
            RelOp::Greater => write!(f, ">"),
            RelOp::GreaterEqual => write!(f, ">="),
            RelOp::Less => write!(f, "<"),
            RelOp::LessEqual => write!(f, "<="),
        }
    }
}
