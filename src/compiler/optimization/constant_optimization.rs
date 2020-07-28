use crate::bytecode::constant::Constant;
use crate::compiler::control_flow::ControlFlowContext;

use crate::compiler::three_address_code::{Expr, Operand, Stmt};

// Appel p. 418 and 419
pub fn constant_optimization(context: &mut ControlFlowContext) {
    let mut w = context.cfg.get_statements();
    w.reverse();

    while !w.is_empty() {
        let s = w.pop().unwrap();

        // Constant Propagation
        if let Some((v, c)) = s.is_phi_constant() {
            for t in context.cfg.get_statements_using(v) {
                t.replace_operand_with(v, c);
                if !w.contains(&t) {
                    w.push(t.clone());
                }
            }
            context.cfg.remove_statement(s);
        } else if let Stmt::Tac(lval, Expr::Operand(Operand::Constant(constant))) = s {
            let c = Operand::Constant(constant);
            for t in context.cfg.get_statements_using(lval) {
                t.replace_operand_with(lval, c);
                if !w.contains(&t) {
                    w.push(t.clone());
                }
            }
            context.cfg.remove_statement(s);
        }
        // Copy Propagation
        else if let Stmt::Tac(lval, Expr::Operand(Operand::Assignable(var, offset, is_var))) = s {
            let y = Operand::Assignable(var, offset, is_var);
            for t in context.cfg.get_statements_using(lval) {
                t.replace_operand_with(lval, y);
                if !w.contains(&t) {
                    w.push(t.clone());
                }
            }
            context.cfg.remove_statement(s);
        }
        // Constant Folding
        else if let Stmt::Tac(_, Expr::Binary(Operand::Constant(a), op, Operand::Constant(b))) = s {
            let constant = Operand::Constant(a.compute_binary(op, b));
            let stmt = context.cfg.replace_statement_rval_with(s, constant);
            w.push(stmt);
        } else if let Stmt::Tac(_, Expr::Logical(Operand::Constant(a), op, Operand::Constant(b))) = s {
            let constant = Operand::Constant(a.compute_logical(op, b));
            let stmt = context.cfg.replace_statement_rval_with(s, constant);
            w.push(stmt);
        }
        // Constant Conditions
        else if let Stmt::CJump(Expr::Operand(Operand::Constant(Constant::Boolean(b))), _) = s {
            context.cfg.remove_conditional_jump(s, b, &mut w);
        }
    }
}
