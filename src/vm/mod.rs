use crate::bytecode::module::Module;
use crate::bytecode::string_intern::get_string;
use crate::bytecode::string_intern::intern_string;
use crate::core::value::compute_binary;
use crate::core::{
    binop::BinOp,
    relop::RelOp,
    value::{compute_logical, Primitive, Value, ValueType},
};
use crate::{
    bytecode::{Instruction, Label, IR, OR},
    vm::memory::function::Function,
};

mod memory;

pub const NUM_REGISTERS: usize = 16;

// I told someone I was making a VM and they wanted to try to run it in VirtualBox.
// I told them it wasn't that kind of VM but they gave me a funny look so I just went with it.
// Turns out you cant run this VM in VirtualBox (go figure).
// Now that person just thinks I'm a bad vm programmer. Thats probably not true. Probably.

#[derive(Debug)]
pub struct CallFrame {
    function: Function,
    ip: usize,
    base_counter: usize,
    chunk_index: usize,
}

// We're going to try to emulate a CISC approach, but this may change as time goes on.
#[derive(Debug)]
pub struct VM {
    frames: Vec<CallFrame>,

    /**
     *  [Registers]
     */
    // TODO - Change value to its own "VM" type (See Mango VM)
    register: [Value; NUM_REGISTERS],

    // return value
    return_value: Value,

    // Return Address
    return_addr: usize,

    stack: Vec<Value>,

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
            frames: vec![],
            register: [Value::from(Primitive::None); NUM_REGISTERS],
            return_value: Value::from(Primitive::None),
            return_addr: 0,
            stack: vec![],
            stack_ptr: 0,
        }
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("Expect &mut Callframe to exist")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames
            .last_mut()
            .expect("Expect &mut Callframe to exist")
    }

    fn store_ir(&mut self, or: &OR, ir: &IR) {
        let value = self.load_input_register(ir);
        match or {
            OR::REG(or) => self.register[*or] = value,
            OR::STACK(ptr) => self.stack[self.stack_ptr - *ptr] = value,
            _ => unimplemented!(),
        }
    }

    fn store_value(&mut self, or: &OR, value: Value) {
        match or {
            OR::REG(or) => self.register[*or] = value,
            OR::STACK(ptr) => self.stack[self.stack_ptr - *ptr] = value,
            _ => unimplemented!(),
        }
    }

    fn load_input_register(&mut self, ir: &IR) -> Value {
        match ir {
            IR::REG(reg) => self.register[*reg],
            IR::VALUE(c) => *c,
            IR::STACK(ptr) => self.stack[self.stack_ptr - *ptr],
            IR::STACKPOP => self.stack_pop(),
            IR::RETVAL => self.return_value,
        }
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
        self.stack_ptr += 1;
    }

    fn stack_pop(&mut self) -> Value {
        let value = self.stack.pop().unwrap();
        self.stack_ptr -= 1;
        value
    }

    fn begin_frame(&mut self, function: Function) {
        let arity = function.arity;
        let index = function.chunk_index;
        self.frames.push(CallFrame {
            function,
            ip: 0,
            base_counter: self.stack.len() - arity - 1,
            chunk_index: index,
        });
    }

    /*
     *  Dispatches the current instruction, executing the specified
     *  virtual machine operations that align with each instruction.
     *
     *  Returns:
     *      Ok(()) : Execution had no problems.
     *      Err(e) : An error occured at some stage of execution.
     */
    pub fn dispatch(&mut self, module: &Module) -> Result<(), String> {
        // TODO :- Investigate callframe base_counter

        let main_index = module
            .chunks
            .iter()
            .position(|chunk| {
                chunk.instructions[0]
                    == Instruction::LABEL(Label::Named(intern_string("main".to_string())))
            })
            .unwrap();

        let function = Function {
            arity: 0,
            chunk_index: main_index,
            name: intern_string("main".to_string()),
        };

        self.frames.push(CallFrame {
            function,
            ip: 0,
            base_counter: 0,
            chunk_index: main_index,
        });

        loop {
            let instruction: &Instruction = {
                let frame = self.current_frame();
                &module.get_chunk(frame.chunk_index).instructions[frame.ip]
            };

            match instruction {
                Instruction::LABEL(_) => (),
                Instruction::MOV(or, ir) => {
                    self.store_ir(or, ir);
                }

                Instruction::ADD(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_binary(a, BinOp::Plus, b);
                    self.store_value(or, res);
                }

                Instruction::SUB(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_binary(a, BinOp::Minus, b);
                    self.store_value(or, res);
                }

                Instruction::MUL(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_binary(a, BinOp::Star, b);
                    self.store_value(or, res);
                }

                Instruction::DIV(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_binary(a, BinOp::Slash, b);
                    self.store_value(or, res);
                }

                Instruction::MOD(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_binary(a, BinOp::Modulo, b);
                    self.store_value(or, res);
                }

                Instruction::POW(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_binary(a, BinOp::Carat, b);
                    self.store_value(or, res);
                }

                Instruction::NEQ(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_logical(a, RelOp::NotEqual, b);
                    self.store_value(or, res);
                }

                Instruction::EQ(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_logical(a, RelOp::EqualEqual, b);
                    self.store_value(or, res);
                }

                Instruction::LT(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_logical(a, RelOp::Less, b);
                    self.store_value(or, res);
                }

                Instruction::LTE(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_logical(a, RelOp::LessEqual, b);
                    self.store_value(or, res);
                }

                Instruction::GT(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_logical(a, RelOp::Greater, b);
                    self.store_value(or, res);
                }

                Instruction::GTE(or, ir1, ir2) => {
                    let a = self.load_input_register(ir1);
                    let b = self.load_input_register(ir2);

                    let res = compute_logical(a, RelOp::GreaterEqual, b);
                    self.store_value(or, res);
                }

                Instruction::JUMP(label) => match label {
                    Label::Label(l) => {
                        let chunk_index = self.current_frame().chunk_index;
                        self.current_frame_mut().ip =
                            *module.get_chunk(chunk_index).labels.get(label).unwrap();
                    }
                    Label::Named(l) => {
                        // self.instruction_ptr = compiler_context.named_labels[l];
                    }
                },

                Instruction::BT(ir, label) => {
                    let res = self.load_input_register(ir) == Value::from(Primitive::Bool(true));

                    if res {
                        match label {
                            Label::Label(l) => {
                                let chunk_index = self.current_frame().chunk_index;
                                self.current_frame_mut().ip =
                                    *module.get_chunk(chunk_index).labels.get(label).unwrap();
                            }
                            Label::Named(l) => {
                                // self.instruction_ptr = compiler_context.named_labels[l];
                            }
                        }
                    }
                }

                Instruction::PUSH(ir) => {
                    let res = self.load_input_register(ir);
                    self.stack_push(res);
                }

                // This is dirty for now, we can be smarter about how we save / restore registers in the future
                // See https://courses.cs.washington.edu/courses/cse378/09wi/lectures/lec05.pdf
                Instruction::PUSHA => {
                    for rindex in 0..self.register.len() {
                        self.stack_push(self.register[rindex]);
                    }
                    // self.stack_push(Value::Number(Distance::from(self.return_addr as f64)));
                    self.stack_push(Value::from(Primitive::UInt(self.return_addr)));
                }

                Instruction::POPA => {
                    let top = self.stack_pop();
                    if let ValueType::Primitive(Primitive::UInt(d)) = top.inner {
                        self.return_addr = d;
                    }

                    for rindex in (0..self.register.len()).rev() {
                        let popped = self.stack_pop();
                        self.register[rindex] = popped;
                    }
                }

                Instruction::CALL(intern, arity) => {
                    if get_string(*intern) == "print" {
                        let mut values = vec![Value::from(Primitive::None); *arity];
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
                    } else if module.named_labels.contains_key(intern) {
                        let function = Function {
                            arity: *arity,
                            chunk_index: *module.named_labels.get(intern).unwrap(),
                            name: intern_string("".to_string()),
                        };
                        self.begin_frame(function);
                    } else {
                        panic!(format!("function doesn't exist, {:?}", get_string(*intern)));
                    }
                }

                Instruction::RETURN(ir) => {
                    let return_value = self.load_input_register(ir);
                    self.frames.pop();

                    if self.frames.is_empty() {
                        return Ok(());
                    }

                    self.return_value = return_value;
                }
                Instruction::NOP => (),
                Instruction::HLT => {
                    break;
                }
                _ => unimplemented!("{:?} not implemented", instruction),
            }
            self.current_frame_mut().ip += 1;
        }

        Ok(())
    }
}
