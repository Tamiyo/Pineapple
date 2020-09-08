// TODO :- Implement the following optimizations at some point in the compilation process.
//         Ideally Optimization = fn(CFG) : transform the cfg in some way

// Phase 1 Optimization (Source Code)

//     1. Scalar replacement of array references

// Phase 2 Optimization (Medium Level IR)

//     1. Tail-call optimization
//     2. Local and global subexpression elimination
//     3. Loop-invariant analysis
//     4. Value propagation
//     5. Value Folding
//     6. Copy Propagation
//     7. Dead code elimination
//     8. Hoisting
//     9. Induction Variable analysis (maybe)
//     10. Control-flowgraph optimizations

// Phase 3 Optimzation (Low Level IR)

//     1. Loop unrolling
//     2. Dead code elimination
//     3. Branch prediction (maybe)
//     4. Branch optimizations
//     5. Register Allocation / Coalescing

use super::ir::{Expr, Oper};
use crate::compiler::flowgraph::cfg::CFG;
use crate::core::value::compute_binary;
use crate::core::value::compute_logical;
use crate::{compiler::ir::Stmt, core::value::Primitive};
use std::cell::RefCell;
use std::rc::Rc;

pub fn constant_optimization(cfg: &mut CFG) {
    let mut w = cfg.statements();
    w.reverse();

    while !w.is_empty() {
        let s: Rc<RefCell<Stmt>> = w.pop().unwrap();

        // Value Propagation
        if let Stmt::Tac(lval, Expr::Oper(oper)) = &*s.borrow() {
            if let Oper::Value(_) = oper {
                for t in cfg.statements_using(lval) {
                    t.borrow_mut().replace_all_oper_use_with(lval, oper);
                    if !w.contains(&t) {
                        w.push(Rc::clone(&t));
                    }
                }
                cfg.remove_statement(Rc::clone(&s));
            }
        };

        // Copy Propagation
        if let Stmt::Tac(lval, Expr::Oper(oper)) = &*s.borrow() {
            if let Oper::Var(_, _) | Oper::Temp(_, _) = oper {
                for t in cfg.statements_using(lval) {
                    t.borrow_mut().replace_all_oper_use_with(lval, oper);
                    if !w.contains(&t) {
                        w.push(Rc::clone(&t));
                    }
                }
                cfg.remove_statement(Rc::clone(&s));
            }
        };

        // Value Folding
        if let Stmt::Tac(_, rval) = &mut *s.borrow_mut() {
            if let Expr::Binary(left, op, right) = rval.clone() {
                match (left, right) {
                    (Oper::Value(a), Oper::Value(b)) => {
                        let Value = Oper::Value(compute_binary(a, op, b));
                        *rval = Expr::Oper(Value);
                        w.push(Rc::clone(&s));
                    }
                    _ => (),
                };
            } else if let Expr::Logical(left, op, right) = rval.clone() {
                match (left, right) {
                    (Oper::Value(a), Oper::Value(b)) => {
                        let Value = Oper::Value(compute_logical(a, op, b));
                        *rval = Expr::Oper(Value);
                        w.push(Rc::clone(&s));
                    }
                    _ => (),
                };
            };
        };

        // Value Conditions
        // This one kinda wonky rn, will look at it again
        {
            let mut label: Option<usize> = None;
            if let Stmt::CJump(Expr::Oper(operand), l) = &*((s.clone()).borrow()) {
                if let Oper::Value(value) = operand {
                    if let Primitive::Bool(b) = &value.inner {
                        let (mut modified_statements, jump_label) =
                            cfg.remove_conditional_jump(&s, *b, *l);
                        w.append(&mut modified_statements);

                        label = jump_label;
                    }
                }
            };

            if let Some(l) = label {
                s.replace(Stmt::Jump(l));
            };
        }
    }
}
