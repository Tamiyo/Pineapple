use crate::bytecode::string_intern::get_string;
use crate::compiler::CompilerContext;
use crate::{
    bytecode::{constant::Constant, distance::Distance, Label, OpCode, IR, OR},
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
     *  Points to the current opcode being executed.
     */
    instruction_ptr: usize,

    /**
     *  [Frame Pointer]
     *
     *  Points to the current frame, at the top of the function call stack.
     */
    frame_ptr: usize,

    /**
     *  [Registers]
     *  
     *  p = parameters
     *  t = locals
     *
     *  register[0]               ->  return values
     *  register[1] ... register[p]      ->  parameter registers
     *  register[p+1] ... register[p+t]  ->  local registers
     */
    // TODO - Change constant to its own "VM" type (See Mango VM)
    register: [Constant; NUM_REGISTERS],

    // return value
    return_value: Constant,

    // Return Address
    return_addr: usize,

    stack: Vec<Constant>,

    stack_ptr: usize,
}

impl VM {
    /**
     *  [Create a new VM]
     *
     *  Creates a new virtual machine with default parameters.
     */
    pub fn new() -> Self {
        VM {
            instruction_ptr: 0,
            frame_ptr: 0,
            register: [Constant::None; NUM_REGISTERS],
            return_value: Constant::None,
            return_addr: 0,
            stack: vec![],
            stack_ptr: 0,
        }
    }

    fn store_ir(&mut self, or: &OR, ir: &IR) {
        let constant = self.from_input_register(ir);
        match or {
            OR::REG(or) => self.register[*or] = constant,
            OR::STACK(ptr) => self.stack[self.stack_ptr - *ptr] = constant,
            _ => unimplemented!(),
        }
    }

    fn store_constant(&mut self, or: &OR, constant: Constant) {
        match or {
            OR::REG(or) => self.register[*or] = constant,
            OR::STACK(ptr) => self.stack[self.stack_ptr - *ptr] = constant,
            _ => unimplemented!(),
        }
    }

    fn from_input_register(&mut self, ir: &IR) -> Constant {
        match ir {
            IR::REG(reg) => self.register[*reg],
            IR::CONST(c) => *c,
            IR::STACK(ptr) => self.stack[self.stack_ptr - *ptr],
            IR::STACKPOP => self.stack_pop(),
            IR::RETVAL => self.return_value,
        }
    }

    fn stack_push(&mut self, value: Constant) {
        self.stack.push(value);
        self.stack_ptr += 1;
    }

    fn stack_pop(&mut self) -> Constant {
        let constant = self.stack.pop().unwrap();
        self.stack_ptr -= 1;
        constant
    }

    /**
     *  Dispatches the current opcode, executing the specified
     *  virtual machine operations that align with each opcode.
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

        let opcodes = &compiler_context.opcodes;
        if opcodes.is_empty() {
            return Ok(());
        }

        loop {
            let opcode = &opcodes[self.instruction_ptr];

            match opcode {
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

                OpCode::SUB(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_binary(BinOp::Minus, b);
                    self.store_constant(or, res);
                }

                OpCode::MUL(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_binary(BinOp::Star, b);
                    self.store_constant(or, res);
                }

                OpCode::DIV(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_binary(BinOp::Slash, b);
                    self.store_constant(or, res);
                }

                OpCode::MOD(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_binary(BinOp::Modulo, b);
                    self.store_constant(or, res);
                }

                OpCode::POW(or, ir1, ir2) => {
                    let a = self.from_input_register(ir1);
                    let b = self.from_input_register(ir2);

                    let res = a.compute_binary(BinOp::Carat, b);
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

                OpCode::JUMP(label) => match label {
                    Label::Label(l) => {
                        self.instruction_ptr = compiler_context.labels[l];
                    }
                    Label::Named(l) => {
                        self.instruction_ptr = compiler_context.named_labels[l];
                    }
                },

                OpCode::JUMPR(label) => {
                    self.return_addr = self.instruction_ptr + 1;
                    match label {
                        Label::Label(l) => {
                            self.instruction_ptr = compiler_context.labels[l];
                        }
                        Label::Named(l) => {
                            self.instruction_ptr = compiler_context.named_labels[l];
                        }
                    }
                }

                OpCode::BT(ir, label) => {
                    let res = self.from_input_register(ir) == Constant::Boolean(true);

                    if res {
                        match label {
                            Label::Label(l) => {
                                self.instruction_ptr = compiler_context.labels[l];
                            }
                            Label::Named(l) => {
                                self.instruction_ptr = compiler_context.named_labels[l];
                            }
                        }
                    }
                }

                OpCode::PUSH(ir) => {
                    let res = self.from_input_register(ir);
                    self.stack_push(res);
                }
                
                // This is dirty for now, we can be smarter about how we save / restore registers in the future
                // See https://courses.cs.washington.edu/courses/cse378/09wi/lectures/lec05.pdf
                OpCode::PUSHA => {
                    for rindex in 0..self.register.len() {
                        self.stack_push(self.register[rindex]);
                    }
                    self.stack_push(Constant::Number(Distance::from(self.return_addr as f64)));
                }

                OpCode::POPA => {
                    if let Constant::Number(d) = self.stack_pop() {
                        self.return_addr = (Into::<f64>::into(d)) as usize;
                    }

                    for rindex in (0..self.register.len()).rev() {
                        let popped = self.stack_pop();
                        self.register[rindex] = popped;
                    }
                }

                // This is temporary, we'll either chose to use this or the JUMPR opcode
                // The print is defentially temporary, we'll change this when we add native function bindings
                OpCode::CALL(intern, arity) => {
                    // Sometimes we need this because of our janky call setup
                    // self.return_addr = self.instruction_ptr + 1;
                    let name = get_string(*intern);
                    if name == "print" {
                        let mut values = vec![Constant::None; *arity];
                        for i in 0..*arity {
                            let value = self
                                .stack
                                .pop()
                                .expect("The stack should have a value here!");
                            values[i] = value;
                        }

                        for value in values.iter().rev() {
                            print!("{} ", value);
                        }

                        println!("");
                    } else {
                        panic!("function doesn't exist")
                    }
                }

                OpCode::RETURN(ir) => {
                    let res = self.from_input_register(ir);
                    self.return_value = res;
                    self.instruction_ptr = self.return_addr;
                }
                OpCode::NOP => (),
                OpCode::HLT => {
                    break;
                }
                _ => unimplemented!("{:?} not implemented", opcode),
            }

            self.instruction_ptr += 1;
        }

        Ok(())
    }
}
