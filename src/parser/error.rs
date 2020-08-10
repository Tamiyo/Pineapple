use crate::parser::ast::Expr;
use crate::parser::tokens::{Symbol, Token};

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    TokenStreamEmpty,
    UnexpectedToken(Token, Symbol),
    UnexpectedInfixOperator(Token),
    UnexpectedPrefixOperator(Token),
    ExpectedIdentifier(Token),
    ExpectedBinaryOperator(Token),
    ExpectedLiteral(Token),
    ExpectedLValue(Expr),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::TokenStreamEmpty => {
                write!(f, "Attempted to perform a seek() operation on the token stream, but the stream is empty.")
            }
            ParseError::UnexpectedToken(found, expected) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Encountered an unexpected token while parsing. Expected: `{:?}` but instead found: `{:?}`.", line, col, expected, &found)
            }
            ParseError::UnexpectedInfixOperator(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Expected an infix operator while parsing but instead found: `{:?}`.", line, col, found)
            }
            ParseError::UnexpectedPrefixOperator(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Expected an prefix operator while parsing but instead found: `{:?}`.", line, col, found)
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
