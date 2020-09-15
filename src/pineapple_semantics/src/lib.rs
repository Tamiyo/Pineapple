use pineapple_ast::ast::Stmt;
use pineapple_error::TypeError;

mod typecheck;

pub fn typecheck(ast: &Vec<Stmt>) -> Result<(), TypeError> {
    typecheck::typecheck(ast)
}
