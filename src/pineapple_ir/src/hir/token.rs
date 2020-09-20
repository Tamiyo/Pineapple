use crate::value::Value;

type Identifier = usize;
type InternIndex = usize;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
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
    Ident(Identifier),
    StrLit(InternIndex),
    IntLit(Value),
    FloatLit(Value),

    // Keywords.
    Class,
    Fun,
    Var,
    And,
    As,
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

    // Types
    I8Ty,
    I16Ty,
    I32Ty,
    I64Ty,

    U8Ty,
    U16Ty,
    U32Ty,
    U64Ty,

    F32Ty,
    F64Ty,

    BoolTy,

    StringTy,
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,

    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, line, col }
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
