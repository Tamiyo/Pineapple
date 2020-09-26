use crate::module::Module;
use convert::Compiler;
use pineapple_codegen_ssa::analysis::cfg::CFG;

pub mod bytecode;
mod convert;
pub mod module;

pub fn compile_cfgs_to_bytecode(cfgs: Vec<CFG>) -> Module {
    let compiler = Compiler::default();
    compiler.compile_program(cfgs)
}
