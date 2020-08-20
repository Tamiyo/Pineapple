use crate::{
    compiler::{flowgraph::cfg::CFG, transformer::linearcode::LinearCodeTransformer},
    parser::{scanner::Scanner, Parser},
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

    let mut lcs = LinearCodeTransformer::new();
    let linear_code_blocks = lcs.translate(ast);

    for linear_code in linear_code_blocks {
        let mut cfg = CFG::new();
        cfg.generate_from_linear_code(&linear_code);
        cfg.compute_dom();
        cfg.compute_idom();
        cfg.compute_domf();

        cfg.insert_phi_functions();
        cfg.construct_ssa();

        if args.debug {
            println!("::Construct SSA::");
            println!("{:?}", cfg);
        }

        cfg.destruct_ssa();

        if args.debug {
            println!("::Destruct SSA::");
            println!("{:?}", cfg);
        }
    }

    Ok(())
}
