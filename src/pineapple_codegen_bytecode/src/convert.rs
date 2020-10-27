use crate::bytecode::IR;
use crate::{bytecode::Instruction, bytecode::OR, module::Module};
use pineapple_codegen_ssa::analysis::{basic_block::BlockEntry, basic_block::BlockExit, cfg::CFG};
use pineapple_ir::mir::Label;
use pineapple_ir::mir::Oper;
use pineapple_ir::mir::Stmt;
use pineapple_ir::op::RelOp;
use pineapple_ir::{mir::Expr, op::BinOp};
use pineapple_ir::{NoneTy, Value};

#[derive(Default)]
pub struct Compiler {
    module: Module,
}

impl Compiler {
    fn operand_to_or(&mut self, operand: &Oper) -> OR {
        match operand {
            Oper::Register(r) => OR::REG(*r),
            Oper::StackLocation(s) => OR::STACK(*s),
            _ => unimplemented!(),
        }
    }

    fn operand_to_ir(&mut self, operand: &Oper) -> IR {
        match operand {
            Oper::Register(r) => IR::REG(*r),
            Oper::Value(c) => {
                let value_location = self.module.add_value(*c);
                IR::VALUE(value_location)
            }
            Oper::StackLocation(s) => IR::STACK(*s),
            Oper::StackPop => IR::STACKPOP,
            Oper::ReturnValue => IR::RETVAL,
            _ => unimplemented!("{:?} not implemented", operand),
        }
    }

    pub fn compile_program(mut self, cfgs: Vec<CFG>) -> Module {
        for cfg in cfgs {
            self.module.add_chunk(cfg.entry_label);
            self.compile_cfg(cfg)
        }

        self.module
    }

    fn compile_cfg(&mut self, cfg: CFG) {
        for block in cfg.blocks {
            match block.entry {
                BlockEntry::Entry(statement) => self.compile_statement(&*statement.borrow()),
                BlockEntry::None => (),
            }

            for statement in block.statements {
                self.compile_statement(&*statement.borrow())
            }

            match block.exit {
                BlockExit::Exit(statement) => self.compile_statement(&*statement.borrow()),
                BlockExit::None => (),
            }
        }
    }

    fn compile_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Tac(lval, rval) => self.compile_tac(lval, rval),
            Stmt::Label(label) => self.compile_label(label),
            Stmt::Jump(label) => self.compile_jump(label),
            Stmt::CJump(cond, jumpto) => self.compile_cjump(cond, jumpto),
            Stmt::CastAs(oper, ty) => {
                let or = self.operand_to_or(oper);
                self.module.add_instruction(Instruction::CAST(or, *ty));
            }
            Stmt::Call(sym, arity) => self.compile_call(sym, arity),
            Stmt::StackPush(oper) => {
                let or = self.operand_to_ir(oper);
                self.module.add_instruction(Instruction::PUSH(or))
            }
            Stmt::Return(retval) => self.compile_return(retval),
            _ => unimplemented!(""),
        }
    }

    fn compile_tac(&mut self, lval: &Oper, rval: &Expr) {
        let lval = self.operand_to_or(lval);
        self.compile_expression(lval, rval);
    }

    fn compile_label(&mut self, label: &Label) {
        self.module.add_label(label);
    }

    fn compile_jump(&mut self, label: &Label) {
        self.module.add_instruction(Instruction::JUMP(*label));
    }

    fn compile_cjump(&mut self, cond: &Expr, jumpto: &Label) {
        let res = match cond {
            Expr::Oper(oper) => self.operand_to_ir(oper),
            _ => unimplemented!(),
        };

        self.module.add_instruction(Instruction::BT(res, *jumpto));
    }

    fn compile_call(&mut self, sym: &usize, arity: &usize) {
        self.module.add_instruction(Instruction::CALL(*sym, *arity));

        // This noop may actually not be needed, check this out...
        self.module.add_instruction(Instruction::NOP);
    }

    fn compile_return(&mut self, retval: &Option<Oper>) {
        if let Some(retval) = retval {
            let retval = self.operand_to_ir(retval);
            self.module.add_instruction(Instruction::RETURN(retval));
        } else {
            let retval = Value::from(NoneTy::None);
            let retval = self.operand_to_ir(&Oper::Value(retval));
            self.module.add_instruction(Instruction::RETURN(retval));
        }
    }

    fn compile_expression(&mut self, or: OR, expr: &Expr) {
        match expr {
            Expr::Binary(left, op, right) => self.compile_binary(or, left, op, right),
            Expr::Logical(left, op, right) => self.compile_logical(or, left, op, right),
            Expr::Oper(oper) => {
                let rval = self.operand_to_ir(oper);
                self.module.add_instruction(Instruction::MOV(or, rval));
            }
            _ => unimplemented!("{:?}", expr),
        }
    }

    fn compile_binary(&mut self, or: OR, left: &Oper, op: &BinOp, right: &Oper) {
        let a = self.operand_to_ir(left);
        let b = self.operand_to_ir(right);

        match op {
            BinOp::Plus => self.module.add_instruction(Instruction::ADD(or, a, b)),
            BinOp::Minus => self.module.add_instruction(Instruction::SUB(or, a, b)),
            BinOp::Star => self.module.add_instruction(Instruction::MUL(or, a, b)),
            BinOp::Slash => self.module.add_instruction(Instruction::DIV(or, a, b)),
            BinOp::Modulo => self.module.add_instruction(Instruction::MOD(or, a, b)),
            BinOp::Carat => self.module.add_instruction(Instruction::POW(or, a, b)),
            _ => unimplemented!(),
        }
    }

    fn compile_logical(&mut self, or: OR, left: &Oper, op: &RelOp, right: &Oper) {
        let a = self.operand_to_ir(left);
        let b = self.operand_to_ir(right);

        match op {
            RelOp::NotEqual => self.module.add_instruction(Instruction::NEQ(or, a, b)),
            RelOp::EqualEqual => self.module.add_instruction(Instruction::EQ(or, a, b)),
            RelOp::Less => self.module.add_instruction(Instruction::LT(or, a, b)),
            RelOp::LessEqual => self.module.add_instruction(Instruction::LTE(or, a, b)),
            RelOp::Greater => self.module.add_instruction(Instruction::GT(or, a, b)),
            RelOp::GreaterEqual => self.module.add_instruction(Instruction::GTE(or, a, b)),
        }
    }
}
