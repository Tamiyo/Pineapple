use pineapple_ast::ast::Stmt;
use pineapple_ir::token::Token;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

#[macro_export]
macro_rules! benchmark {
    ($name: expr, $code:expr) => {
        let start = Instant::now();
        let x = $code;
        let duration = start.elapsed();
        println!("{:<24}{:}μs", $name, format!("{:?}", duration.as_micros()));
        x
    };
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
    let ast = ast_pass(tokens, &args);
    typcheck_pass(&ast, &args);
}

fn lexical_pass(buf: &str, args: &PassArgs) -> Vec<Token> {
    let code = || match pineapple_lexer::lex(buf) {
        Ok(tokens) => {
            if args.debug {
                println!("::Lexical Analysis::\n{:#?}\n", tokens);
            }
            tokens
        }
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

fn ast_pass(tokens: Vec<Token>, args: &PassArgs) -> Vec<Stmt> {
    let code = || match pineapple_ast::parse(tokens) {
        Ok(ast) => {
            if args.debug {
                println!("::AST Creation::\n{:#?}\n", ast);
            }
            ast
        }
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

fn typcheck_pass(ast: &Vec<Stmt>, args: &PassArgs) {
    let code = || match pineapple_semantics::typecheck(ast) {
        Ok(ast) => {
            if args.debug {
                println!("::Type Checking::\n{:#?}\n", ast);
            }
            ast
        }
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
