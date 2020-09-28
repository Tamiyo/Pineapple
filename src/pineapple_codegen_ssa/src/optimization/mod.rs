use pineapple_ir::mir::{Expr, Oper, Stmt};

use crate::CFG;

pub fn constant_optimization(cfg: &mut CFG) {
    let mut w = cfg.active_statements();
    w.reverse();

    while !w.is_empty() {
        let s_index: usize = w.pop().unwrap();
        let s = cfg.statements[s_index].clone();

        // Copy Propagation
        if let Stmt::Tac(lval, Expr::Oper(oper)) = &*s.borrow() {
            if let Oper::SSA(_) = oper {
                for t in cfg.get_statements_using_oper(lval) {
                    cfg.statements[t]
                        .borrow_mut()
                        .replace_all_oper_use_with(lval, oper);
                    if !w.contains(&t) {
                        w.push(t);
                    }
                }
                cfg.remove_statement(s_index);
            }
        };
    }
}
