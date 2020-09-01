use crate::bytecode::string_intern::get_string;
use crate::compiler::CompilerContext;
use crate::{
    bytecode::{constant::Constant, OpCode, IR, OR},
    parser::{binop::BinOp, relop::RelOp},
};

pub const NUM_REGISTERS: usize = 16;

// I told someone I was making a VM and they wanted to try to run it in VirtualBox.
// I told them it wasn't that kind of VM but they gave me a funny look so I just went with it.
// Turns out you cant run this VM in VirtualBox (go figure).
// Now that person just thinks I'm a bad vm programmer. Thats probably not true. Probably.

// We're going to try to emulate a CISC approach, but this may change as time goes on.
#[derive(Debug)]
pub struct VM {
    /**
     *  [OpCode Pointer]
     *
     *  Points to the current instruction being executed.
     */
    ip: usize,

    /**
     *  [Frame Pointer]
     *
     *  Points to the current frame, at the top of the function call stack.
     */
    fp: usize,

    /**
     *  [Registers]
     *  
     *  p = parameters
     *  t = locals
     *
     *  R[0]               ->  return values
     *  R[1] ... R[p]      ->  parameter registers
     *  R[p+1] ... R[p+t]  ->  local registers
     */
    // TODO - Change constant to its own "VM" type (See Mango VM)
    r: [Constant; NUM_REGISTERS],

    // Return Address
    ra: usize,

    stack: Vec<Constant>,

    sp: usize,
}

impl VM {
    /**
     *  [Create a new VM]
     *
     *  Creates a new virtual machine with default parameters.
     */
    pub fn new() -> Self {
        VM {
            ip: 0,
            fp: 0,
            r: [Constant::None; NUM_REGISTERS],
            ra: 0,
            stack: vec![],
            sp: 0,
        }
    }

    fn store_ir(&mut self, or: &OR, ir: &IR) {
        let constant = self.from_input_register(ir);
        match or {
            OR::REG(or) => self.r[*or] = constant,
            OR::STACK(ptr) => self.stack[self.sp - *ptr] = constant,
            _ => unimplemented!(),
        }
    }

    fn store_constant(&mut self, or: &OR, constant: Constant) {
        match or {
            OR::REG(or) => self.r[*or] = constant,
            OR::STACK(ptr) => self.stack[self.sp - *ptr] = constant,
            _ => unimplemented!(),
        }
    }

    fn from_input_register(&self, ir: &IR) -> Constant {
        match ir {
            IR::REG(reg) => self.r[*reg],
            IR::CONST(c) => *c,
            IR::STACK(ptr) => self.stack[self.sp - *ptr],
        }
    }

    fn stack_push(&mut self, value: Constant) {
        self.stack.push(value);
        self.sp += 1;
    }

    /**
     *  Dispatches the current instruction, executing the specified
     *  virtual machine operations that align with each instruction.
     *
     *  Returns:
     *      Ok(()) : Execution had no problems.
     *      Err(e) : An error occured at some stage of execution.
     */
    pub fn dispatch(&mut self, compiler_context: &CompilerContext) -> Result<(), String> {

        // Reserve space on the stack for overflow'd variable allocations
        for _ in 0..(compiler_context.stack_offset + 1) {
            self.stack_push(Constant::None);
        }

        let instructions = &compiler_context.instructions;
        loop {
            let instruction = &instructions[self.ip];

            match instruction {
                OpCode::LABEL(_) => (),
                OpCode::MOV(or, ir) => {
                    self.store_ir(or, ir);
                }

                OpCode::ADD(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_binary(BinOp::Plus, b);
                    self.store_constant(or, res);
                }

                OpCode::NEQ(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_logical(RelOp::NotEqual, b);
                    self.store_constant(or, res);
                }

                OpCode::EQ(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_logical(RelOp::EqualEqual, b);
                    self.store_constant(or, res);
                }

                OpCode::LT(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_logical(RelOp::Less, b);
                    self.store_constant(or, res);
                }

                OpCode::LTE(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_logical(RelOp::LessEqual, b);
                    self.store_constant(or, res);
                }

                OpCode::GT(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_logical(RelOp::Greater, b);
                    self.store_constant(or, res);
                }

                OpCode::GTE(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_logical(RelOp::GreaterEqual, b);
                    self.store_constant(or, res);
                }

                OpCode::JUMP(label) => {
                    self.ip = *label;
                }

                OpCode::BT(ir, label) => {
                    let res = self.from_input_register(ir) == Constant::Boolean(true);

                    if res {
                        self.ip = *label;
                    }
                }

                OpCode::PUSH(ir) => {
                    let res = self.from_input_register(ir);
                    self.stack_push(res);
                }

                OpCode::CALL(intern, arity) => {
                    let name = get_string(*intern);
                    if name != "print" {
                        unimplemented!()
                    } else {
                        let mut values = vec![];
                        for _ in 0..*arity {
                            let value = self
                                .stack
                                .pop()
                                .expect("The stack should have a value here!");
                            values.push(value);
                        }

                        for value in values.iter().rev() {
                            print!("{} ", value);
                        }

                        print!("\n");
                    }
                }

                OpCode::HLT => {
                    break;
                }

                _ => unimplemented!(),
            }

            self.ip += 1;
        }

        Ok(())
    }
}
