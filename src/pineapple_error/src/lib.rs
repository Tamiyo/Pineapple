use std::fmt;

use pineapple_ir::hir::token::{Token, TokenKind};
use pineapple_ir::value::{Value, ValueTy};

type Ident = usize;
type Type = ValueTy;

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

#[derive(Debug)]
pub enum ParseError {
    TokenStreamEmpty,
    CastError(Token, Type),
    UnexpectedToken(Token, TokenKind),
    UnexpectedInfixOperator(Token),
    UnexpectedPrefixOperator(Token),
    ExpectedIdentifier(Token),
    ExpectedBinaryOperator(Token),
    ExpectedLiteral(Token),
    ExpectedVariableTy(Token),
    ExpectedLValue,
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
            }
            ParseError::UnexpectedToken(found, expected) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}. Encountered an unexpected token while parsing. Expected: `{:?}` but instead found: `{:?}`.", line, col, found, expected)
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
            ParseError::ExpectedVariableTy(found) => {
                let line = found.line;
                let col = found.col;
                write!(f, "At {}:{}, Expected a type, but instead found: `{:?}`.", line, col, *found)
            }
            ParseError::ExpectedLValue => {
                write!(f, "Expected Lvalue in assign")
            }
            ParseError::UndefinedVariable(sym) => {
                write!(f, "Undefined Variable {:?}", pineapple_session::get_string(*sym))
            }
        }
    }
}

pub enum BindingError {
    PlaceHolder,
}

impl fmt::Display for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            BindingError::PlaceHolder => write!(f, "scoping rules were not sastified"),
        }
    }
}

pub enum TypeError {
    InvalidValueType(Value, Type, Type),
    InvalidVariableType(Ident, Type, Type),
    InvalidReturnType(Type, Type),
    InvalidExprType(Type, Type),
    ExpectedNestedType,
    UndefinedVariable(Ident),
    UndefinedFunction(Ident),
    FunctionArityMismatch(Ident, usize, usize),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            TypeError::InvalidValueType(value, expected_type, actual_type) => {
                write!(f, "Invalid Value Type for {:?}. Expected {:?} but got {:?} instead.", value, expected_type, actual_type)
            }
            TypeError::InvalidVariableType(value, actual_type, expected_type) => {
                write!(f, "Invalid type for {:?}. Expected {:?} but got {:?}", pineapple_session::get_string(*value), actual_type, expected_type)
            }
            TypeError::InvalidReturnType(actual_type, expected_type) => {
                write!(f, "Invalid return type. Expected {:?} but got {:?}", actual_type, expected_type)
            }
            TypeError::InvalidExprType(expected, actual) => {
                write!(f, "Invalid expr type. Expected {:?} but got {:?}", expected, actual)
            }
            TypeError::ExpectedNestedType => {
                write!(f, "Expected expr to have a type. This is an internal error and you should flame the compiler engineer.")
            }
            TypeError::UndefinedVariable(sym) => {
                write!(f, "Undefined Variable '{}'", pineapple_session::get_string(*sym))
            }
            TypeError::UndefinedFunction(sym) => {
                write!(f, "Undefined Function '{}'", pineapple_session::get_string(*sym))
            }
            TypeError::FunctionArityMismatch(sym, a, b) => {
                write!(f, "Function '{}' expected {} args, but got {}.", pineapple_session::get_string(*sym), a, b)
            }
        }
    }
}
