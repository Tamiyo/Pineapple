use crate::bytecode::distance::Distance;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;

/**
 *  [Symbol]
 *
 *  Symbols are the possible characters that can appear in a user's
 *  source code. They build the foundation for what the parser can
 *  determine about a user's input.
 */
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
    Identifier(String),
    String(String),
    Number(Distance),
    // Keywords.
    And,
    Class,
    Elif,
    Else,
    False,
    Fun,
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
    Var,
    Eof,
}

/**
 *  [Token]
 *  
 *  Token is a wrapper around the Symbol enum that also contains metadata
 *  about the sym, like its location in the user program.
 */
#[derive(Clone)]
pub struct Token {
    /**
     *  [Symbol]
     *
     *  The sym that this Token is associated with.
     */
    pub sym: Symbol,

    /**
     *  [Line]
     *
     *  The line number that this Token is associated with.
     */
    pub line: usize,

    /**
     *  [Column]
     *
     *  The column number that this Token is associated with.
     */
    pub col: usize,
}

impl Token {
    /**
     *  [Create a new Token]
     *
     *  Creates a new token the parameters given.
     */
    pub fn new(sym: Symbol, line: usize, col: usize) -> Self {
        Token { sym, line, col }
    }
}

/**
 *  Dereferences the Token, giving us the inner Symbol.
 *
 *  This is important because we do a lot of moves / copying during
 *  parsing and compilation. Having a convenient dereference makes
 *  working with Tokens easier in this regard.
 */
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
