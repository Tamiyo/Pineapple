#![allow(warnings)]

use crate::cli::parse_cli;
use crate::core::value::Type::{UInt16, UInt8};
use crate::core::value::*;

mod bytecode;
mod cli;
mod compiler;
mod core;
mod parser;
mod semantic_analysis;
mod util;
mod vm;

fn main() {
    match parse_cli() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
