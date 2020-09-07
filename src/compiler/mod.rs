use crate::bytecode::chunk::Chunk;
use crate::bytecode::module::Module;
use crate::bytecode::Instruction;
use crate::bytecode::Label;
use crate::bytecode::IR;
use crate::bytecode::OR;
use crate::compiler::flowgraph::cfg::CFG;
use crate::compiler::ir::Expr;
use crate::compiler::ir::Oper;
use crate::core::value::{Primitive, Value};
use crate::{
    compiler::ir::Stmt,
    core::{binop::BinOp, relop::RelOp},
};
use std::cell::RefCell;
use std::rc::Rc;

pub mod dominator;
pub mod flowgraph;
pub mod ir;
pub mod optimizer;
pub mod register_allocation;
pub mod transformer;

type Index = usize;

pub enum ContextType {
    Function,
    Method,
    Initializer,
    Script,
}

pub struct CompilerContext {
    context_type: ContextType,
    enclosing: Option<usize>,
    chunk_index: usize,
    stack_offset: usize,
}

impl CompilerContext {
    pub fn new() -> Self {
        CompilerContext {
            context_type: ContextType::Function,
            enclosing: None,
            chunk_index: 0,
            stack_offset: 0,
        }
    }
}

pub struct Compiler {
    module: Module,
    contexts: Vec<CompilerContext>,
}

// We can do a thing where we precomile the program to check for
// correctness first before doing optimizations and translating
// to machine code

impl Compiler {
    pub fn new() -> Self {
        let mut module = Module::new();

        let mut contexts = vec![];

        Compiler { module, contexts }
    }

    fn current_context(&self) -> &CompilerContext {
        self.contexts
            .last()
            .expect("expected a &CompilerContext to exist")
    }

    fn current_context_mut(&mut self) -> &mut CompilerContext {
        self.contexts
            .last_mut()
            .expect("expected a &CompilerContext to exist")
    }

    fn current_chunk(&self) -> &Chunk {
        self.module.get_chunk(self.current_context().chunk_index)
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        self.module
            .get_chunk_mut(self.current_context().chunk_index)
    }

    // fn add_constant(&mut self, constant: Constant) -> usize {
    //     self.module.add_constant(constant)
    // }

    fn add_instruction(&mut self, instruction: Instruction) {
        self.current_chunk_mut().add_instruction(instruction)
    }

    fn operand_to_or(&mut self, operand: &Oper) -> OR {
        match operand {
            Oper::Register(r) => OR::REG(*r),
            Oper::StackLocation(s) => {
                self.current_context_mut().stack_offset =
                    usize::max(self.current_context().stack_offset, *s);
                OR::STACK(*s)
            }
            _ => panic!(format!(
                "Expected a valid output register, got {:?} instead",
                operand
            )),
        }
    }

    fn operand_to_ir(&mut self, operand: &Oper) -> IR {
        match operand {
            Oper::Register(r) => IR::REG(*r),
            Oper::Value(c) => IR::VALUE(*c),
            Oper::StackLocation(s) => {
                self.current_context_mut().stack_offset =
                    usize::max(self.current_context().stack_offset, *s);
                IR::STACK(*s)
            }
            Oper::StackPop => IR::STACKPOP,
            Oper::ReturnValue => IR::RETVAL,
            _ => panic!(format!(
                "Expected machine instruction, got {:?} instead",
                operand
            )),
        }
    }

    pub fn compile_ir_to_bytecode(&mut self, cfgs: Vec<CFG>) -> Module {
        for cfg in cfgs {
            for stmt in cfg.statements() {
                self.compile_statement(stmt);
            }
        }
        self.module.clone()
    }

    // Statements
    fn compile_statement(&mut self, stmt: Rc<RefCell<Stmt>>) {
        match &*stmt.borrow() {
            Stmt::Tac(lval, rval) => self.compile_tac(lval, rval),
            Stmt::Label(label) => self.add_instruction(Instruction::LABEL(Label::Label(*label))),
            Stmt::NamedLabel(label) => {
                let chunk_index = self.module.add_chunk();
                self.contexts.push(CompilerContext {
                    context_type: ContextType::Function,
                    enclosing: None,
                    chunk_index: chunk_index,
                    stack_offset: 0,
                });
                self.module
                    .named_labels
                    .insert(*label, self.module.chunks.len() - 1);

                self.add_instruction(Instruction::LABEL(Label::Named(*label)))
            }
            Stmt::Jump(jumpto) => self.compile_jump(jumpto),
            Stmt::CJump(cond, jumpto) => self.compile_cjump(cond, jumpto),
            Stmt::Call(name, arity) => self.compile_call(name, arity),
            Stmt::Return(retval) => self.compile_return(retval),
            Stmt::StackPush(oper) => {
                let ir = self.operand_to_ir(oper);
                self.add_instruction(Instruction::PUSH(ir));
            }
            Stmt::StackPushAllReg => self.add_instruction(Instruction::PUSHA),
            Stmt::StackPopAllReg => self.add_instruction(Instruction::POPA),
            _ => unimplemented!("{:?}", stmt),
        }
    }

    fn compile_tac(&mut self, lval: &Oper, rval: &Expr) {
        let lval = self.operand_to_or(lval);
        self.compile_expression(lval, rval);
    }

    fn compile_jump(&mut self, jumpto: &usize) {
        self.add_instruction(Instruction::JUMP(Label::Label(*jumpto)));
    }

    fn compile_cjump(&mut self, cond: &Expr, jumpto: &usize) {
        let res = match cond {
            Expr::Oper(oper) => self.operand_to_ir(oper),
            _ => panic!("Expect cjump to contain operand"),
        };

        self.add_instruction(Instruction::BT(res, Label::Label(*jumpto)));
    }

    fn compile_call(&mut self, sym: &usize, arity: &usize) {
        self.add_instruction(Instruction::CALL(*sym, *arity));
        self.add_instruction(Instruction::NOP);
    }

    fn compile_return(&mut self, retval: &Option<Oper>) {
        if let Some(retval) = retval {
            let retval = self.operand_to_ir(retval);
            self.add_instruction(Instruction::RETURN(retval));
        } else {
            let retval = Value::from(Primitive::None);
            let retval = self.operand_to_ir(&Oper::Value(retval));
            self.add_instruction(Instruction::RETURN(retval));
        }
    }

    // Expressions
    fn compile_expression(&mut self, lval: OR, expr: &Expr) {
        match expr {
            Expr::Binary(left, op, right) => {
                let a = self.operand_to_ir(left);
                let b = self.operand_to_ir(right);

                match op {
                    BinOp::Plus => self.add_instruction(Instruction::ADD(lval, a, b)),
                    BinOp::Minus => self.add_instruction(Instruction::SUB(lval, a, b)),
                    BinOp::Star => self.add_instruction(Instruction::MUL(lval, a, b)),
                    BinOp::Slash => self.add_instruction(Instruction::DIV(lval, a, b)),
                    BinOp::Modulo => self.add_instruction(Instruction::MOD(lval, a, b)),
                    BinOp::Carat => self.add_instruction(Instruction::POW(lval, a, b)),
                    _ => unimplemented!(),
                }
            }
            Expr::Logical(left, op, right) => {
                let a = self.operand_to_ir(left);
                let b = self.operand_to_ir(right);

                match op {
                    RelOp::NotEqual => self.add_instruction(Instruction::NEQ(lval, a, b)),
                    RelOp::EqualEqual => self.add_instruction(Instruction::EQ(lval, a, b)),
                    RelOp::Less => self.add_instruction(Instruction::LT(lval, a, b)),
                    RelOp::LessEqual => self.add_instruction(Instruction::LTE(lval, a, b)),
                    RelOp::Greater => self.add_instruction(Instruction::GT(lval, a, b)),
                    RelOp::GreaterEqual => self.add_instruction(Instruction::GTE(lval, a, b)),
                    _ => unimplemented!(),
                }
            }
            Expr::Oper(oper) => {
                let rval = self.operand_to_ir(oper);
                self.add_instruction(Instruction::MOV(lval, rval));
            }
            _ => unimplemented!(),
        }
    }
}
