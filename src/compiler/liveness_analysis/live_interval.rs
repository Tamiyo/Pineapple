use crate::compiler::control_flow::ControlFlowContext;
use crate::compiler::three_address_code::Operand;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
}

pub fn compute_live_intervals(context: &ControlFlowContext) -> Vec<(Operand, Interval)> {
    let mut intervals: HashMap<Operand, Interval> = HashMap::new();

    for (i, statement) in context.cfg.get_statements().iter().enumerate() {
        for v in statement.vars_defined() {
            match intervals.get(&v) {
                None => intervals.insert(v, Interval { start: i, end: i }),
                _ => None,
            };
        }

        for v in statement.vars_used() {
            match intervals.get_mut(&v) {
                Some(interval) => interval.end = i,
                _ => (),
            };
        }
    }

    let mut intervals_vec: Vec<(Operand, Interval)> = Vec::new();
    for (k, v) in intervals.iter() {
        intervals_vec.push((*k, *v));
    }

    intervals_vec.sort_by(|(_, a2), (_, b2)| a2.start.partial_cmp(&b2.start).unwrap());
    intervals_vec
}
