use crate::bytecode::constant::Constant;
use crate::bytecode::string_intern::get_string;
use crate::bytecode::{Instruction, IR};
use crate::compiler::CompilerContext;

use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;

mod natives;

#[derive(Debug)]
pub struct VM {
    /**
     *  [Instruction Pointer]
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
    r: [Constant; 16],

    // Return Address
    ra: usize,

    stack: Vec<Constant>,
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
            r: [Constant::None; 16],
            ra: 0,
            stack: vec![],
        }
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
        let instructions = &compiler_context.instructions;
        loop {
            let instruction = &instructions[self.ip];

            match instruction {
                Instruction::LABEL(_) => (),
                Instruction::MOV(or, ir) => {
                    self.r[*or] = match ir {
                        IR::REG(reg) => self.r[*reg],
                        IR::CONST(c) => *c,
                    };
                }

                Instruction::ADD(or, ir1, ir2) => {
                    let a = match ir1 {
                        IR::REG(reg) => self.r[*reg],
                        IR::CONST(c) => *c,
                    };

                    let b = match ir2 {
                        IR::REG(reg) => self.r[*reg],
                        IR::CONST(c) => *c,
                    };

                    self.r[*or] = a.compute_binary(BinOp::Plus, b);
                }

                Instruction::NEQ(or, ir1, ir2) => {
                    let a = match ir1 {
                        IR::REG(reg) => self.r[*reg],
                        IR::CONST(c) => *c,
                    };

                    let b = match ir2 {
                        IR::REG(reg) => self.r[*reg],
                        IR::CONST(c) => *c,
                    };

                    self.r[*or] = a.compute_logical(RelOp::NotEqual, b);
                }

                Instruction::JUMP(label) => {
                    self.ip = *label;
                }

                Instruction::BT(ir, label) => {
                    let res = match ir {
                        IR::REG(reg) => self.r[*reg] == Constant::Boolean(true),
                        IR::CONST(c) => *c == Constant::Boolean(true),
                    };

                    if res {
                        self.ip = *label;
                    }
                }

                Instruction::PUSH(ir) => {
                    match ir {
                        IR::REG(reg) => {
                            self.stack.push(self.r[*reg]);
                        }
                        IR::CONST(c) => {
                            self.stack.push(*c);
                        }
                    };
                }

                Instruction::CALL(intern, arity) => {
                    let name = get_string(*intern);
                    if name != "print" {
                        unimplemented!()
                    } else {
                        for _ in 0..*arity {
                            let value = self
                                .stack
                                .pop()
                                .expect("The stack should have a value here!");
                            print!("{} ", value);
                        }
                        print!("\n");
                    }
                }

                Instruction::HLT => {
                    break;
                }

                _ => unimplemented!(),
            }

            self.ip += 1;
        }

        Ok(())
    }
}
