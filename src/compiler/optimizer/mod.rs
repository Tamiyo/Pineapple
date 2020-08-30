use crate::compiler::flowgraph::cfg::CFG;
use crate::{
    bytecode::constant::Constant,
    compiler::ir::{Expr, Oper, Stmt},
};
use core::cell::RefCell;
use std::rc::Rc;

type Statement = Rc<RefCell<Stmt>>;

// TODO :- Implement the following optimizations at some point in the compilation process.
//         Ideally Optimization = fn(CFG) : transform the cfg in some way

// Phase 1 Optimization (Source Code)

//     1. Scalar replacement of array references

// Phase 2 Optimization (Medium Level IR)

//     1. Tail-call optimization
//     2. Local and global subexpression elimination
//     3. Loop-invariant analysis
//     4. Constant propagation
//     5. Constant Folding
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
