use pineapple_codegen_ssa::analysis::cfg::CFG;
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Instant;

use pineapple_ast::ast::Stmt;
use pineapple_ir::hir::token::Token;
use structopt::StructOpt;

#[macro_export]
macro_rules! benchmark {
    ($name: expr, $code:expr) => {
        let start = Instant::now();
        let x = $code;
        let duration = start.elapsed();

        PERF_METRICS.with(|m| {
            m.borrow_mut().push(format!(
                "{:<24}{:}μs",
                $name,
                format!("{:?}", duration.as_micros())
            ))
        });
        x
    };
}

thread_local! {
    static PERF_METRICS: RefCell<Vec<String>> = RefCell::new(vec![]);
}

#[derive(Debug, Default, StructOpt)]
pub struct PassArgs {
    #[structopt(short, long)]
    pub debug: bool,

    #[structopt(short, long)]
    pub perf: bool,

    #[structopt(short = "o", long = "optimize")]
    pub optimize: bool,

    #[structopt(parse(from_os_str))]
    pub input: PathBuf,
}

pub fn compile(buf: &str, args: PassArgs) {
    let tokens = lexical_pass(buf, &args);
    if args.debug {
        println!("::Lexical Analysis::\n{:#?}\n", tokens);
    }

    let mut ast = ast_pass(tokens, &args);
    if args.debug {
        println!("::AST Creation::\n{:#?}\n", ast);
    }

    typcheck_pass(&mut ast, &args);
    if args.debug {
        println!("::Type Checking::\n{:#?}\n", ast);
    }

    let mut linear_code = linear_code_pass(ast, &args);
    if args.debug {
        println!("::AST to LinearCode::\n{:#?}\n", linear_code);
    }

    codegen_ssa_pass(linear_code, &args);

    if args.perf {
        PERF_METRICS.with(|m| println!("{:#?}", m.borrow()));
    }
}

fn lexical_pass(buf: &str, args: &PassArgs) -> Vec<pineapple_ir::hir::token::Token> {
    let code = || match pineapple_lexer::lex(buf) {
        Ok(tokens) => tokens,
        Err(e) => panic!(format!("{}", e)),
    };

    if args.perf {
        benchmark! {
            "Lexical Analysis",
            code()
        }
    } else {
        code()
    }
}

fn ast_pass(tokens: Vec<Token>, args: &PassArgs) -> Vec<pineapple_ast::ast::Stmt> {
    let code = || match pineapple_ast::parse(tokens) {
        Ok(ast) => ast,
        Err(e) => panic!(format!("{}", e)),
    };

    if args.perf {
        benchmark! {
            "AST Creation",
            code()
        }
    } else {
        code()
    }
}

fn typcheck_pass(ast: &mut Vec<Stmt>, args: &PassArgs) {
    let mut code = || match pineapple_semantics::typecheck(ast) {
        Ok(ast) => ast,
        Err(e) => panic!(format!("{}", e)),
    };

    if args.perf {
        benchmark! {
            "Type Checking",
            code()
        }
    } else {
        code()
    }
}

fn linear_code_pass(ast: Vec<Stmt>, args: &PassArgs) -> Vec<Vec<pineapple_ir::mir::Stmt>> {
    let code = || pineapple_translation::convert_ast_to_linear_code(ast);

    if args.perf {
        benchmark! {
            "AST to LinearCode",
            code()
        }
    } else {
        code()
    }
}

fn codegen_ssa_pass(linear_code: Vec<Vec<pineapple_ir::mir::Stmt>>, args: &PassArgs) {
    for compilable_block in linear_code {
        let mut cfg = CFG::from(compilable_block);

        if args.debug {
            println!("{:#?}", cfg);
        }
    }
}
