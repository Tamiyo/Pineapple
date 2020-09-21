use pineapple_error::ParseError;
use pineapple_ir::hir::token::Token;

use crate::ast::Stmt;
use crate::parser::Parser;

pub mod ast;
mod parser;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
