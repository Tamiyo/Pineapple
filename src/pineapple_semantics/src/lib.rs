use pineapple_ast::ast::Stmt;
use pineapple_error::TypeError;

mod typecheck;

pub fn typecheck(ast: &mut Vec<Stmt>) -> Result<(), TypeError> {
    typecheck::typecheck(ast)
}
