use pineapple_ast::ast;
use pineapple_ir::mir::Stmt;

use crate::convert::LinearCodeTranslator;

mod convert;

type Block = Vec<Stmt>;

pub fn convert_ast_to_linear_code(ast: Vec<ast::Stmt>) -> Vec<Block> {
    let mut translator = LinearCodeTranslator::new();
    translator.translate(ast)
}
