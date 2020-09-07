use crate::core::distance::Distance;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Symbol {
    // Single-character Symbols.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSquare,
    RightSquare,
    Comma,
    Colon,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Modulo,
    Carat,
    Star,
    // One or two character Symbols.
    Not,
    NotEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier(usize),
    StringLiteral(usize),
    IntegerLiteral(u128),
    FloatLiteral(f64),
    // Keywordss.
    Class,
    Fun,
    Var,
    And,
    Elif,
    Else,
    False,
    For,
    If,
    In,
    My,
    None,
    Or,
    Print,
    Return,
    Super,
    True,
    While,
    Eof,

    // Integer Primitives
    TypeInt8,
    TypeInt16,
    TypeInt32,
    TypeInt64,
    TypeInt128,
    TypeInt,

    TypeUInt8,
    TypeUInt16,
    TypeUInt32,
    TypeUInt64,
    TypeUInt128,
    TypeUInt,

    // Floating Point Primitives
    TypeFloat32,
    TypeFloat64,

    // Boolean Primitive
    TypeBool,

    // Character Primitive
    TypeChar,

    // Vector Complex Builtin
    TypeVector,

    // Tuple Complex Builtin
    TypeTuple,
}

#[derive(Copy, Clone)]
pub struct Token {
    pub sym: Symbol,

    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(sym: Symbol, line: usize, col: usize) -> Self {
        Token { sym, line, col }
    }
}

impl Deref for Token {
    type Target = Symbol;

    fn deref(&self) -> &Symbol {
        &self.sym
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.sym)
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.sym)
    }
}
