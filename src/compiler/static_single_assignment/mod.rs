use crate::bytecode::string_intern::get_string;
use crate::compiler::control_flow::basic_block::BasicBlock;
use crate::compiler::control_flow::ControlFlowContext;
use crate::compiler::three_address_code::{Expr, Operand, Stmt};
use std::collections::HashMap;
use std::collections::HashSet;

// Appel p. 407
pub fn insert_phi_functions(context: &mut ControlFlowContext) {
    let mut defsites: HashMap<Operand, HashSet<usize>> = HashMap::new();
    let mut a_orig: Vec<HashSet<Operand>> = vec![HashSet::new(); context.cfg.blocks.len()];
    let mut a_phi: Vec<HashSet<Operand>> = vec![HashSet::new(); context.cfg.blocks.len()];

    for (i, n) in context.cfg.blocks.iter().enumerate() {
        for a in n.get_variables_defined() {
            a_orig[i].insert(a);
            defsites.entry(a).or_insert_with(HashSet::new).insert(i);
        }
    }

    for a in &context.cfg.get_variables() {
        let (value, is_var) = match a {
            Operand::Assignable(value, _, is_var) => (*value, *is_var),
            _ => panic!("Expected SSA Value"),
        };

        let mut w: Vec<usize> = defsites.get(a).unwrap().iter().copied().collect();

        while !w.is_empty() {
            let n = w.pop().unwrap();
            for y in context.dominator.get_frontier(n) {
                if !a_phi[*y].contains(a) && is_var {
                    let mut phivec: Vec<Operand> = Vec::new();
                    for _ in &context.cfg.graph.pred[y] {
                        phivec.push(Operand::Assignable(value, 0, is_var));
                    }
                    context.cfg.blocks[*y].statements.insert(
                        1,
                        Stmt::Tac(Operand::Assignable(value, 0, is_var), Expr::Phi(phivec)),
                    );

                    a_phi[*y].insert(*a);
                    if !a_orig[*y].contains(a) {
                        w.push(*y);
                    }
                }
            }
        }
    }
}

// Appel p. 409
pub fn convert_vars_to_ssa(context: &mut ControlFlowContext) {
    let mut count: HashMap<(usize, bool), usize> = HashMap::new();
    let mut stack: HashMap<(usize, bool), Vec<usize>> = HashMap::new();
    for a in &context.cfg.get_variables() {
        let (value, is_var) = match a {
            Operand::Assignable(value, offset, is_var) => (*value, *is_var),
            _ => panic!("Expected SSA Value"),
        };

        count.insert((value, is_var), 0);
        stack.insert((value, is_var), vec![0]);
    }

    rename(context, 0, &mut count, &mut stack);
}

// Appel p. 409
fn rename(
    context: &mut ControlFlowContext,
    n: usize,
    count: &mut HashMap<(usize, bool), usize>,
    stack: &mut HashMap<(usize, bool), Vec<usize>>,
) {
    for s in &mut context.cfg.blocks[n].statements {
        // if s is not a Φ function, replace the variables
        for x in s.vars_used() {
            let (value, is_var) = match x {
                Operand::Assignable(value, _, is_var) => (value, is_var),
                _ => panic!("Expected SSA Value"),
            };

            let top = stack
                .get(&(value, is_var))
                // .expect(format!("Unexpected Identifier: {}", get_string(value)).as_str())
                .unwrap()
                .last();
            if let Some(i) = top {
                s.replace_var_use_with_ssa(value, *i, is_var)
            }
        }

        // update count definition
        for a in s.vars_defined() {
            let (value, is_var) = match a {
                Operand::Assignable(value, offset, is_var) => (value, is_var),
                _ => panic!("Expected SSA Value"),
            };

            let i = count.get(&(value, is_var)).unwrap();
            stack.get_mut(&(value, is_var)).unwrap().push(*i);
            s.replace_var_def_with_ssa(value, *i, is_var);
            *count.get_mut(&(value, is_var)).unwrap() += 1;
        }
    }

    // update successors
    for y in &context.cfg.graph.succ[&n] {
        let j = context.cfg.graph.pred[y]
            .iter()
            .position(|&r| r == n)
            .unwrap();
        for stmt in &mut context.cfg.blocks[*y].statements {
            if let Stmt::Tac(_, Expr::Phi(args)) = stmt {
                let (value, is_var) = match &args[0] {
                    Operand::Assignable(value, offset, is_var) => (*value, *is_var),
                    _ => panic!("expected assignable"),
                };
                if let Some(i) = stack.get(&(value, is_var)).expect("").last() {
                    args[j] = Operand::Assignable(value, *i, is_var);
                }
            }
        }
    }

    // rename children
    let mut children: Vec<usize> = vec![];
    for (i, domi) in context.dominator.immediate.iter().enumerate() {
        if domi.contains(&n) {
            children.push(i);
        }
    }
    for x in children {
        rename(context, x, count, stack);
    }

    for s in &context.cfg.blocks[n].statements {
        for a in s.vars_defined() {
            let (value, is_var) = match a {
                Operand::Assignable(value, offset, is_var) => (value, is_var),
                _ => panic!("Expected SSA Value"),
            };

            stack
                .get_mut(&(value, is_var))
                .expect("Expect var to exist")
                .pop();
        }
    }
}

// Appel p. 408
pub fn edge_split(context: &mut ControlFlowContext) {
    for a in 0..context.cfg.blocks.len() {
        for b in context.cfg.graph.succ[&a].clone() {
            if context.cfg.graph.succ[&a].len() > 1 && context.cfg.graph.pred[&b].len() > 1 {
                let pos = context.cfg.graph.succ[&a]
                    .iter()
                    .position(|&r| r == b)
                    .unwrap();
                context.cfg.graph.succ.get_mut(&a).unwrap().remove(&pos);

                let pos = context.cfg.graph.pred[&b]
                    .iter()
                    .position(|&r| r == a)
                    .unwrap();
                context.cfg.graph.pred.get_mut(&b).unwrap().remove(&pos);

                let z = context.cfg.blocks.len();
                context.cfg.blocks.push(BasicBlock::new(&[]));
                context.cfg.graph.succ.get_mut(&a).unwrap().insert(z);
                context.cfg.graph.pred.get_mut(&b).unwrap().insert(z);
            }
        }
    }
}

pub fn convert_from_ssa(context: &mut ControlFlowContext) {
    for block in &mut context.cfg.blocks {
        let mut phi_stmts: Vec<Stmt> = Vec::new();

        for statement in &mut block.statements {
            for var in statement.vars_defined() {
                let (value, is_var) = match var {
                    Operand::Assignable(v, _, i) => (v, i),
                    _ => panic!("Expected SSA Value"),
                };
                statement.replace_var_def_with_ssa(value, 0, is_var)
            }

            for var in statement.vars_used() {
                let (value, is_var) = match var {
                    Operand::Assignable(v, _, i) => (v, i),
                    _ => panic!("Expected SSA Value"),
                };
                statement.replace_var_use_with_ssa(value, 0, is_var)
            }

            if let Stmt::Tac(_, Expr::Phi(_)) = statement {
                phi_stmts.push(statement.clone());
            }
        }

        for statement in phi_stmts {
            block.remove_statement(&statement);
        }
    }
}
