#![allow(warnings)]

use crate::cli::parse_cli;

mod bytecode;
mod cli;
mod core;
mod compiler;
mod parser;
mod util;
mod semantic_analysis;
mod vm;

fn main() {
    match parse_cli() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
