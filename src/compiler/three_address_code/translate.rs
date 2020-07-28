use crate::bytecode::constant::Constant;
use crate::bytecode::string_intern::intern_string;
use crate::compiler::three_address_code::{Expr, Operand, Stmt};
use crate::parser::ast;
use crate::parser::binop::BinOp;
use crate::parser::relop::RelOp;

use lazy_static::lazy_static;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Mutex;

type Label = usize;

lazy_static! {
    static ref STMTS: Mutex<Vec<Vec<Stmt>>> = Mutex::new(Vec::new());
    static ref BLOCK: Mutex<Vec<Stmt>> = Mutex::new(Vec::new());
    static ref REG_COUNT: Mutex<usize> = Mutex::new(0);
    static ref BACKPATCHES: Mutex<Vec<usize>> = Mutex::new(Vec::new());
}

fn get_register() -> Operand {
    let count = *REG_COUNT.lock().unwrap();
    *REG_COUNT.lock().unwrap() += 1;
    Operand::Assignable(count, 0, false)
}

fn get_label() -> usize {
    BLOCK.lock().unwrap().len()
}

fn merge_labels(b: usize) {
    let mut block = STMTS.lock().unwrap()[b].clone();

    let mut marked: Vec<usize> = vec![0; block.len()];
    let mut label_ref: HashMap<Label, Vec<usize>> = HashMap::new();

    for (i, stmt) in STMTS.lock().unwrap()[b].iter().enumerate() {
        match stmt {
            Stmt::Label(l) => {
                let labels = match label_ref.entry(*l) {
                    Entry::Occupied(o) => o.into_mut(),
                    Entry::Vacant(v) => v.insert(vec![]),
                };
                labels.push(i);
            }
            Stmt::Jump(j) => {
                let labels = match label_ref.entry(*j) {
                    Entry::Occupied(o) => o.into_mut(),
                    Entry::Vacant(v) => v.insert(vec![]),
                };
                labels.push(i);
            }
            Stmt::CJump(_, cj) => {
                let labels = match label_ref.entry(*cj) {
                    Entry::Occupied(o) => o.into_mut(),
                    Entry::Vacant(v) => v.insert(vec![]),
                };
                labels.push(i);
            }
            _ => (),
        };

        if i > 1 {
            if let Stmt::Label(main) = block[i - 1] {
                if let Stmt::Label(other) = stmt {
                    marked[i] = 1;
                    for stmt in block.iter_mut().take(i + 1) {
                        if stmt.has_label(*other) {
                            stmt.replace_label(main);
                        }
                    }
                }
            }
        }
    }

    let filtered = block
        .iter()
        .enumerate()
        .filter(|(i, _)| marked[*i] == 0)
        .map(|(_, e)| e.clone());
    STMTS.lock().unwrap()[b] = filtered.collect::<Vec<_>>();
}

pub fn translate_to_tac_ir(ast: Vec<ast::Stmt>) -> Vec<Vec<Stmt>> {
    BLOCK.lock().unwrap().push(Stmt::Label(0));
    for statement in &ast {
        if let ast::Stmt::Function(_, _, _) = statement {
            translate_statement(statement);
            let block = BLOCK.lock().unwrap().clone();
            STMTS.lock().unwrap().push(block);
            BLOCK.lock().unwrap().clear();
            BLOCK.lock().unwrap().push(Stmt::Label(0));
        } else {
            translate_statement(statement);
        }
    }

    if !BLOCK.lock().unwrap().is_empty() {
        let block = BLOCK.lock().unwrap().clone();
        STMTS.lock().unwrap().push(block);
    }

    let len = STMTS.lock().unwrap().len();
    for i in 0..len {
        merge_labels(i)
    }
    STMTS.lock().unwrap().clone()
}

fn translate_statement(stmt: &ast::Stmt) {
    match stmt {
        ast::Stmt::If(ref cond, ref block, ref other) => {
            translate_if_statement(cond, block, other);
        }
        ast::Stmt::Expression(ref expr) => {
            translate_expression(expr, false);
        }
        ast::Stmt::Block(ref stmts) => {
            for stmt in stmts {
                translate_statement(stmt);
            }
        }
        ast::Stmt::While(ref cond, ref block) => {
            translate_while_statement(cond, block);
        }
        ast::Stmt::Print(args) => {
            translate_print_statement(args);
        }
        ast::Stmt::Return(value) => translate_return(value),
        // ast::Stmt::Function(name, args, body) => translate_function(name, args, body),
        _ => unimplemented!(),
    }
}

fn translate_if_statement(cond: &ast::Expr, block: &ast::Stmt, other: &Option<Box<ast::Stmt>>) {
    let label = get_label();

    let cond = Expr::Operand(translate_expression(cond, true));
    let cjump = Stmt::CJump(cond, label);

    BLOCK.lock().unwrap().push(cjump);
    translate_statement(block);

    if let Some(stmt) = other {
        let jump = Stmt::Jump(label);

        BACKPATCHES
            .lock()
            .unwrap()
            .push(BLOCK.lock().unwrap().len());
        BLOCK.lock().unwrap().push(jump);

        BLOCK.lock().unwrap().push(Stmt::Label(label));

        translate_statement(stmt);
        match **stmt {
            ast::Stmt::If(_, _, _) => {}
            _ => {
                let label = get_label();
                BLOCK.lock().unwrap().push(Stmt::Label(label));

                for patch in BACKPATCHES.lock().unwrap().iter() {
                    BLOCK.lock().unwrap()[*patch] = Stmt::Jump(label);
                }
                BACKPATCHES.lock().unwrap().clear();
            }
        }
    } else {
        BLOCK.lock().unwrap().push(Stmt::Label(label));
    }
}

fn translate_expression(expr: &ast::Expr, is_cond: bool) -> Operand {
    match expr {
        ast::Expr::Number(d) => Operand::Constant(Constant::Number(*d)),
        ast::Expr::Boolean(b) => {
            if is_cond {
                Operand::Constant(Constant::Boolean(!*b))
            } else {
                Operand::Constant(Constant::Boolean(*b))
            }
        }
        ast::Expr::Variable(n) => Operand::Assignable(*n, 0, true),
        ast::Expr::Assign(n, l) => translate_assign(n, l),
        ast::Expr::Binary(l, o, r) => translate_binary(l, o, r),
        ast::Expr::Logical(l, o, r) => translate_logical(l, o, r),
        ast::Expr::Grouping(e) => translate_expression(e, is_cond),
        // ast::Expr::Call(e, args) => translate_call(e, args),
    }
}

// fn translate_call(e: &ast::Expr, args: &Vec<ast::Expr>) -> Operand {
//     let name = translate_expression(e, false);
//     let ident = match e {
//         ast::Expr::Variable(ident) => ident,
//         _ => panic!("Expected identifier!"),
//     };

//     for arg in args {
//         let operand = translate_expression(arg, false);
//         BLOCK.lock().unwrap().push(Stmt::StackPush(operand));
//     }

//     let call = Stmt::Call(*ident, args.len());
//     BLOCK.lock().unwrap().push(call);
//     name
// }

fn translate_while_statement(cond: &ast::Expr, block: &ast::Stmt) {
    let label = get_label();
    BLOCK.lock().unwrap().push(Stmt::Label(label));

    let cond = Expr::Operand(translate_expression(cond, true));
    let end_label = get_label();

    let cjump = Stmt::CJump(cond, end_label);

    BLOCK.lock().unwrap().push(cjump);
    translate_statement(block);

    let jump = Stmt::Jump(label);

    BLOCK.lock().unwrap().push(jump);
    BLOCK.lock().unwrap().push(Stmt::Label(end_label));
}

fn translate_print_statement(args: &[ast::Expr]) {
    let name = intern_string("print".to_string());
    for arg in args {
        let operand = translate_expression(arg, false);
        BLOCK.lock().unwrap().push(Stmt::StackPush(operand));
    }

    let call = Stmt::Call(name, args.len());
    BLOCK.lock().unwrap().push(call);
}

fn translate_return(value: &Option<Box<ast::Expr>>) {
    if let Some(expr) = value {
        let retval = translate_expression(expr, false);
        BLOCK.lock().unwrap().push(Stmt::Return(retval));
    }
}

// fn translate_function(name: &usize, args: &[usize], body: &[ast::Stmt]) {
//     BLOCK.lock().unwrap().push(Stmt::NamedLabel(*name));
//     for arg in args {
//         let lval = Operand::Assignable(*arg, 0, true);
//         let rval = Expr::StackPop;
//         let p = Stmt::Tac(lval, rval);

//         BLOCK.lock().unwrap().push(p);
//     }

//     for stmt in body {
//         translate_statement(stmt);
//     }
// }

fn translate_assign(n: &usize, l: &ast::Expr) -> Operand {
    let lval = Operand::Assignable(*n, 0, true);
    // let rval = if let ast::Expr::Call(_, _) = l {
    //     translate_expression(l, false);
    //     Expr::Operand(Operand::Return)
    // } else {
    //     Expr::Operand(translate_expression(l, false))
    // };

    let rval = Expr::Operand(translate_expression(l, false));

    let code = Stmt::Tac(lval, rval);
    BLOCK.lock().unwrap().push(code);

    lval
}

fn translate_binary(l: &ast::Expr, o: &BinOp, r: &ast::Expr) -> Operand {
    let lval = get_register();
    let rval = Expr::Binary(
        translate_expression(l, false),
        *o,
        translate_expression(r, false),
    );

    let code = Stmt::Tac(lval, rval);
    BLOCK.lock().unwrap().push(code);

    lval
}

fn translate_logical(l: &ast::Expr, o: &RelOp, r: &ast::Expr) -> Operand {
    let lval = get_register();
    let rval = Expr::Logical(
        translate_expression(l, false),
        o.flip(),
        translate_expression(r, false),
    );

    let code = Stmt::Tac(lval, rval);
    BLOCK.lock().unwrap().push(code);

    lval
}
