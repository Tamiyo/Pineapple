use crate::bytecode::string_intern::get_string;
use crate::core::value::Type;
use crate::parser::ast::Expr;
use crate::parser::token::{Symbol, Token};

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    TokenStreamEmpty,
    CastError(Token, Type),
    UnexpectedToken(Token, Symbol),
    UnexpectedInfixOperator(Token),
    UnexpectedPrefixOperator(Token),
    ExpectedIdentifier(Token),
    ExpectedBinaryOperator(Token),
    ExpectedLiteral(Token),
    ExpectedLValue(Expr),
    ExpectedVariableType(Token),
    UndefinedVariable(usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::TokenStreamEmpty => {
                write!(f, "Attempted to perform a seek() operation on the token stream, but the stream is empty.")
            }
            ParseError::CastError(token, target_type) => {
                let line = token.line;
                let col = token.col;
                write!(f, "At {}:{}. Attempted to cast {:?} to {:?}, but cast failed", line, col, token, target_type)
            },
            ParseError::UnexpectedToken(found, expected) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Encountered an unexpected token while parsing. Expected: `{:?}` but instead found: `{:?}`.", line, col, expected, &found)
            }
            ParseError::UnexpectedInfixOperator(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Expected a infix operator while parsing but instead found: `{:?}`.", line, col, found)
            }
            ParseError::UnexpectedPrefixOperator(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Expected a prefix operator while parsing but instead found: `{:?}`.", line, col, found)
            }
            ParseError::ExpectedIdentifier(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Expected a identifier while parsing, but instead found: `{:?}`.", line, col, *found)
            }
            ParseError::ExpectedBinaryOperator(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Expected a binary operator while parsing, but instead found: `{:?}`.", line, col, *found)
            }
            ParseError::ExpectedLiteral(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Expected a literal while parsing, but instead found: `{:?}`.", line, col, *found)
            }
            ParseError::ExpectedLValue(expr) => {
                write!(f, "Expected a lvalue while parsing, but instead found: `{:?}`.", expr)
            }
            ParseError::ExpectedVariableType(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}, Expected a type, but instead found: `{:?}`.", line, col, *found)
            }
            ParseError::UndefinedVariable(sym) => {
                write!(f, "Undfined Variable {:?}", get_string(*sym))
            }
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub enum ScanError {
    InputStreamEmpty,
    UnterminatedString(usize, usize),
    InvalidNumeric(usize, usize),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScanError::InputStreamEmpty => {
                write!(f, "Attempted to perform a seek() operation on the input stream, but the stream is empty.")
            }
            ScanError::UnterminatedString(line, col) => {
                write!(f, "At {}:{}. Encountered a unterminated string while scanning.", line, col)
            }
            ScanError::InvalidNumeric(line, col) => {
                write!(f, "At {}:{}. Encountered a invalid numeric while scanning.", line, col)
            }
        }
    }
}

impl error::Error for ScanError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
