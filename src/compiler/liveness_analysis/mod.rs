use super::{
    control_flow::ControlFlowContext,
    three_address_code::{Expr, Stmt},
};
use crate::compiler::liveness_analysis::interference_graph::InterferenceGraph;
use crate::compiler::three_address_code::Operand;

pub mod interference_graph;
pub mod register_alloc;

// Appel p. 220 (Register Allocation)
// Appel p. 428 (Inference Graph Construction)

pub fn liveness_analysis(ctx: &ControlFlowContext, ig: &mut InterferenceGraph) {
    let mut _in: Vec<Vec<Operand>> = vec![];
    let mut _out: Vec<Vec<Operand>> = vec![];

    let mut _in_p: Vec<Vec<Operand>> = vec![];
    let mut _out_p: Vec<Vec<Operand>> = vec![];

    let mut _use: Vec<Vec<Operand>> = vec![];
    let mut _def: Vec<Vec<Operand>> = vec![];

    for n in 0..ctx.cfg.blocks.len() {
        _in.push(Vec::new());
        _out.push(Vec::new());

        _in_p.push(Vec::new());
        _out_p.push(Vec::new());

        _use.push(ctx.cfg.blocks[n].used());
        _def.push(ctx.cfg.blocks[n].def());
    }

    let mut i = 0;

    loop {
        for n in 0..ctx.cfg.blocks.len() {
            _in_p[n] = _in[n].clone();
            _out_p[n] = _out[n].clone();

            _out[n].clear();
            for s in &ctx.cfg.graph.succ[&n] {
                for y in &_in[*s] {
                    if !_out[n].contains(y) {
                        _out[n].push(*y);
                    }
                }
            }

            let mut diff = vec![];
            for a in &_out[n] {
                if !_def[n].contains(a) {
                    diff.push(*a);
                }
            }
            
            // let union = _use[n].union(&diff).cloned().collect::<Vec<Operand>>();
            let mut union = vec![];
            for u in &_use[n] {
                union.push(*u);
            }

            for d in &diff {
                if !union.contains(d) {
                    union.push(*d);
                }
            }

            _in[n] = union;
        }

        let mut converged = true;
        if _in != _in_p || _out != _out_p {
            converged = false;
        }

        if converged {
            println!("converged after '{:?}' iterations", i);
            break;
        }
        i += 1;
    }

    // println!("in: {:?}\nout: {:?}", _in, _out);
    construct_interference_graph(_out, _def, ctx, ig);
}

fn construct_interference_graph(
    _out: Vec<Vec<Operand>>,
    _def: Vec<Vec<Operand>>,
    ctx: &ControlFlowContext,
    ig: &mut InterferenceGraph,
) {
    for n in 0..ctx.cfg.blocks.len() {
        for def in &_def[n] {
            for temp in &_out[n] {
                if def != temp {
                    ig.graph.insert(*def);
                    ig.graph.insert(*temp);

                    if !ig.colors.contains_key(def) {
                        ig.colors.insert(*def, 0);
                    }

                    if !ig.colors.contains_key(temp) {
                        ig.colors.insert(*temp, 0);
                    }

                    ig.graph.add_edge(*def, *temp);
                }
            }
        }
    }
}
