use analysis::cfg::CFG;

mod allocation;
pub mod analysis;
mod convert;
mod optimization;

pub fn convert_cfg_to_ssa_form(cfg: &mut CFG) {
    convert::construct_ssa(cfg);
}

pub fn destruct_cfg_from_ssa_form(cfg: &mut CFG) {
    convert::destruct_ssa(cfg);
    optimization::constant_optimization(cfg);
    println!("{:?}", cfg);
}

pub fn register_allocation(cfg: &mut CFG) {
    allocation::register_allocation(cfg);
}
