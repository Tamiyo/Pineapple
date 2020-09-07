// use crate::vm::memory::Upvalue;
use core::cell::RefCell;

#[derive(Debug)]
pub struct Function {
    pub name: usize,
    pub chunk_index: usize,
    pub arity: usize,
}

// pub struct Closure {
//     pub function: Function,
//     pub upvalues: Vec<RefCell<Upvalue>>,
// }