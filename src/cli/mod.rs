use crate::compiler::dominator::compute_dominator_context;
use crate::compiler::flowgraph::cfg::CFG;
use crate::compiler::ir::ssa::construct_ssa;
use crate::compiler::ir::ssa::destruct_ssa;
use crate::compiler::optimizer::constant_optimization;
use crate::compiler::register_allocation::register_allocation;
use crate::compiler::transformer::linearcode::LinearCodeTransformer;
use crate::compiler::Compiler;
use crate::parser::parse_program;
use crate::parser::scanner::Scanner;
use crate::semantic_analysis::binding_check_ast;
use crate::semantic_analysis::type_check_ast;
use crate::vm::VM;
use std::time::Instant;
// use crate::parser::Parser;
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

    #[structopt(short, long)]
    pub profile: bool,

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
            return Err(format!("{}", e));
        }
    };

    let start = Instant::now();
    let ast = match parse_program(tokens) {
        Ok(a) => a,
        Err(e) => {
            return Err(format!("{}", e));
        }
    };
    let duration = start.elapsed();

    if args.profile {
        println!(
            "{:<24}{:}μs",
            "Build AST",
            format!("{:?}", duration.as_micros())
        );
    }

    let start = Instant::now();
    match type_check_ast(&ast) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("{}", e));
        }
    }

    match binding_check_ast(&ast) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("{}", e));
        }
    }
    let duration = start.elapsed();

    if args.profile {
        println!(
            "{:<24}{}μs",
            "Semantic Analysis",
            format!("{:?}", duration.as_micros())
        );
    }

    let start = Instant::now();
    let mut lcs = LinearCodeTransformer::new();
    let linear_code_blocks = lcs.translate(ast);
    let duration = start.elapsed();

    if args.profile {
        println!(
            "{:<24}{}μs",
            "AST to LinearCode",
            format!("{:?}", duration.as_micros())
        );
    }

    // cfg performance
    let mut opt_duration: u128 = 0;

    let start = Instant::now();
    let mut cfgs: Vec<CFG> = vec![];
    for linear_code in linear_code_blocks {
        let mut cfg = CFG::from(&linear_code);

        compute_dominator_context(&mut cfg);
        construct_ssa(&mut cfg);

        if args.optimize {
            let opt_start = Instant::now();
            constant_optimization(&mut cfg);
            opt_duration += opt_start.elapsed().as_micros();
            if args.debug {
                println!("::CFG OPTIMIZED::");
                println!("{:?}", cfg);
            }
        } else {
            if args.debug {
                println!("::CFG::");
                println!("{:?}", cfg);
            }
        }
        
        destruct_ssa(&mut cfg);
        register_allocation(&mut cfg);

        cfgs.push(cfg);
    }
    let duration = start.elapsed();

    if args.profile {
        println!(
            "{:<24}{}μs",
            "CFGs & Optimization",
            format!("{:?}", duration.as_micros())
        );
    }

    let start = Instant::now();
    let module = {
        let mut compiler = Compiler::new();
        compiler.compile_ir_to_bytecode(cfgs)
    };
    let duration = start.elapsed();
    if args.profile {
        println!(
            "{:<24}{}μs\n",
            "Bytecode Compilation",
            format!("{:?}", duration.as_micros())
        );
    }

    if args.debug {
        for chunk in module.chunks.iter() {
            for (index, instr) in chunk.instructions.iter().enumerate() {
                println!("{}:\t{:?}", index, instr);
            }
            println!("")
        }
    }

    let mut vm = VM::new();
    let start = Instant::now();
    match vm.dispatch(&module) {
        Ok(()) => (),
        Err(e) => panic!(e),
    };
    let duration = start.elapsed();

    if args.profile {
        println!(
            "\n{:<24}{}μs",
            "VM Execution",
            format!("{:?}", duration.as_micros())
        );
    }

    Ok(())
}
