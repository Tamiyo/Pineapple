use std::fmt;

use crate::hir::token::{Token, TokenKind};

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
        match token.kind {
            TokenKind::Plus => BinOp::Plus,
            TokenKind::Minus => BinOp::Minus,
            TokenKind::Slash => BinOp::Slash,
            TokenKind::Modulo => BinOp::Modulo,
            TokenKind::Carat => BinOp::Carat,
            TokenKind::Star => BinOp::Star,
            TokenKind::And => BinOp::And,
            TokenKind::Or => BinOp::Or,
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

#[derive(Copy, Clone, PartialEq, Eq)]
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
        match token.kind {
            TokenKind::NotEqual => RelOp::NotEqual,
            TokenKind::EqualEqual => RelOp::EqualEqual,
            TokenKind::Greater => RelOp::Greater,
            TokenKind::GreaterEqual => RelOp::GreaterEqual,
            TokenKind::Less => RelOp::Less,
            TokenKind::LessEqual => RelOp::LessEqual,
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
