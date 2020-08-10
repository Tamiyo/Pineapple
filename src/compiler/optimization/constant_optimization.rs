use crate::bytecode::constant::Constant;
use crate::compiler::control_flow::ControlFlowContext;
use core::cell::RefCell;

use crate::compiler::three_address_code::{Expr, Operand, Stmt};
use std::rc::Rc;

// Appel p. 418 and 419
pub fn constant_optimization(context: &mut ControlFlowContext) {
    let mut w = context.cfg.get_statements();
    w.reverse();

    while !w.is_empty() {
        let s: Rc<RefCell<Stmt>> = w.pop().unwrap();

        if let Some((v, c)) = s.borrow().is_phi_constant() {
            for t in context.cfg.get_statements_using(v) {
                t.borrow_mut().replace_operand_with(v, c);
                if !w.contains(&t) {
                    w.push(t.clone());
                }
            }
            context.cfg.remove_statement(Rc::clone(&s));
        };

        if let Stmt::Tac(lval, Expr::Operand(operand)) = &*s.borrow() {
            if let Operand::Constant(_) = operand {
                for t in context.cfg.get_statements_using(*lval) {
                    t.borrow_mut().replace_operand_with(*lval, *operand);
                    if !w.contains(&t) {
                        w.push(Rc::clone(&t));
                    }
                }
                context.cfg.remove_statement(Rc::clone(&s));
            }
        };

        // Copy Propagation
        if let Stmt::Tac(lval, Expr::Operand(operand)) = &*s.borrow() {
            if let Operand::Assignable(_, _, _) = operand {
                for t in context.cfg.get_statements_using(*lval) {
                    t.borrow_mut().replace_operand_with(*lval, *operand);
                    if !w.contains(&t) {
                        w.push(Rc::clone(&t));
                    }
                }
                context.cfg.remove_statement(Rc::clone(&s));
            }
        };

        // Constant Folding
        if let Stmt::Tac(_, rval) = &mut *s.borrow_mut() {
            if let Expr::Binary(left, op, right) = rval.clone() {
                match (left, right) {
                    (Operand::Constant(a), Operand::Constant(b)) => {
                        let constant = Operand::Constant(a.compute_binary(op, b));
                        *rval = Expr::Operand(constant);
                        w.push(Rc::clone(&s));
                    }
                    _ => (),
                };
            } else if let Expr::Logical(left, op, right) = rval.clone() {
                match (left, right) {
                    (Operand::Constant(a), Operand::Constant(b)) => {
                        let constant = Operand::Constant(a.compute_logical(op, b));
                        *rval = Expr::Operand(constant);
                        w.push(Rc::clone(&s));
                    }
                    _ => (),
                };
            }
        };

        // Constant Conditions
        if let Stmt::CJump(Expr::Operand(operand), _) = &*s.borrow() {
            if let Operand::Constant(Constant::Boolean(b)) = operand {
                context
                    .cfg
                    .remove_conditional_jump(Rc::clone(&s), *b, &mut w);
            }
        };
    }
}
