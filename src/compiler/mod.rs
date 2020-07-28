use std::collections::HashMap;

use crate::bytecode::{Instruction, IR, OR};
use crate::compiler::control_flow::ControlFlowContext;
use crate::compiler::three_address_code::{Expr, Operand, Stmt};
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;

pub mod control_flow;
pub mod dominator;
pub mod liveness_analysis;
pub mod optimization;
pub mod static_single_assignment;
pub mod three_address_code;

pub struct CompilerContext {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<usize, usize>,
}

impl CompilerContext {
    pub fn new() -> Self {
        CompilerContext {
            instructions: Vec::new(),
            labels: HashMap::new(),
        }
    }
}

pub fn compile_ir(context: &ControlFlowContext) -> CompilerContext {
    let mut compiler_context = CompilerContext::new();

    let statements = context.cfg.get_statements();
    let size = statements.len();
    for (i, statement) in statements.iter().rev().enumerate() {
        match statement {
            Stmt::Tac(lval, rval) => compile_tac(&mut compiler_context, lval, rval),
            Stmt::Label(label) | Stmt::NamedLabel(label) => {
                compiler_context.labels.insert(*label, size - i - 1);
                compiler_context
                    .instructions
                    .push(Instruction::LABEL(size - i - 1));
            }
            Stmt::Jump(label) => {
                let label = compiler_context.labels[label];
                compiler_context.instructions.push(Instruction::JUMP(label))
            }
            Stmt::CJump(cond, label) => compile_cjump(&mut compiler_context, cond, label),
            Stmt::StackPush(operand) => {
                let inreg = match operand {
                    Operand::Register(r) => IR::REG(*r),
                    Operand::Constant(c) => IR::CONST(*c),
                    _ => panic!("Unexpected Register"),
                };

                compiler_context.instructions.push(Instruction::PUSH(inreg))
            }
            _ => unimplemented!(),
        }
    }

    compiler_context.instructions.reverse();
    compiler_context.instructions.push(Instruction::HLT);
    compiler_context
}

fn compile_tac(compiler_context: &mut CompilerContext, lval: &Operand, rval: &Expr) {
    let outreg = match lval {
        Operand::Register(r) => *r,
        _ => panic!("Expected a register as output!"),
    };

    match rval {
        Expr::Binary(a, op, b) => {
            let a = match a {
                Operand::Register(r) => IR::REG(*r),
                Operand::Constant(c) => IR::CONST(*c),
                _ => panic!("Unexpected Register"),
            };

            let b = match b {
                Operand::Register(r) => IR::REG(*r),
                Operand::Constant(c) => IR::CONST(*c),
                _ => panic!("Unexpected Register"),
            };

            match op {
                BinOp::Plus => compiler_context
                    .instructions
                    .push(Instruction::ADD(outreg, a, b)),
                _ => unimplemented!(),
            }
        }
        Expr::Logical(a, op, b) => {
            let a = match a {
                Operand::Register(r) => IR::REG(*r),
                Operand::Constant(c) => IR::CONST(*c),
                _ => panic!("Unexpected Register"),
            };

            let b = match b {
                Operand::Register(r) => IR::REG(*r),
                Operand::Constant(c) => IR::CONST(*c),
                _ => panic!("Unexpected Register"),
            };

            match op {
                RelOp::NotEqual => compiler_context
                    .instructions
                    .push(Instruction::NEQ(outreg, a, b)),
                _ => unimplemented!(),
            }
        }
        Expr::Operand(operand) => {
            let inreg = match operand {
                Operand::Register(r) => IR::REG(*r),
                Operand::Constant(c) => IR::CONST(*c),
                _ => panic!("Unexpected Register!"),
            };
            compiler_context
                .instructions
                .push(Instruction::MOV(outreg, inreg));
        }
        Expr::StackPop => {}
        _ => unimplemented!(),
    };
}

fn compile_cjump(compiler_context: &mut CompilerContext, cond: &Expr, label: &usize) {
    let outreg = match cond {
        Expr::Operand(Operand::Register(r)) => IR::REG(*r),
        Expr::Operand(Operand::Constant(c)) => IR::CONST(*c),
        _ => panic!("expected register here!"),
    };

    let label = compiler_context.labels[label];
    compiler_context
        .instructions
        .push(Instruction::BT(outreg, label))
}
