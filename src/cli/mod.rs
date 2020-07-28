use crate::compiler::compile_ir;
use crate::compiler::control_flow::ControlFlowContext;
use crate::compiler::dominator::algorithm::compute_dominators;
use crate::compiler::liveness_analysis::linear_scan_register_allocation;
use crate::compiler::static_single_assignment::convert_from_ssa;
use crate::compiler::static_single_assignment::convert_vars_to_ssa;
use crate::compiler::static_single_assignment::edge_split;
use crate::compiler::static_single_assignment::insert_phi_functions;
use crate::compiler::three_address_code::translate::translate_to_tac_ir;
use crate::vm::VM;

use crate::compiler::optimization::constant_optimization::constant_optimization;
use crate::compiler::optimization::dead_code::dead_code_elimination;
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

        convert_from_ssa(&mut context);
        linear_scan_register_allocation(&mut context);
        // let compiled = compile_ir(&context);
        // for (i, instr) in compiled.instructions.iter().enumerate() {
        //     println!("{:<3}:{:>2}{:?}", i, "", instr);
        // }

        if args.debug {
            for (i, s) in context.cfg.get_statements().into_iter().enumerate() {
                println!("{:<3}:{:>2}{:?}", i, "", s);
            }
        }

        contexts.push(context);
    }

    // this is temporary since we haven't introduced functions
    let context = &contexts[0];
    let compiled = compile_ir(context);
    let mut vm: VM = VM::new();
    vm.dispatch(&compiled)?;

    println!("{:?}", vm);

    Ok(())
}
