use crate::convert::LinearCodeTransformer;
use pineapple_ast::ast;
use pineapple_ir::mir::Stmt;

mod convert;

type Block = Vec<Stmt>;

pub fn convert_ast_to_linearcode(ast: Vec<ast::Stmt>) -> Vec<Block> {
    let mut transformer = LinearCodeTransformer::new();
    transformer.translate(ast)
}
