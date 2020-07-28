use crate::compiler::control_flow::ControlFlowContext;
use crate::compiler::liveness_analysis::live_interval::compute_live_intervals;
use crate::compiler::liveness_analysis::live_interval::Interval;
use crate::compiler::three_address_code::Operand;

use std::collections::HashMap;

pub mod live_interval;

// https://en.wikipedia.org/wiki/Register_allocation
pub fn linear_scan_register_allocation(context: &mut ControlFlowContext) {
    let intervals = compute_live_intervals(&context);
    let R: usize = 16; // Number of registers we have

    let mut register: HashMap<(Operand, Interval), usize> = HashMap::new();
    let mut registers: Vec<usize> = (0..R).rev().collect();

    let mut active: Vec<(Operand, Interval)> = Vec::new();

    for (operand, i) in intervals {
        {
            // ExpireOldInterval(J)
            let mut marked: Vec<usize> = Vec::new();
            for (ind, j) in active.iter().enumerate() {
                if j.1.end >= i.start {
                    break;
                }
                marked.push(ind);
                let reg = register[&j];
                registers.push(reg);
            }
            // Retain the values as specified in algorithm
            let mut i: usize = 0;
            active.retain(|_| {
                i += 1;
                marked.contains(&(i - 1))
            })
        }
        if active.len() == R {
            // SpillAtInterval(i)
            {}
        } else {
            register.insert((operand, i), registers.pop().unwrap());
            active.push((operand, i));
            active.sort_by(|(_, a2), (_, b2)| a2.start.partial_cmp(&b2.start).unwrap());
        }
    }

    for ((operand, _), reg) in register.iter() {
        context.cfg.replace_all_operand_with(*operand, Operand::Register(*reg))
    }

}
