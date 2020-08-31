use crate::cli::parse_cli;

mod bytecode;
mod cli;
mod compiler;
mod parser;
mod util;
mod vm;

fn main() {
    match parse_cli() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
