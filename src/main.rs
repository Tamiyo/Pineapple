use crate::cli::parse_cli;

mod graph;
mod compiler;
mod parser;
mod bytecode;
mod cli;
mod vm;

fn main() {
    match parse_cli() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}