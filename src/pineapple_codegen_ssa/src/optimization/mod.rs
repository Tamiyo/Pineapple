use std::rc::Rc;

use pineapple_ir::mir::{Expr, Oper, Stmt};

use crate::CFG;

pub fn constant_optimization(cfg: &mut CFG) {
    let mut w = cfg.active_statements();
    w.reverse();

    while !w.is_empty() {
        let statement = w.pop().unwrap();

        // Copy Propagation
        if let Stmt::Tac(lval, Expr::Oper(oper)) = &*statement.borrow() {
            if let Oper::SSA(_) = oper {
                for t in cfg.get_statements_using_oper(lval) {
                    match t.try_borrow_mut() {
                        Ok(mut refmut) => {
                            refmut.replace_all_oper_use_with(lval, oper);
                            if !w.contains(&t) {
                                w.push(Rc::clone(&t));
                            }
                        }
                        _ => (),
                    }
                    
                }
                cfg.remove_statement(Rc::clone(&statement));
            }
        };
    }
}
