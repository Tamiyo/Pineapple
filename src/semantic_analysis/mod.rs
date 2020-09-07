use crate::parser::ast::Stmt;
use binding::Binder;
use error::{BindingError, TypeError};
use typecheck::type_check;

mod binding;
mod error;
mod local;
mod typecheck;

pub fn type_check_ast(ast: &Vec<Stmt>) -> Result<(), TypeError> {
    type_check(ast)
}

pub fn binding_check_ast(ast: &Vec<Stmt>) -> Result<(), BindingError> {
    let mut binder = Binder::new();
    binder.compile(ast)
}
