use crate::core::value::Type;
use crate::parser::ast::{Expr, Stmt};
use crate::semantic_analysis::error::TypeError;
use core::cell::RefCell;
use std::collections::HashMap;

type Args = Vec<(usize, Type)>;

thread_local! {
    static FUN_SYMBOL_TABLE: RefCell<HashMap<usize, (Type, Args)>> = RefCell::new(HashMap::new());
    static VAR_SYMBOL_TABLE: RefCell<HashMap<usize, Type>> = RefCell::new(HashMap::new());
}

pub fn type_check(ast: &Vec<Stmt>) -> Result<(), TypeError> {
    for stmt in ast {
        if let Stmt::Function(name, args, return_type, body) = stmt {
            check_function(name, args, return_type, body)?;
        }
    }

    for stmt in ast {
        check_stmt(stmt, None)?;
    }
    Ok(())
}

fn check_function(
    name: &usize,
    args: &Args,
    return_type: &Type,
    body: &Vec<Stmt>,
) -> Result<(), TypeError> {
    FUN_SYMBOL_TABLE.with(|symbol_table| {
        symbol_table
            .borrow_mut()
            .insert(*name, (*return_type, args.clone()));
    });

    for arg in args {
        VAR_SYMBOL_TABLE.with(|symbol_table| {
            symbol_table.borrow_mut().insert(arg.0, arg.1);
        });
    }

    for stmt in body {
        check_stmt(stmt, Some(*return_type))?;
    }

    if let Stmt::Return(retval) = body.last().unwrap() {
        if let Some(retval) = retval {
            check_expr(retval, Some(*return_type))?;
        }
    }

    Ok(())
}

fn check_stmt(stmt: &Stmt, expected_type: Option<Type>) -> Result<(), TypeError> {
    match stmt {
        Stmt::Function(name, args, return_type, body) => {
            check_function(name, args, return_type, body)
        }
        Stmt::Block(stmts) => {
            for stmt in stmts {
                check_stmt(stmt, expected_type)?;
            }
            Ok(())
        }
        Stmt::If(cond, body, other) => {
            check_expr(cond, None)?;
            check_stmt(body, expected_type)?;

            if let Some(other) = other {
                check_stmt(other, expected_type)?;
            }
            Ok(())
        }
        Stmt::Expression(expr) => check_expr(expr, None),
        Stmt::Print(_) => Ok(()),
        Stmt::Return(expr) => {
            if let Some(expr) = expr {
                check_expr(expr, expected_type)?;
            }
            Ok(())
        }
        _ => unimplemented!("{:?}", stmt),
    }
}

fn check_expr(expr: &Expr, expected_type: Option<Type>) -> Result<(), TypeError> {
    match expr {
        Expr::Assign(lval, var_type, rval) => {
            VAR_SYMBOL_TABLE.with(|symbol_table| {
                symbol_table.borrow_mut().insert(*lval, *var_type);
            });
            check_expr(rval, Some(*var_type))
        }
        Expr::Binary(left, op, right) => {
            check_expr(left, expected_type)?;
            check_expr(right, expected_type)?;
            Ok(())
        }
        Expr::Logical(left, op, right) => {
            check_expr(left, expected_type)?;
            check_expr(right, expected_type)?;
            Ok(())
        }
        Expr::Grouping(group) => check_expr(group, expected_type),
        Expr::Variable(sym) => {
            if let Some(expected_type) = expected_type {
                VAR_SYMBOL_TABLE.with(|symbol_table| match symbol_table.borrow().get(sym) {
                    Some(actual_type) => {
                        if actual_type == &expected_type {
                            Ok(())
                        } else {
                            let symbol_table = symbol_table.borrow();
                            let actual_type = symbol_table.get(sym).unwrap();
                            Err(TypeError::InvalidVariableType(
                                *sym,
                                expected_type,
                                *actual_type,
                            ))
                        }
                    }
                    None => Err(TypeError::UndefinedVariable(*sym)),
                })
            } else {
                Err(TypeError::ExpectedNestedType)
            }
        }
        Expr::Value(value) => {
            if let Some(expected_type) = expected_type {
                if let Err(_) = value.try_cast_to(expected_type) {
                    return Err(TypeError::InvalidValueType(value.clone(), expected_type));
                } else {
                    Ok(())
                }
            } else {
                Err(TypeError::ExpectedNestedType)
            }
        }
        Expr::Call(left, args) => {
            if let Expr::Variable(sym) = left.as_ref() {
                let payload = FUN_SYMBOL_TABLE.with(|symbol_table| {
                    if let Some(x) = symbol_table.borrow().get(sym) {
                        Ok(x.clone())
                    } else {
                        return Err(TypeError::UndefinedFunction(*sym));
                    }
                });

                let (_, fargs) = match payload {
                    Ok(x) => x,
                    Err(_) => return Err(TypeError::UndefinedFunction(*sym)),
                };

                if args.len() != fargs.len() {
                    return Err(TypeError::FunctionArityMismatch(
                        *sym,
                        fargs.len(),
                        args.len(),
                    ));
                }

                for i in 0..args.len() {
                    if let Expr::Value(value) = args[i] {
                        if let Err(_) = value.try_cast_to(fargs[i].1) {
                            return Err(TypeError::InvalidValueType(value, fargs[i].1));
                        }
                    } else if let Expr::Variable(sym) = args[i] {
                        VAR_SYMBOL_TABLE.with(|symbol_table| {
                            match symbol_table.borrow().get(&sym) {
                                Some(actual_type) => {
                                    if actual_type == &fargs[i].1 {
                                        Ok(())
                                    } else {
                                        let symbol_table = symbol_table.borrow();
                                        let actual_type = symbol_table.get(&sym).unwrap();
                                        Err(TypeError::InvalidVariableType(
                                            sym,
                                            fargs[i].1,
                                            *actual_type,
                                        ))
                                    }
                                }
                                None => Err(TypeError::UndefinedVariable(sym)),
                            }
                        });
                    }
                }
            }
            Ok(())
        }
        _ => unimplemented!(),
    }
}
