pub mod dominator;
pub mod flowgraph;
pub mod ir;
pub mod optimizer;
pub mod register_allocation;
pub mod transformer;

use crate::bytecode::{OpCode, IR, OR};
use crate::{
    compiler::flowgraph::cfg::CFG,
    parser::{binop::BinOp, relop::RelOp},
};
use ir::{Expr, Oper, Stmt};
use std::collections::HashMap;

pub struct CompilerContext {
    pub instructions: Vec<OpCode>,
    pub labels: HashMap<usize, usize>,
    pub stack_offset: usize,
}

impl CompilerContext {
    pub fn new() -> Self {
        CompilerContext {
            instructions: Vec::new(),
            labels: HashMap::new(),
            stack_offset: 0,
        }
    }
}

pub fn compile_ir(cfg: &CFG) -> CompilerContext {
    let mut compiler_context = CompilerContext::new();

    let statements = cfg.statements();
    let size = statements.len();
    for (i, statement) in statements.iter().rev().enumerate() {
        match &*statement.borrow() {
            Stmt::Label(label) | Stmt::NamedLabel(label) => {
                compiler_context.labels.insert(*label, size - i - 1);
            }
            _ => (),
        }
    }

    for (i, statement) in statements.iter().rev().enumerate() {
        match &*statement.borrow() {
            Stmt::Tac(lval, rval) => compile_tac(&mut compiler_context, lval, rval),
            Stmt::Label(_) | Stmt::NamedLabel(_) => {
                compiler_context
                    .instructions
                    .push(OpCode::LABEL(size - i - 1));
            }
            Stmt::Jump(label) => {
                let label = compiler_context.labels[label];
                compiler_context.instructions.push(OpCode::JUMP(label))
            }
            Stmt::CJump(cond, label) => compile_cjump(&mut compiler_context, cond, label),
            Stmt::StackPush(operand) => {
                let inreg = match operand {
                    Oper::Register(r) => IR::REG(*r),
                    Oper::Constant(c) => IR::CONST(*c),
                    Oper::StackLocation(s) => {
                        compiler_context.stack_offset = usize::max(compiler_context.stack_offset, *s);
                        IR::STACK(*s)
                    },
                    _ => panic!("Unexpected Register"),
                };

                compiler_context.instructions.push(OpCode::PUSH(inreg))
            }
            Stmt::Call(intern, arity) => compiler_context
                .instructions
                .push(OpCode::CALL(*intern, *arity)),
            _ => unimplemented!(),
        }
    }

    compiler_context.instructions.reverse();
    compiler_context.instructions.push(OpCode::HLT);
    compiler_context
}

fn compile_tac(compiler_context: &mut CompilerContext, lval: &Oper, rval: &Expr) {
    let outreg = match lval {
        Oper::Register(r) => OR::REG(*r),
        Oper::StackLocation(s) => {
            compiler_context.stack_offset = usize::max(compiler_context.stack_offset, *s);
            OR::STACK(*s)
        },
        _ => panic!("Expected a register as output!"),
    };

    match rval {
        Expr::Binary(a, op, b) => {
            let a = match a {
                Oper::Register(r) => IR::REG(*r),
                Oper::Constant(c) => IR::CONST(*c),
                Oper::StackLocation(s) => {
                    compiler_context.stack_offset = usize::max(compiler_context.stack_offset, *s);
                    IR::STACK(*s)
                },
                _ => panic!("Unexpected Register"),
            };

            let b = match b {
                Oper::Register(r) => IR::REG(*r),
                Oper::Constant(c) => IR::CONST(*c),
                Oper::StackLocation(s) => {
                    compiler_context.stack_offset = usize::max(compiler_context.stack_offset, *s);
                    IR::STACK(*s)
                },
                _ => panic!("Unexpected Register"),
            };

            match op {
                BinOp::Plus => compiler_context
                    .instructions
                    .push(OpCode::ADD(outreg, a, b)),
                _ => unimplemented!(),
            }
        }
        Expr::Logical(a, op, b) => {
            let a = match a {
                Oper::Register(r) => IR::REG(*r),
                Oper::Constant(c) => IR::CONST(*c),
                Oper::StackLocation(s) => {
                    compiler_context.stack_offset = usize::max(compiler_context.stack_offset, *s);
                    IR::STACK(*s)
                },
                _ => panic!("Unexpected Register"),
            };

            let b = match b {
                Oper::Register(r) => IR::REG(*r),
                Oper::Constant(c) => IR::CONST(*c),
                Oper::StackLocation(s) => {
                    compiler_context.stack_offset = usize::max(compiler_context.stack_offset, *s);
                    IR::STACK(*s)
                },
                _ => panic!("Unexpected Register"),
            };

            match op {
                RelOp::NotEqual => compiler_context
                    .instructions
                    .push(OpCode::NEQ(outreg, a, b)),
                RelOp::EqualEqual => compiler_context.instructions.push(OpCode::EQ(outreg, a, b)),
                RelOp::Less => compiler_context.instructions.push(OpCode::LT(outreg, a, b)),
                RelOp::LessEqual => compiler_context
                    .instructions
                    .push(OpCode::LTE(outreg, a, b)),
                RelOp::Greater => compiler_context.instructions.push(OpCode::GT(outreg, a, b)),
                RelOp::GreaterEqual => compiler_context
                    .instructions
                    .push(OpCode::GTE(outreg, a, b)),
                _ => unimplemented!(),
            }
        }
        Expr::Oper(operand) => {
            let inreg = match operand {
                Oper::Register(r) => IR::REG(*r),
                Oper::Constant(c) => IR::CONST(*c),
                Oper::StackLocation(s) => {
                    compiler_context.stack_offset = usize::max(compiler_context.stack_offset, *s);
                    IR::STACK(*s)
                },
                _ => panic!("Unexpected Register!"),
            };
            compiler_context
                .instructions
                .push(OpCode::MOV(outreg, inreg));
        }
        // Expr::StackPop => {}
        _ => unimplemented!(),
    };
}

fn compile_cjump(compiler_context: &mut CompilerContext, cond: &Expr, label: &usize) {
    let outreg = match cond {
        Expr::Oper(Oper::Register(r)) => IR::REG(*r),
        Expr::Oper(Oper::Constant(c)) => IR::CONST(*c),
        Expr::Oper(Oper::StackLocation(ptr)) => IR::STACK(*ptr),
        _ => panic!("expected register here!"),
    };

    let label = compiler_context.labels[label];
    compiler_context
        .instructions
        .push(OpCode::BT(outreg, label))
}
