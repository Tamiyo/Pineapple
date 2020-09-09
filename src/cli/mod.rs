use crate::bytecode::module::Module;
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

macro_rules! benchmark {
    ($name: expr, $code:expr) => {
        let start = Instant::now();
        $code
        let duration = start.elapsed();
        println!("{:<24}{:}μs", $name, format!("{:?}", duration.as_micros()));
    };
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

    let mut ast = vec![];
    benchmark!("Build AST", {
        ast = match parse_program(tokens) {
            Ok(a) => a,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };
    });

    let start = Instant::now();
    benchmark!("Semantic Analysis", {
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
    });

    let mut linear_code_blocks = vec![];
    benchmark!("AST to LinearCode", {
        let mut lcs = LinearCodeTransformer::new();
        linear_code_blocks = lcs.translate(ast);
    });

    let mut cfgs: Vec<CFG> = vec![];
    benchmark!("CFGs & Optimization", {
        for linear_code in linear_code_blocks {
            let mut cfg = CFG::from(&linear_code);

            compute_dominator_context(&mut cfg);
            construct_ssa(&mut cfg);

            if args.optimize {
                constant_optimization(&mut cfg);
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
    });

    let mut module = Module::new();
    benchmark!("Bytecode Compilation", {
        module = {
            let mut compiler = Compiler::new();
            compiler.compile_ir_to_bytecode(cfgs)
        };
    });

    if args.debug {
        for chunk in module.chunks.iter() {
            for (index, instr) in chunk.instructions.iter().enumerate() {
                println!("{}:\t{:?}", index, instr);
            }
            println!("")
        }
    }

    let mut vm = VM::new();
    match vm.dispatch(&module) {
        Ok(()) => (),
        Err(e) => panic!(e),
    };

    Ok(())
}
