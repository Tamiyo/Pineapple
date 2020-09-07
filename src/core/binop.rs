use crate::parser::token::Symbol;
use crate::parser::token::Token;

use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BinOp {
    Plus,
    Minus,
    Slash,
    Modulo,
    Carat,
    Star,
    And,
    Or,
}

impl From<&Token> for BinOp {
    fn from(token: &Token) -> Self {
        match token.sym {
            Symbol::Plus => BinOp::Plus,
            Symbol::Minus => BinOp::Minus,
            Symbol::Slash => BinOp::Slash,
            Symbol::Modulo => BinOp::Modulo,
            Symbol::Carat => BinOp::Carat,
            Symbol::Star => BinOp::Star,
            Symbol::And => BinOp::And,
            Symbol::Or => BinOp::Or,
            _ => panic!("Invalid BinOp"),
        }
    }
}

impl fmt::Debug for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            BinOp::Plus => write!(f, "+"),
            BinOp::Minus => write!(f, "-"),
            BinOp::Slash => write!(f, "/"),
            BinOp::Modulo => write!(f, "%"),
            BinOp::Carat => write!(f, "^"),
            BinOp::Star => write!(f, "*"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
        }
    }
}
