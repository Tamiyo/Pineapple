use crate::vm::VM;
use pineapple_codegen_bytecode::module::Module;

mod callframe;
mod vm;

pub fn execute_vm(module: Module) {
    let mut vm = VM::new(module);
    vm.run_module();
}
