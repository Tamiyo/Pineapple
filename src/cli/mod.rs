use crate::compiler::algorithm;
use crate::compiler::control_flow::ControlFlowGraph;
use crate::compiler::optimization;
use crate::compiler::three_address::TacTranslator;

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

    let mut translator = TacTranslator::new();
    let statements = translator.translate(ast);

    let mut cfg = ControlFlowGraph::new(statements);

    algorithm::ssa::convert_cfg_to_ssa(&mut cfg);

    if args.optimize {
        optimization::dead_code::dead_code_elimination(&mut cfg);
        optimization::constant_optmization::constant_optimization(&mut cfg);
    }

    if args.debug {
        for (i, s) in cfg.gather_statements().into_iter().enumerate() {
            println!("{:<3}:{:>2}{:?}", i, "", s);
        }
    }

    Ok(())
}
