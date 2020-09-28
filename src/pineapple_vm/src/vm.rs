use crate::callframe::CallFrame;
use crate::callframe::RegVal;
use pineapple_codegen_bytecode::bytecode::Instruction;
use pineapple_codegen_bytecode::bytecode::{IR, OR};
use pineapple_codegen_bytecode::module::Module;
use pineapple_ir::mir::Label;
use pineapple_ir::Value;
use pineapple_ir::{value::ValueContainer, ValueWrapper};

const NUM_REGISTERS: usize = 16;

pub struct VM {
    module: Module,

    register: [RegVal; NUM_REGISTERS],

    frames: Vec<CallFrame>,

    ret: RegVal,

    stack: Vec<RegVal>,

    memory: Vec<pineapple_ir::Value>,

    sp: usize,
}

impl VM {
    pub fn new(module: Module) -> Self {
        VM {
            module,
            register: [RegVal::None; NUM_REGISTERS],
            frames: vec![],
            ret: RegVal::None,
            stack: vec![],
            memory: vec![],
            sp: 0,
        }
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("Expect &Callframe to exist")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames
            .last_mut()
            .expect("Expect &mut Callframe to exist")
    }

    fn pop_frame(&mut self) {
        let frame = self.frames.pop().unwrap();
        for (x, y) in frame.modified_registers {
            self.register[x] = y;
        }
    }

    fn store_ir(&mut self, or: &OR, ir: &IR) {
        let value = self.load_ir(ir);
        match or {
            OR::REG(or) => {
                let old = self.register[*or];
                self.current_frame_mut().modified_registers.push((*or, old));
                self.register[*or] = value;
            }
            OR::STACK(ptr) => self.stack[self.sp - *ptr] = value,
            _ => unimplemented!(),
        }
    }

    fn store_reg(&mut self, or: &OR, reg: RegVal) {
        match or {
            OR::REG(or) => self.register[*or] = reg,
            OR::STACK(ptr) => self.stack[self.sp - *ptr] = reg,
            _ => unimplemented!(),
        }
    }

    fn load_ir(&mut self, ir: &IR) -> RegVal {
        match ir {
            IR::REG(reg) => self.register[*reg],
            IR::VALUE(ptr) => RegVal::ValueLoc(*ptr),
            IR::STACK(ptr) => RegVal::StackLoc(self.sp - *ptr),
            IR::MEMLOC(ptr) => RegVal::MemLoc(*ptr),
            IR::STACKPOP => self.stack_pop(),
            IR::RETVAL => self.ret,
        }
    }

    fn load_reg(&self, reg: RegVal) -> &Value {
        match reg {
            RegVal::ValueLoc(ptr) => &self.module.values[ptr],
            RegVal::StackLoc(ptr) => self.load_reg(self.stack[ptr]),
            RegVal::MemLoc(ptr) => &self.memory[ptr],
            _ => panic!(""),
        }
    }

    fn stack_push(&mut self, value: RegVal) {
        self.stack.push(value);
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> RegVal {
        let value = self.stack.pop().unwrap();
        self.sp -= 1;
        value
    }

    pub fn run_module(&mut self) {
        let main_chunk_index = self
            .module
            .chunks
            .iter()
            .position(|chunk| match chunk.label {
                Label::Marker(_) => false,
                Label::Named(name) => {
                    if name == pineapple_session::intern_string("main".to_string()) {
                        true
                    } else {
                        false
                    }
                }
            })
            .unwrap();

        let frame = CallFrame::new(0, self.sp, main_chunk_index);
        self.frames.push(frame);

        self.dispatch();
    }

    fn dispatch(&mut self) {
        loop {
            let instruction: Instruction = {
                let frame = self.current_frame();
                let chunk_index = frame.chunk_index;
                self.module.chunks[chunk_index].instructions[frame.ip]
            };

            match &instruction {
                Instruction::LABEL(_) => (),
                Instruction::MOV(or, ir) => {
                    self.store_ir(or, ir);
                }

                Instruction::ADD(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = *a + *b;

                    // Need to implement GC (or not? :thinking:)
                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::SUB(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = *a - *b;

                    // Need to implement GC (or not? :thinking:)
                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::MUL(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = *a * *b;

                    // Need to implement GC (or not? :thinking:)
                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::DIV(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = *a / *b;

                    // Need to implement GC (or not? :thinking:)
                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::MOD(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = *a % *b;

                    // Need to implement GC (or not? :thinking:)
                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::LT(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = Value::from(*a < *b);

                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::LTE(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = Value::from(*a <= *b);

                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::GT(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = Value::from(*a > *b);

                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::GTE(or, ir1, ir2) => {
                    let ir1 = self.load_ir(ir1);
                    let ir2 = self.load_ir(ir2);
                    let a = self.load_reg(ir1);
                    let b = self.load_reg(ir2);

                    let res: Value = Value::from(*a >= *b);

                    let vmreg = RegVal::MemLoc(self.memory.len());
                    self.memory.push(res);

                    self.store_reg(or, vmreg);
                }

                Instruction::JUMP(label) => match label {
                    Label::Marker(_) => {
                        self.current_frame_mut().ip =
                            self.module.labels.get(label).unwrap().instruction_index;
                    }
                    _ => (),
                },

                Instruction::BT(ir, label) => {
                    // We'll prob need to do "local" chunk labels and "global" named labels in Module
                    let vmreg = self.load_ir(ir);
                    let val = self.load_reg(vmreg);
                    let res = val.into_inner() == ValueWrapper::BOOL(true);

                    if res {
                        match label {
                            Label::Marker(_) => {
                                self.current_frame_mut().ip =
                                    self.module.labels.get(label).unwrap().instruction_index;
                            }
                            _ => panic!(""),
                        }
                    }
                }

                Instruction::PUSH(ir) => {
                    let res = self.load_ir(ir);
                    self.stack_push(res);
                }

                Instruction::RETURN(ir) => {
                    let return_value = self.load_ir(ir);
                    self.pop_frame();

                    if self.frames.is_empty() {
                        return;
                    }

                    self.ret = return_value;
                }

                Instruction::CALL(intern, arity) => {
                    if pineapple_session::get_string(*intern) == "print" {
                        let mut values = vec![RegVal::None; *arity];
                        for i in 0..*arity {
                            let value = self
                                .stack
                                .pop()
                                .expect("The stack should have a value here!");
                            values[i] = value;
                        }

                        for vmreg in values.iter().rev() {
                            print!("{:?} ", self.load_reg(*vmreg));
                        }

                        println!("");
                    } else if self.module.labels.contains_key(&Label::Named(*intern)) {
                        // We can do tail recursion optimization here

                        let chunk_index = self
                            .module
                            .labels
                            .get(&Label::Named(*intern))
                            .unwrap()
                            .chunk_index;

                        let callframe = CallFrame::new(0, self.sp, chunk_index);
                        self.frames.push(callframe);
                    } else {
                        // This "should" get found during static analysis
                        panic!();
                    }
                }
                Instruction::NOP => (),
                Instruction::HLT => {
                    break;
                }
                _ => unimplemented!("{:?} not implemented", instruction),
            }

            self.current_frame_mut().ip += 1;
        }
    }
}
