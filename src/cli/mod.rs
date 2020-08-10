use crate::compiler::compile_ir;
use crate::compiler::control_flow::ControlFlowContext;
use crate::compiler::dominator::algorithm::compute_dominators;
use crate::compiler::liveness_analysis::interference_graph::InterferenceGraph;
use crate::compiler::liveness_analysis::liveness_analysis;
use crate::compiler::liveness_analysis::register_alloc::register_alloc;
use crate::compiler::optimization::constant_optimization::constant_optimization;
use crate::compiler::optimization::dead_code::dead_code_elimination;
use crate::compiler::ssa::convert_from_ssa;
use crate::compiler::ssa::convert_vars_to_ssa;
use crate::compiler::ssa::edge_split;
use crate::compiler::ssa::insert_phi_functions;
use crate::{
    compiler::three_address_code::{translate::translate_to_tac_ir, Operand},
    vm::VM,
};

use std::path::PathBuf;
use structopt::StructOpt;

/**
 *  [CLI Wrapper]
 *  StructOpt package wrapper for command line operations.
 *
 *  debug ( -d --debug ) :
 *      Displays debug information during execution such
 *      as the stack, locals, and executing instructions.
 *
 *  input: ( string ) :
 *      The path to the file that you with to compile.
 */
#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(short, long)]
    pub debug: bool,

    #[structopt(short = "o", long = "optimize")]
    pub optimize: bool,

    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

/**
 *  Parses command line input and determines what to do with it.
 *  
 *  Returns:
 *      Ok(()) : Execution had no problems.
 *      Err(e) : An error occured at some stage of execution.
 *
 */
pub fn parse_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    let content = std::fs::read_to_string(&args.input)?;
    build(content.as_str(), args)?;
    Ok(())
}

/**
 *  Begins to actually compile a user's source code, running through all
 *  of the intermediate passes and stopping if an error occured during
 *  execution.
 *
 *  Returns:
 *      Ok(()) : Execution had no problems.
 *      Err(e) : An error occured at some stage of execution.
 *
 */
fn build(buf: &str, args: Cli) -> Result<(), String> {
    use crate::parser::scanner::Scanner;
    use crate::parser::Parser;

    let mut scanner = Scanner::new(buf);
    let tokens = match scanner.tokenize() {
        Ok(t) => t,
        Err(e) => {
            println!("{}", e);
            return Err(format!("{}", e));
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            return Err(format!("{}", e));
        }
    };

    let statements = translate_to_tac_ir(ast);

    let mut contexts: Vec<ControlFlowContext> = vec![];
    for func in statements {
        let mut context = ControlFlowContext::new(func);

        compute_dominators(&mut context);
        insert_phi_functions(&mut context);
        convert_vars_to_ssa(&mut context);
        edge_split(&mut context);

        if args.optimize {
            dead_code_elimination(&mut context);
            constant_optimization(&mut context);
        }

        // if args.debug {
        //     println!("::ControlFlowGraph::\n{:?}", context.cfg);
        // }
        // context.cfg.clean();
        contexts.push(context);
    }

    let context = &mut contexts[0];
    let mut ig: InterferenceGraph = InterferenceGraph::new();
    convert_from_ssa(context);
    if args.debug {
        println!("::ControlFlowGraph::\n{:?}", context.cfg);
    }
    // convert_from_ssa(context);

    // Figure out how to do analysis for non-ssa
    liveness_analysis(context, &mut ig);
    // println!("ig.edges: {:?}", ig.graph.edges);
    // // println!("::InterferenceGraph::\n{:?}", ig);

    register_alloc(context, &mut ig);
    // println!("ig.colors: {:?}", ig.colors);
    // println!("ig.edges: {:?}", ig.graph.edges);

    for (operand, reg) in &ig.colors {
        context
            .cfg
            .replace_all_operand_with(*operand, Operand::Register(*reg));
    }

    if args.debug {
        // println!("::ControlFlowGraph::\n{:?}", context.cfg);
        // println!("::InterferenceGraph::\n{:?}", ig);
    }

    let compiled = compile_ir(context);
    if args.debug {
        println!("::Instruction Count:: {}", compiled.instructions.len());
        for instr in &compiled.instructions {
            println!("{:?}", instr);
        }
        print!("\n");
    }

    let mut vm: VM = VM::new();
    vm.dispatch(&compiled)?;

    Ok(())
}
