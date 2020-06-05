use crate::bytecode::constant::Constant;
use crate::compiler::control_flow::ControlFlowGraph;
use crate::compiler::three_address::component::Operand;

// Appel p. 418 and 419
pub fn constant_optimization(cfg: &mut ControlFlowGraph) {
    let mut w = cfg.gather_statements();
    w.reverse();

    while !w.is_empty() {
        let s = w.pop().unwrap();

        // Constant Propagation
        if let Some((v, c)) = s.is_phi_constant() {
            for t in cfg.gather_statements_using(v) {
                t.replace_with(v, c);
                if !w.contains(&t) {
                    w.push(t.clone());
                }
            }
            cfg.remove_statement(s);
        } else if let Some((v, c)) = s.is_constant() {
            for t in cfg.gather_statements_using(v) {
                t.replace_with(v, c);
                if !w.contains(&t) {
                    w.push(t.clone());
                }
            }
            cfg.remove_statement(s);
        }
        // Copy Propagation
        else if let Some((x, y)) = s.is_single_assignment() {
            for t in cfg.gather_statements_using(x) {
                t.replace_with(x, y);
                if !w.contains(&t) {
                    w.push(t.clone());
                }
            }
            cfg.remove_statement(s);
        }
        // Constant Folding
        else if let Some((a, op, b)) = s.is_constant_binary() {
            let constant = Operand::Constant(a.compute_binary(op, b));
            let stmt = cfg.replace_statement_rval_with(s, constant);
            w.push(stmt);
        } else if let Some((a, op, b)) = s.is_constant_logical() {
            let constant = Operand::Constant(a.compute_logical(op, b));
            let stmt = cfg.replace_statement_rval_with(s, constant);
            w.push(stmt);
        }
        // Constant Conditions
        else if let Some(Constant::Boolean(b)) = s.is_constant_jump() {
            cfg.remove_conditional_jump(s, b, &mut w);
        }
    }
}
