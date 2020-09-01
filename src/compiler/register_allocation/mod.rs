use crate::compiler::flowgraph::cfg::CFG;
use crate::{compiler::ir::Oper, vm::NUM_REGISTERS};
use indexmap::IndexSet;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
};

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
    pub oper: Oper,
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}: [{}, {}]", self.oper, self.start, self.end)
    }
}

// Ah good ol' linear search!
fn compute_live_intervals(cfg: &CFG) -> Vec<Interval> {
    let mut intervals: HashMap<Oper, Interval> = HashMap::new();

    let mut s: usize = 1;

    for bb in &cfg.blocks {
        for statement in &bb.statements {
            let statement = &*statement.borrow();

            for def in statement.def() {
                match intervals.entry(def) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().end = s;
                    }
                    Entry::Vacant(entry) => {
                        let interval = Interval {
                            start: s,
                            end: s,
                            oper: def,
                        };
                        entry.insert(interval);
                    }
                }
            }

            for used in statement.used() {
                if let Entry::Occupied(mut entry) = intervals.entry(used) {
                    entry.get_mut().end = s;
                }
            }

            s += 1;
        }
    }

    intervals.values().cloned().collect::<Vec<Interval>>()
}

#[derive(Default, Clone)]
struct AllocState {
    active: Vec<Interval>,
    registers: IndexSet<usize>,
    register: HashMap<Oper, usize>,
    location: HashMap<Oper, usize>,
}

static mut STACK_LOC: usize = 1;

fn new_stack_loc() -> usize {
    unsafe {
        let x = STACK_LOC;
        STACK_LOC += 1;
        x
    }
}

// As much as I like the simplicity of the vanilla linear scan register alloc algo,
// it's pretty cheeks. Theres a better version of this but the literature looks confusing.
// This works for now but I should look into moving toward the better linear scan algo or go
// with a form of graph coloring w/ an interference graph.
fn linear_scan_register_allocation(cfg: &CFG) -> AllocState {
    fn expire_old_intervals(i: &Interval, state: &mut AllocState) {
        state
            .active
            .sort_by(|a, b| a.end.partial_cmp(&b.end).unwrap());

        let mut to_remove = vec![];
        for (index, j) in state.active.iter().enumerate() {
            if j.end >= i.start {
                return;
            } else {
                to_remove.push(index);
            }
        }

        for to_r in &to_remove {
            let oper = &state.active[*to_r].oper;
            let reg = *state.register.get(oper).unwrap();
            state.registers.insert(reg);
        }

        state.active = state
            .active
            .iter()
            .enumerate()
            .filter(|(i, _)| !to_remove.contains(i))
            .map(|(_, i)| *i)
            .collect();
    }

    fn spill_at_interval(i: &Interval, state: &mut AllocState) {
        let spill = state.active.last().unwrap();

        if spill.end > i.end {
            let spilled = *state.register.get(&spill.oper).unwrap();
            state.register.insert(i.oper, spilled);
            state.register.remove(&spill.oper);
            state.location.insert(spill.oper, 0);
            state.active.pop();
            state.active.push(*i);
            state
                .active
                .sort_by(|a, b| a.end.partial_cmp(&b.end).unwrap());
        } else {
            state.location.insert(i.oper, 0);
        }
    }

    let mut intervals = compute_live_intervals(cfg);
    // println!("intervals: {:#?}", intervals);
    intervals.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    let mut state = AllocState::default();
    state.registers = (0..NUM_REGISTERS).rev().collect();
    state.register = HashMap::new();
    state.active = Vec::new();

    for i in intervals {
        expire_old_intervals(&i, &mut state);
        if state.active.len() == NUM_REGISTERS {
            spill_at_interval(&i, &mut state);
        } else {
            state
                .register
                .insert(i.oper, state.registers.pop().unwrap());
            state.active.push(i);
            state
                .active
                .sort_by(|a, b| a.end.partial_cmp(&b.end).unwrap());
        }
    }
    state
}

pub fn register_allocation(cfg: &mut CFG) {
    let mut state = linear_scan_register_allocation(cfg);

    for (oper, reg) in state.register {
        let register = Oper::Register(reg);

        // This line may not be needed, investigate.
        state.location.remove(&oper);
        cfg.replace_all_operand_with(&oper, &register);
    }

    for (oper, _) in state.location {
        let loc = new_stack_loc();
        let stackloc = Oper::StackLocation(loc);
        cfg.replace_all_operand_with(&oper, &stackloc);
    }
}
