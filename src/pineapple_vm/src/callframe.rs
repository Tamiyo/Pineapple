use pineapple_ir::mir::Label;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RegVal {
    // Module Constants
    ValueLoc(usize),

    // Stack locations
    StackLoc(usize),

    // Memory Locations
    MemLoc(usize),

    RetAddr(usize),

    // "None"
    None,
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub ip: usize,
    pub base_sp: usize,
    pub chunk_index: usize,
    pub modified_registers: Vec<(usize, RegVal)>,
}

impl CallFrame {
    pub fn new(ip: usize, base_sp: usize, chunk_index: usize) -> Self {
        CallFrame {
            ip,
            base_sp,
            chunk_index,
            modified_registers: vec![],
        }
    }
}
