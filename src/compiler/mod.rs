pub mod dominator;
pub mod flowgraph;
pub mod ir;
pub mod optimizer;
pub mod register_allocation;
pub mod transformer;

use crate::bytecode::string_intern::get_string;
use crate::bytecode::string_intern::intern_string;
use crate::bytecode::{Label, OpCode, IR, OR};
use crate::{
    compiler::flowgraph::cfg::CFG,
    parser::{binop::BinOp, relop::RelOp},
};
use ir::{Expr, Oper, Stmt};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

// This is interesting. Coming from mango where we didnt compile machine code / tac this processes was very
// different. This will probably change once I implement scores and/or closures. I'll be honest I don't like
// closures very much in an imperative langauge but meh it might be an interesting challenge.
pub struct CompilerContext {
    pub instructions: Vec<OpCode>,
    pub labels: HashMap<usize, usize>,
    pub named_labels: HashMap<usize, usize>,
    pub stack_offset: usize, // How many spaces on the stack we want to reserve
}

impl CompilerContext {
    pub fn new() -> Self {
        CompilerContext {
            instructions: Vec::new(),
            labels: HashMap::new(),
            named_labels: HashMap::new(),
            stack_offset: 0,
        }
    }
}

fn operand_to_ir(cctx: &mut CompilerContext, operand: &Oper) -> IR {
    let inreg = match operand {
        Oper::Register(r) => IR::REG(*r),
        Oper::Constant(c) => IR::CONST(*c),
        Oper::StackLocation(s) => {
            cctx.stack_offset = usize::max(cctx.stack_offset, *s);
            IR::STACK(*s)
        }
        Oper::StackPop => IR::STACKPOP,
        Oper::ReturnValue => IR::RETVAL,
        _ => panic!("Unexpected Register"),
    };
    inreg
}

pub fn compile_ir(cfgs: Vec<CFG>) -> CompilerContext {
    let mut cctx = CompilerContext::new();

    let mut func_cfgs = vec![];
    let mut main_cfgs = vec![];
    for cfg in cfgs {
        if let Some(Stmt::NamedLabel(name)) = cfg.blocks[0].label {
            if intern_string("main".to_string()) == name {
                main_cfgs.push(cfg);
            } else {
                func_cfgs.push(cfg);
            }
        }
    }

    let mut instr_count = 0;
    for cfg in &main_cfgs {
        let statements = cfg.statements();
        compile_statements(&mut cctx, &statements, &mut instr_count);
        let len = cctx.instructions.len();
        cctx.instructions.insert(len - 1, OpCode::HLT);
    }

    for cfg in &func_cfgs {
        let statements = cfg.statements();
        compile_statements(&mut cctx, &statements, &mut instr_count);
    }

    for (i, instr) in cctx.instructions.iter().enumerate() {
        match instr {
            OpCode::LABEL(Label::Label(label)) => {
                cctx.labels.insert(*label, i);
            }
            OpCode::LABEL(Label::Named(label)) => {
                cctx.named_labels.insert(*label, i);
            }
            _ => (),
        }
    }

    cctx
}

fn compile_statements(
    cctx: &mut CompilerContext,
    statements: &Vec<Rc<RefCell<Stmt>>>,
    instr_count: &mut usize,
) {
    for statement in statements.iter() {
        match &*statement.borrow() {
            Stmt::Tac(lval, rval) => compile_tac(cctx, lval, rval),
            Stmt::Label(label) => {
                cctx.instructions
                    .push(OpCode::LABEL(crate::bytecode::Label::Label(*label)));
            }
            Stmt::NamedLabel(label) => {
                cctx.instructions
                    .push(OpCode::LABEL(crate::bytecode::Label::Named(*label)));
            }
            Stmt::Jump(label) => {
                cctx.instructions
                    .push(OpCode::JUMP(crate::bytecode::Label::Label(*label)))
            }
            Stmt::JumpNamed(label) => {
                cctx.instructions
                    .push(OpCode::JUMPR(crate::bytecode::Label::Named(*label)));
            }
            Stmt::CJump(cond, label) => compile_cjump(cctx, cond, label),
            Stmt::StackPush(operand) => {
                let inreg = operand_to_ir(cctx, operand);
                cctx.instructions.push(OpCode::PUSH(inreg))
            }
            Stmt::StackPushAllReg => cctx.instructions.push(OpCode::PUSHA),
            Stmt::StackPopAllReg => cctx.instructions.push(OpCode::POPA),
            Stmt::Call(intern, arity) => cctx.instructions.push(OpCode::CALL(*intern, *arity)),
            Stmt::Return(oper) => {
                let inreg = operand_to_ir(cctx, oper);
                cctx.instructions.push(OpCode::RETURN(inreg));
            }
            // Stmt::Expr(expr) => match expr {
            //     Expr::Oper(Oper::Call(intern, arity)) => {
            //         cctx.instructions.push(OpCode::CALL(*intern, *arity))
            //     }
            //     _ => unimplemented!(""),
            // },
            Stmt::Expr(expr) => cctx.instructions.push(OpCode::NOP),
            _ => unimplemented!("{:?} isnt implemented", statement),
        }
        *instr_count += 1;
    }
}

fn compile_tac(cctx: &mut CompilerContext, lval: &Oper, rval: &Expr) {
    let outreg = match lval {
        Oper::Register(r) => OR::REG(*r),
        Oper::StackLocation(s) => {
            cctx.stack_offset = usize::max(cctx.stack_offset, *s);
            OR::STACK(*s)
        }
        _ => panic!("Expected a register as output!"),
    };

    match rval {
        Expr::Binary(a, op, b) => {
            let a = operand_to_ir(cctx, a);
            let b = operand_to_ir(cctx, b);

            match op {
                BinOp::Plus => cctx.instructions.push(OpCode::ADD(outreg, a, b)),
                BinOp::Minus => cctx.instructions.push(OpCode::SUB(outreg, a, b)),
                BinOp::Star => cctx.instructions.push(OpCode::MUL(outreg, a, b)),
                BinOp::Slash => cctx.instructions.push(OpCode::DIV(outreg, a, b)),
                BinOp::Modulo => cctx.instructions.push(OpCode::MOD(outreg, a, b)),
                BinOp::Carat => cctx.instructions.push(OpCode::POW(outreg, a, b)),
                _ => unimplemented!(),
            }
        }
        Expr::Logical(a, op, b) => {
            let a = operand_to_ir(cctx, a);
            let b = operand_to_ir(cctx, b);

            match op {
                RelOp::NotEqual => cctx.instructions.push(OpCode::NEQ(outreg, a, b)),
                RelOp::EqualEqual => cctx.instructions.push(OpCode::EQ(outreg, a, b)),
                RelOp::Less => cctx.instructions.push(OpCode::LT(outreg, a, b)),
                RelOp::LessEqual => cctx.instructions.push(OpCode::LTE(outreg, a, b)),
                RelOp::Greater => cctx.instructions.push(OpCode::GT(outreg, a, b)),
                RelOp::GreaterEqual => cctx.instructions.push(OpCode::GTE(outreg, a, b)),
                _ => unimplemented!(),
            }
        }
        Expr::Oper(operand) => {
            let inreg = operand_to_ir(cctx, operand);
            cctx.instructions.push(OpCode::MOV(outreg, inreg));
        }
        // Expr::StackPop => {}
        _ => unimplemented!(),
    };
}

fn compile_cjump(cctx: &mut CompilerContext, cond: &Expr, label: &usize) {
    let inreg = match cond {
        Expr::Oper(oper) => operand_to_ir(cctx, oper),
        _ => panic!("expected register here!"),
    };

    cctx.instructions
        .push(OpCode::BT(inreg, crate::bytecode::Label::Label(*label)))
}
