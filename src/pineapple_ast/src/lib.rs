use crate::ast::Stmt;
use crate::parser::Parser;
use pineapple_error::ParseError;
use pineapple_ir::token::Token;

pub mod ast;
mod op;
mod parser;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
