use pineapple_ast::ast::{Expr, Stmt};
use pineapple_error::TypeError;
use pineapple_ir::value::ValueTy;

type Ident = usize;
type Type = ValueTy;
type Args = Vec<(Ident, Type)>;

pub fn typecheck(ast: &Vec<Stmt>) -> Result<(), TypeError> {
    pineapple_session::insert_symbol_table_context();
    for stmt in ast {
        if let Stmt::Function(name, _, return_ty, _) = stmt {
            pineapple_session::insert_function_into_symbol_table(name, return_ty);
        }
    }

    for stmt in ast {
        check_stmt(stmt, None)?;
    }
    pineapple_session::pop_symbol_table_context();
    Ok(())
}

fn check_function(
    name: &Ident,
    args: &Args,
    return_ty: &Type,
    body: &Vec<Stmt>,
) -> Result<(), TypeError> {
    pineapple_session::insert_function_into_symbol_table(name, return_ty);
    pineapple_session::insert_symbol_table_context();

    for (ident, value_ty) in args {
        pineapple_session::insert_variable_into_symbol_table(ident, value_ty);
    }

    for stmt in body {
        check_stmt(stmt, Some(*return_ty))?;
    }

    pineapple_session::pop_symbol_table_context();
    Ok(())
}

fn check_stmt(stmt: &Stmt, func_return_ty: Option<Type>) -> Result<(), TypeError> {
    match stmt {
        Stmt::Function(name, args, return_ty, body) => check_function(name, args, return_ty, body),
        Stmt::Block(stmts) => {
            for stmt in stmts {
                check_stmt(stmt, None)?;
            }
            Ok(())
        }
        Stmt::If(cond, body, other) => {
            check_expr(cond, None)?;
            check_stmt(body, None)?;

            if let Some(other) = other {
                check_stmt(other, None)?;
            }
            Ok(())
        }
        Stmt::While(cond, body) => {
            check_expr(cond, None)?;
            check_stmt(body, None)
        }
        Stmt::Expression(expr) => match check_expr(expr, None) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Stmt::Print(_) => Ok(()),
        Stmt::Return(expr) => {
            if let Some(expr) = expr {
                let rtype = check_expr(expr, func_return_ty)?;
                if rtype != func_return_ty {
                    return Err(TypeError::InvalidReturnType(
                        rtype.unwrap(),
                        func_return_ty.unwrap(),
                    ));
                }
            }
            Ok(())
        }
        _ => unimplemented!("{:?}", stmt),
    }
}

fn check_expr(expr: &Expr, expected_ty: Option<Type>) -> Result<Option<Type>, TypeError> {
    match expr {
        Expr::Assign(lval, var_ty, rval) => {
            match var_ty {
                //  If the assign statement has a ty, that means it is "fresh", this identifier hasn't been assigned to before.
                //      In such a case we evaluate the expression with the type given and add it to the symbol table
                Some(ty) => {
                    check_expr(rval, *var_ty)?;
                    pineapple_session::insert_variable_into_symbol_table(lval, ty);
                }
                //  If the assign statement does not have a ty, that means it is "old", this identifier has been assigned to before.
                //      In such a case we evaluate the expression with the stored in the symbol table
                None => {
                    let ty = match pineapple_session::get_variable_ty(lval) {
                        Some(ty) => ty,
                        None => panic!("undefined variable"),
                    };
                    check_expr(rval, Some(ty))?;
                }
            }
            Ok(None)
        }
        Expr::Binary(left, op, right) => {
            check_expr(left, expected_ty)?;
            check_expr(right, expected_ty)
        }
        Expr::Logical(left, op, right) => {
            check_expr(left, expected_ty)?;
            check_expr(right, expected_ty)
        }
        Expr::Grouping(group) => check_expr(group, expected_ty),
        Expr::Variable(ident) => match (pineapple_session::get_variable_ty(ident), expected_ty) {
            (Some(ty), Some(expected_ty)) => {
                // If we have a variable in our symbol table AND it matches our expected type, we pass
                if ty == expected_ty {
                    Ok(Some(expected_ty))
                } else {
                    Err(TypeError::InvalidVariableType(*ident, ty, expected_ty))
                }
            }
            _ => Err(TypeError::UndefinedVariable(*ident)),
        },
        Expr::Value(value) => {
            if let Some(ty) = expected_ty {
                // If the value is not of the expected type we then we error
                let value_ty = value.get_ty();
                if value_ty != ty {
                    return Err(TypeError::InvalidValueType(value.clone(), value_ty, ty));
                }
                Ok(expected_ty)
            } else {
                Ok(expected_ty)
            }
        }
        _ => unimplemented!(),
    }
}
