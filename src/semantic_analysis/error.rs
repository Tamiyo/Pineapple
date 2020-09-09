use crate::parser::ast::{Expr, Stmt};
use crate::bytecode::string_intern::get_string;
use crate::core::value::Type;
use crate::core::value::Value;
use core::fmt;

type Sym = usize;

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
    InvalidValueType(Value, Type),
    InvalidVariableType(Sym, Type, Type),
    InvalidReturnType(Stmt, Type, Type),
    ExpectedNestedType(Expr),
    UndefinedVariable(Sym),
    UndefinedFunction(Sym),
    FunctionArityMismatch(Sym, usize, usize),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            TypeError::InvalidValueType(value, expected_type) => {
                write!(f, "Invalid Value Type for {:?}. Expected {:?} instead.", value, expected_type)
            }
            TypeError::InvalidVariableType(value, actual_type, expected_type) => {
                write!(f, "Invalid type for {:?}. Expected {:?} but got {:?}", get_string(*value), actual_type, expected_type)
            }
            TypeError::InvalidReturnType(expr, actual_type, expected_type) => {
                write!(f, "Invalid return type at {:?}. Expected {:?} but got {:?}", expr, actual_type, expected_type)
            }
            TypeError::ExpectedNestedType(expr) => {
                write!(f, "Expected {:?} to have a type. This is an internal error and you should flame the compiler engineer.", expr)
            }
            TypeError::UndefinedVariable(sym) => {
                write!(f, "Undefined Variable '{}'", get_string(*sym))
            }
            TypeError::UndefinedFunction(sym) => {
                write!(f, "Undefined Function '{}'", get_string(*sym))
            }
            TypeError::FunctionArityMismatch(sym, a, b) => {
                write!(f, "Function '{}' expected {} args, but got {}.", get_string(*sym), a, b)
            }
        }
    }
}